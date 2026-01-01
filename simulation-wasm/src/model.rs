use crate::enums::*;
pub use crate::enums::ActionCondition; // Explicitly re-export ActionCondition
use crate::resources::{ActionCost, ActionRequirement, ActionTag, ResourceType};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum MonsterRole {
    Boss,
    Brute,
    Striker,
    Controller,
    Minion,
    Unknown,
}

/// Encounter tier classification based on death percentile analysis and resource drain
/// Based on gemini-3-pro-preview analysis for D&D 5e encounter balancing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EncounterTier {
    /// Trivial encounter: CR < 1/8 party level, ~0 resource drain
    /// Used for late-position targets where even Safe would be too difficult
    Trivial = -1,
    /// Safe encounter (~6 per day): P99 <= 0, P50 = 0, P1 <= 1 deaths, 10-30% drain
    Safe = 0,
    /// Challenging encounter (~1 per day): P99 <= 1, P50 <= 1, P1 <= 2 deaths, 30-50% drain
    Challenging = 1,
    /// Boss encounter (~1 per day): P99 <= 2, P50 1-3, P1 <= 4 deaths (NO TPK), 50-80% drain
    Boss = 2,
    /// Failed encounter: TPK occurred (all party members dead) or P1 deaths exceed thresholds
    Failed = 3,
}

/// Metrics for classifying encounters into tiers
/// Based on 100-run simulation percentile analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncounterMetrics {
    /// Worst 1% of outcomes (most deaths)
    pub deaths_p1: usize,
    /// Median outcome (50th percentile)
    pub deaths_p50: usize,
    /// Best 1% of outcomes (fewest deaths)
    pub deaths_p99: usize,
    /// Percentage of resources drained (0-100)
    pub resource_drain_percent: f64,
    /// Total party size (for TPK detection)
    pub party_size: usize,
}

impl EncounterMetrics {
    /// Classify this encounter into a tier based on death percentiles and resource drain
    pub fn classify(&self) -> EncounterTier {
        // Check for TPK (Total Party Kill) - all party members dead in worst 1%
        if self.deaths_p1 >= self.party_size {
            return EncounterTier::Failed;
        }

        // Check Trivial tier criteria: virtually no deaths, minimal drain
        // P99 = 0, P50 = 0, P1 = 0, <10% drain
        if self.deaths_p99 == 0
            && self.deaths_p50 == 0
            && self.deaths_p1 == 0
            && self.resource_drain_percent < 10.0
        {
            return EncounterTier::Trivial;
        }

        // Check Safe tier criteria first (so boundary values go to safer tier)
        // P99 <= 0, P50 = 0, P1 <= 1, 10-30% drain
        let safe_deaths_p99 = self.deaths_p99 == 0;
        let safe_deaths_p50 = self.deaths_p50 == 0;
        let safe_deaths_p1 = self.deaths_p1 <= 1;
        let safe_drain = (10.0..=30.0).contains(&self.resource_drain_percent);

        if safe_deaths_p99 && safe_deaths_p50 && safe_deaths_p1 && safe_drain {
            return EncounterTier::Safe;
        }

        // Check Challenging tier criteria (so boundary 50% goes to Challenging, not Boss)
        // P99 <= 1, P50 <= 1, P1 <= 2, 30-50% drain
        let challenging_deaths_p99 = self.deaths_p99 <= 1;
        let challenging_deaths_p50 = self.deaths_p50 <= 1;
        let challenging_deaths_p1 = self.deaths_p1 <= 2;
        let challenging_drain = (30.0..=50.0).contains(&self.resource_drain_percent);

        if challenging_deaths_p99 && challenging_deaths_p50 && challenging_deaths_p1 && challenging_drain {
            return EncounterTier::Challenging;
        }

        // Check Boss tier criteria
        // P99 <= 2, P50 1-3, P1 <= 4, 50-80% drain
        let boss_deaths_p99 = self.deaths_p99 <= 2;
        let boss_deaths_p50 = (1..=3).contains(&self.deaths_p50);
        let boss_deaths_p1 = self.deaths_p1 <= 4;
        let boss_drain = (50.0..=80.0).contains(&self.resource_drain_percent);

        if boss_deaths_p99 && boss_deaths_p50 && boss_deaths_p1 && boss_drain {
            return EncounterTier::Boss;
        }

        // If no tier matches, classify as Failed (outside acceptable bounds)
        EncounterTier::Failed
    }
}

/// Reverse lookup: calculate required isolated tier to achieve target contextual tier
/// given current resource state. Returns None if impossible (would require tier < Trivial).
///
/// Based on gemini-3-pro-preview analysis:
/// - 85-100% resources: no penalty
/// - 70-84% resources: +1 tier harder
/// - 40-69% resources: +2 tiers harder
/// - <40% resources: +3 tiers harder
///
/// Example: At 50% resources (penalty +2), to achieve contextual Challenging (1),
///          we need isolated Trivial (-1), because: Trivial + 2 = Challenging
pub fn required_isolated_tier_for_contextual(
    target_contextual: EncounterTier,
    resources_remaining_percent: f64,
) -> Option<EncounterTier> {
    // Failed always requires Failed (or worse, which doesn't exist)
    if target_contextual == EncounterTier::Failed {
        return Some(EncounterTier::Failed);
    }

    let penalty = if resources_remaining_percent >= 85.0 {
        0
    } else if resources_remaining_percent >= 70.0 {
        1
    } else if resources_remaining_percent >= 40.0 {
        2
    } else {
        3
    };

    // Calculate required isolated tier: Contextual = Isolated + Penalty
    // So: Isolated = Contextual - Penalty
    let target_num = target_contextual as i32 - penalty;

    // Check if below minimum (Trivial = -1)
    if target_num < EncounterTier::Trivial as i32 {
        None
    } else {
        // Convert i32 back to EncounterTier
        match target_num {
            -1 => Some(EncounterTier::Trivial),
            0 => Some(EncounterTier::Safe),
            1 => Some(EncounterTier::Challenging),
            2 => Some(EncounterTier::Boss),
            3 => Some(EncounterTier::Failed),
            _ => None, // Should never happen
        }
    }
}

impl EncounterTier {
    /// Get the numeric value of this tier for calculations
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Get the display name of this tier
    pub fn name(self) -> &'static str {
        match self {
            EncounterTier::Trivial => "Trivial",
            EncounterTier::Safe => "Safe",
            EncounterTier::Challenging => "Challenging",
            EncounterTier::Boss => "Boss",
            EncounterTier::Failed => "Failed",
        }
    }
}

/// Contextual encounter difficulty accounting for resource depletion
/// Measures actual difficulty at a specific position in an adventuring day
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextualEncounterMetrics {
    /// Which encounter number in the day (1-indexed)
    pub position_in_day: usize,
    /// Percentage of total resources remaining (0-100)
    pub resources_remaining_percent: f64,
    /// Isolated encounter tier (assuming full resources)
    pub isolated_tier: EncounterTier,
    /// Contextual encounter tier (adjusted for resource state)
    pub contextual_tier: EncounterTier,
    /// Cumulative HP lost across party before this encounter
    pub cumulative_hp_lost: f64,
    /// Total party HP (for percentage calculations)
    pub total_party_hp: f64,
    /// Number of survivors entering this encounter
    pub survivors_entering: usize,
}

impl ContextualEncounterMetrics {
    /// Calculate contextual tier based on resource state
    /// Resources act as a difficulty multiplier - fewer resources = harder encounter
    pub fn calculate_contextual_tier(
        isolated_tier: EncounterTier,
        resources_remaining_percent: f64,
    ) -> EncounterTier {
        // If TPK in isolated, it's definitely Failed
        if isolated_tier == EncounterTier::Failed {
            return EncounterTier::Failed;
        }

        // Trivial can become Safe or Challenging with resource depletion
        match resources_remaining_percent {
            // 85-100%: No adjustment (early adventuring day)
            r if r >= 85.0 => isolated_tier,

            // 70-84%: One tier harder (mid-day encounters)
            r if r >= 70.0 => match isolated_tier {
                EncounterTier::Trivial => EncounterTier::Safe,
                EncounterTier::Safe => EncounterTier::Challenging,
                EncounterTier::Challenging => EncounterTier::Boss,
                EncounterTier::Boss => EncounterTier::Failed,
                EncounterTier::Failed => EncounterTier::Failed,
            },

            // 40-69%: Two tiers harder (late-day encounters)
            r if r >= 40.0 => match isolated_tier {
                EncounterTier::Trivial => EncounterTier::Challenging,
                EncounterTier::Safe => EncounterTier::Boss,
                EncounterTier::Challenging => EncounterTier::Failed,
                EncounterTier::Boss => EncounterTier::Failed,
                EncounterTier::Failed => EncounterTier::Failed,
            },

            // <40%: Three tiers harder (everything becomes Failed)
            _ => match isolated_tier {
                EncounterTier::Trivial => EncounterTier::Boss,
                EncounterTier::Safe => EncounterTier::Failed,
                EncounterTier::Challenging => EncounterTier::Failed,
                EncounterTier::Boss => EncounterTier::Failed,
                EncounterTier::Failed => EncounterTier::Failed,
            },
        }
    }

    /// Create contextual metrics from isolated metrics and resource state
    pub fn from_isolated_metrics(
        position: usize,
        resources_remaining_percent: f64,
        isolated_tier: EncounterTier,
        cumulative_hp_lost: f64,
        total_party_hp: f64,
        survivors_entering: usize,
    ) -> Self {
        let contextual_tier =
            Self::calculate_contextual_tier(isolated_tier, resources_remaining_percent);

        Self {
            position_in_day: position,
            resources_remaining_percent,
            isolated_tier,
            contextual_tier,
            cumulative_hp_lost,
            total_party_hp,
            survivors_entering,
        }
    }

    /// Get a human-readable description of the difficulty shift
    pub fn difficulty_shift_description(&self) -> String {
        if self.isolated_tier == self.contextual_tier {
            format!(
                "Encounter #{}: {} (No resource impact - {}% resources remaining)",
                self.position_in_day,
                tier_name(&self.isolated_tier),
                self.resources_remaining_percent as i32
            )
        } else {
            format!(
                "Encounter #{}: {} → {} (Resource depletion - {}% resources remaining)",
                self.position_in_day,
                tier_name(&self.isolated_tier),
                tier_name(&self.contextual_tier),
                self.resources_remaining_percent as i32
            )
        }
    }
}

/// Helper function to get tier name for display
fn tier_name(tier: &EncounterTier) -> &'static str {
    match tier {
        EncounterTier::Trivial => "Trivial",
        EncounterTier::Safe => "Safe",
        EncounterTier::Challenging => "Challenging",
        EncounterTier::Boss => "Boss",
        EncounterTier::Failed => "Failed",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Hash)]
pub enum TargetRole {
    Skirmish,
    #[default]
    Standard,
    Elite,
    Boss,
}

impl TargetRole {
    pub fn weight(&self) -> f64 {
        match self {
            TargetRole::Skirmish => 1.0,
            TargetRole::Standard => 2.0,
            TargetRole::Elite => 3.0,
            TargetRole::Boss => 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DiceFormula {
    Value(f64),
    Expr(String),
}

impl Hash for DiceFormula {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DiceFormula::Value(v) => {
                0.hash(state);
                crate::utilities::hash_f64(*v, state);
            }
            DiceFormula::Expr(s) => {
                1.hash(state);
                s.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(untagged)]
pub enum Frequency {
    Static(String), // "at will", "1/fight", "1/day"
    Recharge {
        reset: String, // "recharge"
        #[serde(rename = "cooldownRounds")]
        cooldown_rounds: i32,
    },
    Limited {
        reset: String, // "sr", "lr"
        uses: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct EffectTrigger {
    pub condition: TriggerCondition,
    #[serde(default)]
    pub requirements: Vec<TriggerRequirement>,
    pub effect: TriggerEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Buff {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub duration: BuffDuration,

    pub ac: Option<DiceFormula>,
    #[serde(rename = "toHit")]
    pub to_hit: Option<DiceFormula>,
    pub damage: Option<DiceFormula>,
    #[serde(rename = "damageReduction")]
    pub damage_reduction: Option<DiceFormula>,
    #[serde(rename = "damageMultiplier")]
    pub damage_multiplier: Option<f64>,
    #[serde(rename = "damageTakenMultiplier")]
    pub damage_taken_multiplier: Option<f64>,
    pub dc: Option<DiceFormula>,
    pub save: Option<DiceFormula>,
    pub condition: Option<CreatureCondition>,

    pub magnitude: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub concentration: bool,
    #[serde(default)]
    pub triggers: Vec<EffectTrigger>,
}

impl Hash for Buff {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.display_name.hash(state);
        self.duration.hash(state);
        self.ac.hash(state);
        self.to_hit.hash(state);
        self.damage.hash(state);
        self.damage_reduction.hash(state);
        crate::utilities::hash_opt_f64(self.damage_multiplier, state);
        crate::utilities::hash_opt_f64(self.damage_taken_multiplier, state);
        self.dc.hash(state);
        self.save.hash(state);
        self.condition.hash(state);
        crate::utilities::hash_opt_f64(self.magnitude, state);
        self.source.hash(state);
        self.concentration.hash(state);
        self.triggers.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiderEffect {
    pub dc: f64,
    pub buff: Buff,
}

impl Hash for RiderEffect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        crate::utilities::hash_f64(self.dc, state);
        self.buff.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct AcKnowledge {
    pub min: i32,
    pub max: i32,
}

impl Default for AcKnowledge {
    fn default() -> Self {
        Self { min: 0, max: 30 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "atk")]
    Atk(AtkAction),
    #[serde(rename = "heal")]
    Heal(HealAction),
    #[serde(rename = "buff")]
    Buff(BuffAction),
    #[serde(rename = "debuff")]
    Debuff(DebuffAction),
    #[serde(rename = "template")]
    Template(TemplateAction),
}

impl Action {
    pub fn base(&self) -> ActionBase {
        match self {
            Action::Atk(a) => a.base(),
            Action::Heal(a) => a.base(),
            Action::Buff(a) => a.base(),
            Action::Debuff(a) => a.base(),
            Action::Template(a) => a.base(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct ActionBase {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct AtkAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub dpr: DiceFormula,
    #[serde(rename = "toHit")]
    pub to_hit: DiceFormula,
    pub target: EnemyTarget,
    #[serde(rename = "useSaves")]
    pub use_saves: Option<bool>,
    #[serde(rename = "halfOnSave")]
    pub half_on_save: Option<bool>,
    #[serde(rename = "riderEffect")]
    pub rider_effect: Option<RiderEffect>,
}

impl AtkAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct HealAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub amount: DiceFormula,
    #[serde(rename = "tempHP")]
    pub temp_hp: Option<bool>,
    pub target: AllyTarget,
}

impl HealAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct BuffAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub target: AllyTarget,
    pub buff: Buff,
}

impl BuffAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebuffAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub target: EnemyTarget,
    #[serde(rename = "saveDC")]
    pub save_dc: f64,
    pub buff: Buff,
}

impl Hash for DebuffAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.action_slot.hash(state);
        self.cost.hash(state);
        self.requirements.hash(state);
        self.tags.hash(state);
        self.freq.hash(state);
        self.condition.hash(state);
        self.targets.hash(state);
        self.target.hash(state);
        crate::utilities::hash_f64(self.save_dc, state);
        self.buff.hash(state);
    }
}

impl DebuffAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct TemplateAction {
    pub id: String,
    #[serde(default, deserialize_with = "default_template_name")]
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    #[serde(default = "default_targets")]
    pub targets: i32,

    #[serde(rename = "templateOptions")]
    pub template_options: TemplateOptions,
}

fn default_targets() -> i32 {
    1
}

fn default_template_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Try to deserialize the name field, if it doesn't exist, return empty string
    // The actual name will be set from template_options.template_name in a post-processing step
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateOptions {
    #[serde(rename = "templateName")]
    pub template_name: String,
    // Optional target that can be either ally or enemy
    pub target: Option<TargetType>,
    // Add other options as needed, like saveDC, amount, etc.
    #[serde(rename = "saveDC")]
    pub save_dc: Option<f64>,
    pub amount: Option<DiceFormula>,
}

impl Hash for TemplateOptions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.template_name.hash(state);
        self.target.hash(state);
        crate::utilities::hash_opt_f64(self.save_dc, state);
        self.amount.hash(state);
    }
}

impl TemplateAction {
    pub fn base(&self) -> ActionBase {
        // Use template_name if name is empty
        let name = if self.name.is_empty() {
            self.template_options.template_name.clone()
        } else {
            self.name.clone()
        };

        ActionBase {
            id: self.id.clone(),
            name,
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct ActionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub action: Action,    // The action to execute when triggered
    pub cost: Option<i32>, // e.g. Reaction (4)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum CleanupInstruction {
    RemoveAllBuffsFromSource(String), // Combatant ID of the source that died
    BreakConcentration(String, String), // (Combatant ID of concentrator, Buff ID)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Creature {
    pub id: String,
    pub arrival: Option<i32>,
    #[serde(default)]
    pub mode: String, // "player", "monster", "custom"
    pub name: String,
    pub count: f64, // TS uses number, but usually integer.
    pub hp: u32,
    #[serde(alias = "AC")]
    pub ac: u32,
    #[serde(rename = "speed_fly")]
    pub speed_fly: Option<f64>,

    // Save bonuses - average is required, individual are optional overrides
    #[serde(rename = "saveBonus")]
    pub save_bonus: f64,
    #[serde(
        default,
        rename = "strSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub str_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "dexSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub dex_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "conSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub con_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "intSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub int_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "wisSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub wis_save_bonus: Option<f64>,
    #[serde(
        default,
        rename = "chaSaveBonus",
        skip_serializing_if = "Option::is_none"
    )]
    pub cha_save_bonus: Option<f64>,

    // Advantage on saves
    #[serde(
        default,
        rename = "conSaveAdvantage",
        skip_serializing_if = "Option::is_none"
    )]
    pub con_save_advantage: Option<bool>,
    #[serde(
        default,
        rename = "saveAdvantage",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_advantage: Option<bool>, // Advantage on ALL saves (e.g. Paladin Aura)

    #[serde(default = "default_initiative_bonus")]
    #[serde(rename = "initiativeBonus")]
    pub initiative_bonus: DiceFormula,

    #[serde(default)]
    #[serde(rename = "initiativeAdvantage")]
    pub initiative_advantage: bool,
    pub actions: Vec<Action>, // This might need to be flexible if templates are involved
    #[serde(default)]
    pub triggers: Vec<ActionTrigger>,
    #[serde(rename = "spellSlots")]
    pub spell_slots: Option<HashMap<String, i32>>,
    #[serde(rename = "classResources")]
    pub class_resources: Option<HashMap<String, i32>>,
    #[serde(rename = "hitDice")]
    pub hit_dice: Option<String>, // Changed from DiceFormula
    #[serde(rename = "conModifier")]
    pub con_modifier: Option<f64>, // New field for constitution modifier to apply to hit dice rolls
}

impl Hash for Creature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.arrival.hash(state);
        self.mode.hash(state);
        self.name.hash(state);
        crate::utilities::hash_f64(self.count, state);
        self.hp.hash(state);
        self.ac.hash(state);
        crate::utilities::hash_opt_f64(self.speed_fly, state);
        crate::utilities::hash_f64(self.save_bonus, state);
        crate::utilities::hash_opt_f64(self.str_save_bonus, state);
        crate::utilities::hash_opt_f64(self.dex_save_bonus, state);
        crate::utilities::hash_opt_f64(self.con_save_bonus, state);
        crate::utilities::hash_opt_f64(self.int_save_bonus, state);
        crate::utilities::hash_opt_f64(self.wis_save_bonus, state);
        crate::utilities::hash_opt_f64(self.cha_save_bonus, state);
        self.con_save_advantage.hash(state);
        self.save_advantage.hash(state);
        self.initiative_bonus.hash(state);
        self.initiative_advantage.hash(state);
        self.actions.hash(state);
        self.triggers.hash(state);
        
        // HashMap hashing needs sorting for determinism
        if let Some(slots) = &self.spell_slots {
            let mut sorted_slots: Vec<_> = slots.iter().collect();
            sorted_slots.sort_by_key(|a| a.0);
            sorted_slots.hash(state);
        } else {
            None::<()>.hash(state);
        }

        if let Some(res) = &self.class_resources {
            let mut sorted_res: Vec<_> = res.iter().collect();
            sorted_res.sort_by_key(|a| a.0);
            sorted_res.hash(state);
        } else {
            None::<()>.hash(state);
        }

        self.hit_dice.hash(state);
        crate::utilities::hash_opt_f64(self.con_modifier, state);
    }
}

fn default_initiative_bonus() -> DiceFormula {
    DiceFormula::Value(0.0)
}

impl Creature {
    pub fn initialize_ledger(&self) -> crate::resources::ResourceLedger {
        let mut ledger = crate::resources::ResourceLedger::new();

        // Add standard resources
        ledger.register_resource(
            crate::resources::ResourceType::Action,
            None,
            1.0,
            Some(crate::resources::ResetType::Turn),
        );
        ledger.register_resource(
            crate::resources::ResourceType::BonusAction,
            None,
            1.0,
            Some(crate::resources::ResetType::Turn),
        );
        ledger.register_resource(
            crate::resources::ResourceType::Reaction,
            None,
            1.0,
            Some(crate::resources::ResetType::Round),
        );
        ledger.register_resource(
            crate::resources::ResourceType::Movement,
            None,
            30.0,
            Some(crate::resources::ResetType::Turn),
        ); // Default 30ft, should use self.speed if available

        // Add spell slots
        if let Some(slots) = &self.spell_slots {
            for (level_str, count) in slots {
                // Try to extract digit from string (e.g. "1st" -> 1)
                let cleaned_level = level_str.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
                if let Ok(level) = cleaned_level.parse::<u8>() {
                    let resource_type = ResourceType::SpellSlot; 
                    ledger.register_resource(
                        resource_type,
                        Some(&level.to_string()),
                        *count as f64,
                        Some(crate::resources::ResetType::LongRest),
                    );
                }
            }
        }

        // Add class resources
        if let Some(resources) = &self.class_resources {
            for (name, count) in resources {
                let resource_type = ResourceType::ClassResource; // Use resources::ResourceType
                ledger.register_resource(
                    resource_type,
                    Some(name),
                    *count as f64,
                    Some(crate::resources::ResetType::LongRest),
                );
            }
        }

        // Add Hit Dice resource
        if let Some(hit_dice_expr) = &self.hit_dice {
            let s = hit_dice_expr.replace(" ", "");
            let mut current_term = String::new();

            for c in s.chars() {
                if c == '+' || c == '-' {
                    if !current_term.is_empty() {
                        register_hit_dice_term(&mut ledger, &current_term);
                        current_term.clear();
                    }
                } else {
                    current_term.push(c);
                }
            }
            if !current_term.is_empty() {
                register_hit_dice_term(&mut ledger, &current_term);
            }
        }

        ledger
    }
}

// Helper function to register a single hit dice term (e.g., "3d8")
fn register_hit_dice_term(ledger: &mut crate::resources::ResourceLedger, term: &str) {
    let cleaned_term = if let Some(bracket_pos) = term.find('[') {
        &term[..bracket_pos]
    } else {
        term
    };

    if cleaned_term.contains('d') {
        let parts: Vec<&str> = cleaned_term.split('d').collect();
        if parts.len() == 2 {
            let count = parts[0].parse::<i32>().unwrap_or(1); // "d8" -> count 1
            let count = if count == 0 && parts[0].is_empty() {
                1
            } else {
                count
            };
            let sides = parts[1].parse::<i32>().unwrap_or(6); // Default to d6 if parse fails

            let resource_type = match sides {
                6 => ResourceType::HitDiceD6,   // Use resources::ResourceType
                8 => ResourceType::HitDiceD8,   // Use resources::ResourceType
                10 => ResourceType::HitDiceD10, // Use resources::ResourceType
                12 => ResourceType::HitDiceD12, // Use resources::ResourceType
                _ => {
                    // Log a warning or error for unsupported hit dice size
                    eprintln!(
                        "Warning: Unsupported hit dice size 'd{}' for term '{}'",
                        sides, term
                    );
                    return;
                }
            };

            ledger.register_resource(
                resource_type,
                None,
                count as f64,
                Some(crate::resources::ResetType::LongRest),
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreatureState {
    #[serde(rename = "currentHp")]
    pub current_hp: u32,
    #[serde(rename = "tempHp")]
    pub temp_hp: Option<u32>,
    pub buffs: HashMap<String, Buff>,

    // Use SerializableResourceLedger for frontend compatibility
    #[serde(default = "default_serializable_resource_ledger")]
    pub resources: SerializableResourceLedger,

    #[serde(rename = "upcomingBuffs")]
    pub upcoming_buffs: HashMap<String, Buff>,
    #[serde(rename = "usedActions")]
    pub used_actions: HashSet<String>,
    #[serde(rename = "concentratingOn")]
    pub concentrating_on: Option<String>,
    pub actions_used_this_encounter: HashSet<String>,
    #[serde(rename = "bonusActionUsed")]
    pub bonus_action_used: bool,
    #[serde(default)]
    pub known_ac: HashMap<String, AcKnowledge>,
    #[serde(
        rename = "arcaneWardHp",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arcane_ward_hp: Option<u32>,
}

fn default_serializable_resource_ledger() -> SerializableResourceLedger {
    SerializableResourceLedger::from(crate::resources::ResourceLedger::new())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableResourceLedger {
    pub current: HashMap<String, f64>,
    pub max: HashMap<String, f64>,
}

impl From<crate::resources::ResourceLedger> for SerializableResourceLedger {
    fn from(ledger: crate::resources::ResourceLedger) -> Self {
        let current = ledger.current.into_iter().collect();

        let max = ledger.max.into_iter().collect();

        SerializableResourceLedger { current, max }
    }
}

impl From<SerializableResourceLedger> for crate::resources::ResourceLedger {
    fn from(ledger: SerializableResourceLedger) -> Self {
        crate::resources::ResourceLedger {
            current: ledger.current,
            max: ledger.max,
            reset_rules: HashMap::new(),
        }
    }
}

// We also need a way to convert back if needed, or just use it for output
// For now, it's mostly for output to frontend.

impl Default for CreatureState {
    fn default() -> Self {
        CreatureState {
            current_hp: 0,
            temp_hp: None,
            buffs: HashMap::new(),
            resources: default_serializable_resource_ledger(),
            upcoming_buffs: HashMap::new(),
            used_actions: HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: HashSet::new(),
            bonus_action_used: false,
            known_ac: HashMap::new(),
            arcane_ward_hp: None,
        }
    }
}

// Combattant now uses Arc<Creature> for shared ownership.
// When cloning a Combattant, the Creature is not deep-copied - only the Arc pointer is copied.
// This significantly reduces memory usage when creating many clones (e.g., in simulation iterations).
// Arc is used instead of Rc for thread safety (Send + Sync).
#[derive(Debug, PartialEq)]
pub struct Combattant {
    pub id: String,
    pub team: u32, // 0 for Team 1 (Players), 1 for Team 2 (Monsters) - defaults to 0
    pub creature: Arc<Creature>,
    pub initiative: f64, // defaults to 0.0
    pub initial_state: CreatureState,
    pub final_state: CreatureState,
    pub actions: Vec<CombattantAction>,
}

// Manual Clone implementation - cheap! Only clones the Arc pointer, not the Creature
impl Clone for Combattant {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            team: self.team,
            creature: Arc::clone(&self.creature), // Just copies the Arc pointer
            initiative: self.initiative,
            initial_state: self.initial_state.clone(),
            final_state: self.final_state.clone(),
            actions: self.actions.clone(),
        }
    }
}

// Manual Serialize implementation - delegates to inner Creature
impl Serialize for Combattant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Combattant", 6)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("team", &self.team)?;
        state.serialize_field("creature", self.creature.as_ref())?;
        state.serialize_field("initiative", &self.initiative)?;
        state.serialize_field("initialState", &self.initial_state)?;
        state.serialize_field("finalState", &self.final_state)?;
        state.serialize_field("actions", &self.actions)?;
        state.end()
    }
}

// Manual Deserialize implementation - reconstructs Arc<Creature>
impl<'de> Deserialize<'de> for Combattant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CombattantHelper {
            id: String,
            #[serde(default)]
            team: u32,
            creature: Creature,
            #[serde(default)]
            initiative: f64,
            #[serde(rename = "initialState")]
            initial_state: CreatureState,
            #[serde(rename = "finalState")]
            final_state: CreatureState,
            actions: Vec<CombattantAction>,
        }

        let helper = CombattantHelper::deserialize(deserializer)?;
        Ok(Combattant {
            id: helper.id,
            team: helper.team,
            creature: Arc::new(helper.creature),
            initiative: helper.initiative,
            initial_state: helper.initial_state,
            final_state: helper.final_state,
            actions: helper.actions,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombattantAction {
    pub action: Action, // Should be FinalAction
    pub targets: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Encounter {
    pub monsters: Vec<Creature>,
    #[serde(rename = "playersSurprised")]
    pub players_surprised: Option<bool>,
    #[serde(rename = "monstersSurprised")]
    pub monsters_surprised: Option<bool>,
    #[serde(rename = "playersPrecast")]
    pub players_precast: Option<bool>, // New field
    #[serde(rename = "monstersPrecast")]
    pub monsters_precast: Option<bool>, // New field
    #[serde(rename = "targetRole", default)]
    pub target_role: TargetRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct ShortRest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum TimelineStep {
    #[serde(rename = "combat")]
    Combat(Encounter),
    #[serde(rename = "shortRest")]
    ShortRest(ShortRest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncounterStats {
    #[serde(rename = "damageDealt")]
    pub damage_dealt: f64,
    #[serde(rename = "damageTaken")]
    pub damage_taken: f64,
    #[serde(rename = "healGiven")]
    pub heal_given: f64,
    #[serde(rename = "healReceived")]
    pub heal_received: f64,
    #[serde(rename = "charactersBuffed")]
    pub characters_buffed: f64,
    #[serde(rename = "buffsReceived")]
    pub buffs_received: f64,
    #[serde(rename = "charactersDebuffed")]
    pub characters_debuffed: f64,
    #[serde(rename = "debuffsReceived")]
    pub debuffs_received: f64,
    #[serde(rename = "timesUnconscious")]
    pub times_unconscious: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Round {
    pub team1: Vec<Combattant>,
    pub team2: Vec<Combattant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncounterResult {
    pub stats: HashMap<String, EncounterStats>,
    pub rounds: Vec<Round>,
    #[serde(rename = "targetRole", default)]
    pub target_role: TargetRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationRunData {
    pub encounters: Vec<EncounterResult>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(rename = "numCombatEncounters", default)]
    pub num_combat_encounters: usize,
    /// The RNG seed used for this run - enables exact reproduction
    #[serde(default)]
    pub seed: u64,
}

impl std::ops::Deref for SimulationRunData {
    type Target = Vec<EncounterResult>;
    fn deref(&self) -> &Self::Target {
        &self.encounters
    }
}

impl std::ops::DerefMut for SimulationRunData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encounters
    }
}

pub type SimulationResult = SimulationRunData;

/// A complete simulation run with both results and events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRun {
    pub result: SimulationResult,
    pub events: Vec<crate::events::Event>,
}

/// A lightweight representation of a simulation run for Two-Pass analysis
/// Contains only the data needed to identify interesting runs for re-simulation
/// Memory: ~32 bytes per run vs ~6-50 KB for full SimulationRun
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightRun {
    /// The RNG seed used for this run - allows exact re-simulation
    pub seed: u64,
    /// Cumulative score after each encounter (e.g., [50.0, 85.0] means scored 50 after E1, 85 after E2)
    pub encounter_scores: Vec<f64>,
    /// Final cumulative score across all encounters
    pub final_score: f64,
    /// Total HP lost by the party across all encounters
    pub total_hp_lost: f64,
    /// Number of party members still alive at the end
    pub total_survivors: usize,
    /// Whether any combatant died during this run (for TPK detection)
    pub has_death: bool,
    /// Which encounter the first death occurred in (if has_death is true)
    pub first_death_encounter: Option<usize>,
}

/// Tier classification for selected seeds in Three-Tier Phase 3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterestingSeedTier {
    /// Tier A: Full event logs (11 decile seeds) - ~200 KB per run
    /// Used for: BattleCard logs, full playback visualization
    TierA,
    /// Tier B: Lean event logs (100 1% median seeds) - ~10-30 KB per run
    /// Used for: 1% percentile analysis, median per bucket
    TierB,
    /// Tier C: No events (59 per-encounter extremes) - already in LightweightRun
    /// Used for: Per-encounter analysis only
    TierC,
}

/// A selected seed with its tier classification and bucket label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedSeed {
    /// The RNG seed to re-simulate
    pub seed: u64,
    /// Which tier this seed belongs to (determines event collection level)
    pub tier: InterestingSeedTier,
    /// Human-readable label for display (e.g., "P45-46", "P5", "E3-P25")
    pub bucket_label: String,
}

/// Lean run summary for Tier B event collection (1% medians)
/// Stores aggregate statistics instead of per-attack events
/// Memory: ~10-30 KB per run vs ~200-500 KB for full event logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanRunLog {
    /// The RNG seed used for this run
    pub seed: u64,
    /// Final cumulative score across all encounters
    pub final_score: f64,
    /// Cumulative score after each encounter
    pub encounter_scores: Vec<f64>,
    /// Per-round aggregate summaries (NOT per-attack events)
    pub round_summaries: Vec<LeanRoundSummary>,
    /// Key death events only (typically 0-5 per run)
    pub deaths: Vec<LeanDeathEvent>,
    /// Which encounter had a Total Party Kill (if any)
    pub tpk_encounter: Option<usize>,
    /// Final HP for each combatant for quick display
    pub final_hp: HashMap<String, u32>,
    /// Combatant IDs that survived to the end
    pub survivors: Vec<String>,
}

/// Per-round aggregate summary for lean event collection
/// Contains aggregated statistics instead of individual attack events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanRoundSummary {
    /// Round number within the encounter (1-indexed)
    pub round_number: u32,
    /// Which encounter this round belongs to
    pub encounter_index: usize,
    /// Total damage dealt by each combatant (aggregated across all attacks)
    pub total_damage: HashMap<String, f64>,
    /// Total healing applied by each combatant (aggregated)
    pub total_healing: HashMap<String, f64>,
    /// Combatant IDs that died during this round
    pub deaths_this_round: Vec<String>,
    /// Combatant IDs that survived this round
    pub survivors_this_round: Vec<String>,
}

/// Lean death event tracking - records only death, not the attack that caused it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanDeathEvent {
    /// ID of the combatant who died
    pub combatant_id: String,
    /// Round number when death occurred
    pub round: u32,
    /// Which encounter the death occurred in
    pub encounter_index: usize,
    /// Whether this was a player (true) or monster (false)
    pub was_player: bool,
}

/// Aggregated statistics from multiple simulation runs
/// This is O(1) in memory regardless of iteration count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub total_iterations: usize,
    pub successful_iterations: usize,
    pub aggregated_encounters: Vec<EncounterResult>,
    pub score_percentiles: ScorePercentiles,
    #[serde(default)]
    pub sample_runs: Vec<SimulationRun>, // Small sample for debugging (e.g., first/last/best/worst)
}

/// Percentile scores across all iterations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorePercentiles {
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub mean: f64,
    pub std_dev: f64,
}

impl Default for ScorePercentiles {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 0.0,
            median: 0.0,
            p25: 0.0,
            p75: 0.0,
            mean: 0.0,
            std_dev: 0.0,
        }
    }
}

/// A single simulation job in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSimulationJob {
    pub id: String,
    pub players: Vec<Creature>,
    pub timeline: Vec<TimelineStep>,
    pub iterations: usize,
    pub seed: Option<u64>,
}

/// A request to run a batch of simulations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSimulationRequest {
    pub jobs: Vec<BatchSimulationJob>,
}

/// The result of a single simulation job in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSimulationResult {
    pub id: String,
    pub summary: SimulationSummary,
}

/// The response for a batch simulation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSimulationResponse {
    pub results: Vec<BatchSimulationResult>,
}

#[cfg(test)]
mod encounter_tier_tests {
    use super::*;

    fn create_metrics(
        deaths_p1: usize,
        deaths_p50: usize,
        deaths_p99: usize,
        drain: f64,
        party_size: usize,
    ) -> EncounterMetrics {
        EncounterMetrics {
            deaths_p1,
            deaths_p50,
            deaths_p99,
            resource_drain_percent: drain,
            party_size,
        }
    }

    #[test]
    fn test_safe_encounter_classification() {
        let metrics = create_metrics(0, 0, 0, 20.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Safe);

        let metrics = create_metrics(1, 0, 0, 25.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Safe);

        let metrics = create_metrics(1, 0, 0, 15.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Safe);
    }

    #[test]
    fn test_challenging_encounter_classification() {
        let metrics = create_metrics(2, 1, 1, 40.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Challenging);

        let metrics = create_metrics(1, 0, 0, 35.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Challenging);

        let metrics = create_metrics(2, 1, 1, 45.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Challenging);
    }

    #[test]
    fn test_boss_encounter_classification() {
        let metrics = create_metrics(3, 2, 1, 60.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);

        // TPK - all 4 dead in P1, should be Failed not Boss
        let metrics = create_metrics(4, 3, 2, 70.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        let metrics = create_metrics(2, 1, 0, 55.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);
    }

    #[test]
    fn test_tpk_detection() {
        // TPK - all 4 party members dead in worst 1%
        let metrics = create_metrics(4, 3, 2, 75.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        // TPK - all 4 party members dead
        let metrics = create_metrics(4, 4, 4, 90.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        // TPK for 5 person party
        let metrics = create_metrics(5, 4, 3, 80.0, 5);
        assert_eq!(metrics.classify(), EncounterTier::Failed);
    }

    #[test]
    fn test_failed_outside_thresholds() {
        // Too many deaths for Safe tier
        let metrics = create_metrics(2, 0, 0, 25.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        // Resource drain too low for Safe is now Trivial (not Failed)
        let metrics = create_metrics(0, 0, 0, 5.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Trivial);

        // Resource drain too high for Boss
        let metrics = create_metrics(3, 2, 1, 95.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        // Deaths exceed Boss threshold
        let metrics = create_metrics(5, 3, 2, 60.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);
    }

    #[test]
    fn test_drain_boundary_conditions() {
        // Safe: exactly 30% drain (upper boundary) - Safe checked first
        let metrics = create_metrics(1, 0, 0, 30.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Safe);

        // Safe: exactly 10% drain (lower boundary)
        let metrics = create_metrics(1, 0, 0, 10.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Safe);

        // Challenging: exactly 50% drain (upper boundary) - Challenging checked before Boss
        let metrics = create_metrics(2, 1, 1, 50.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Challenging);

        // Challenging: exactly 30% drain - but Safe criteria not met (p99 > 0)
        let metrics = create_metrics(2, 1, 1, 30.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Challenging);

        // Boss: exactly 80% drain (upper boundary)
        let metrics = create_metrics(3, 3, 2, 80.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);

        // Boss: exactly 50% drain - p50=2 means Boss, not Challenging
        let metrics = create_metrics(3, 2, 1, 50.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);
    }

    #[test]
    fn test_p50_boss_range() {
        // Boss: P50 = 1 (lower bound)
        let metrics = create_metrics(3, 1, 1, 65.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);

        // Boss: P50 = 3 (upper bound) - but TPK takes precedence
        let metrics = create_metrics(4, 3, 2, 65.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Failed);

        // Boss: P50 = 2 (middle)
        let metrics = create_metrics(3, 2, 1, 65.0, 4);
        assert_eq!(metrics.classify(), EncounterTier::Boss);
    }
}

#[cfg(test)]
mod contextual_difficulty_tests {
    use super::*;

    #[test]
    fn test_no_resource_impact_90_percent() {
        // 95% resources - no adjustment
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            1,
            95.0,
            EncounterTier::Safe,
            10.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Safe);
        assert_eq!(contextual.contextual_tier, EncounterTier::Safe);
    }

    #[test]
    fn test_one_tier_harder_75_percent() {
        // 75% resources - one tier harder (now in 70-84% range)
        // Safe → Challenging
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            2,
            75.0,
            EncounterTier::Safe,
            25.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Safe);
        assert_eq!(contextual.contextual_tier, EncounterTier::Challenging);

        // Challenging → Boss
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            3,
            72.0,
            EncounterTier::Challenging,
            28.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Challenging);
        assert_eq!(contextual.contextual_tier, EncounterTier::Boss);

        // Boss → Failed
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            4,
            70.0,
            EncounterTier::Boss,
            30.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Boss);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);
    }

    #[test]
    fn test_two_tiers_harder_45_percent() {
        // 45% resources - two tiers harder (40-69% range)
        // Safe → Boss
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            5,
            45.0,
            EncounterTier::Safe,
            55.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Safe);
        assert_eq!(contextual.contextual_tier, EncounterTier::Boss);

        // Challenging → Failed
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            6,
            50.0,
            EncounterTier::Challenging,
            50.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Challenging);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);

        // Boss → Failed
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            7,
            40.0,
            EncounterTier::Boss,
            60.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Boss);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);
    }

    #[test]
    fn test_three_tiers_harder_under_40_percent() {
        // <40% resources - everything becomes Failed
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            8,
            35.0,
            EncounterTier::Safe,
            65.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Safe);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);

        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            8,
            20.0,
            EncounterTier::Challenging,
            80.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Challenging);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);

        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            8,
            5.0,
            EncounterTier::Boss,
            95.0,
            100.0,
            4,
        );
        assert_eq!(contextual.isolated_tier, EncounterTier::Boss);
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);
    }

    #[test]
    fn test_failed_remains_failed() {
        // Failed stays Failed regardless of resources
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            1,
            100.0,
            EncounterTier::Failed,
            0.0,
            100.0,
            4,
        );
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);

        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            5,
            50.0,
            EncounterTier::Failed,
            50.0,
            100.0,
            4,
        );
        assert_eq!(contextual.contextual_tier, EncounterTier::Failed);
    }

    #[test]
    fn test_boundary_conditions() {
        // Exactly 85% - no adjustment (upper boundary of no-impact zone)
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            1,
            85.0,
            EncounterTier::Safe,
            15.0,
            100.0,
            4,
        );
        assert_eq!(contextual.contextual_tier, EncounterTier::Safe);

        // Exactly 70% - one tier harder (lower boundary of 70-84% zone)
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            3,
            70.0,
            EncounterTier::Safe,
            30.0,
            100.0,
            4,
        );
        assert_eq!(contextual.contextual_tier, EncounterTier::Challenging);

        // Exactly 40% - two tiers harder (lower boundary of 40-69% zone)
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            5,
            40.0,
            EncounterTier::Safe,
            60.0,
            100.0,
            4,
        );
        assert_eq!(contextual.contextual_tier, EncounterTier::Boss);
    }

    #[test]
    fn test_difficulty_shift_description() {
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            3,
            45.0,
            EncounterTier::Safe,
            55.0,
            100.0,
            4,
        );
        let desc = contextual.difficulty_shift_description();
        assert!(desc.contains("Encounter #3"));
        assert!(desc.contains("Safe → Boss"));
        assert!(desc.contains("45%"));
    }

    #[test]
    fn test_no_shift_description() {
        let contextual = ContextualEncounterMetrics::from_isolated_metrics(
            1,
            95.0,
            EncounterTier::Safe,
            5.0,
            100.0,
            4,
        );
        let desc = contextual.difficulty_shift_description();
        assert!(desc.contains("Encounter #1"));
        assert!(desc.contains("Safe"));
        assert!(desc.contains("No resource impact"));
        assert!(desc.contains("95%"));
    }

    #[test]
    fn test_adventuring_day_progression() {
        // Simulate an adventuring day with 6 Safe encounters
        // Each drains ~15% resources

        // Encounter #1: 100% resources → Safe
        let e1 = ContextualEncounterMetrics::from_isolated_metrics(
            1,
            100.0,
            EncounterTier::Safe,
            0.0,
            100.0,
            4,
        );
        assert_eq!(e1.contextual_tier, EncounterTier::Safe);

        // Encounter #2: 85% resources → Safe
        let e2 = ContextualEncounterMetrics::from_isolated_metrics(
            2,
            85.0,
            EncounterTier::Safe,
            15.0,
            100.0,
            4,
        );
        assert_eq!(e2.contextual_tier, EncounterTier::Safe);

        // Encounter #3: 70% resources → Challenging
        let e3 = ContextualEncounterMetrics::from_isolated_metrics(
            3,
            70.0,
            EncounterTier::Safe,
            30.0,
            100.0,
            4,
        );
        assert_eq!(e3.contextual_tier, EncounterTier::Challenging);

        // Encounter #4: 55% resources → Boss
        let e4 = ContextualEncounterMetrics::from_isolated_metrics(
            4,
            55.0,
            EncounterTier::Safe,
            45.0,
            100.0,
            4,
        );
        assert_eq!(e4.contextual_tier, EncounterTier::Boss);

        // Encounter #5: 40% resources → Boss (two tiers from Safe)
        let e5 = ContextualEncounterMetrics::from_isolated_metrics(
            5,
            40.0,
            EncounterTier::Safe,
            60.0,
            100.0,
            4,
        );
        assert_eq!(e5.contextual_tier, EncounterTier::Boss);

        // Encounter #6: 25% resources → Failed (three tiers from Safe)
        let e6 = ContextualEncounterMetrics::from_isolated_metrics(
            6,
            25.0,
            EncounterTier::Safe,
            75.0,
            100.0,
            4,
        );
        assert_eq!(e6.contextual_tier, EncounterTier::Failed);
    }
}

