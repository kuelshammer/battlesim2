use crate::enums::*;
pub use crate::enums::ActionCondition; // Explicitly re-export ActionCondition
use crate::resources::{ActionCost, ActionRequirement, ActionTag, ResourceType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DiceFormula {
    Value(f64),
    Expr(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiderEffect {
    pub dc: f64,
    pub buff: Buff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcKnowledge {
    pub min: i32,
    pub max: i32,
}

impl Default for AcKnowledge {
    fn default() -> Self {
        Self { min: 0, max: 30 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub action: Action,    // The action to execute when triggered
    pub cost: Option<i32>, // e.g. Reaction (4)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
                // Parse level string "1", "2", etc.
                if let Ok(level) = level_str.parse::<u8>() {
                    let resource_type = ResourceType::SpellSlot; // Use resources::ResourceType
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Combattant {
    pub id: String,
    #[serde(default)]
    pub team: u32, // 0 for Team 1 (Players), 1 for Team 2 (Monsters)
    pub creature: Creature,
    #[serde(default)]
    pub initiative: f64,
    #[serde(rename = "initialState")]
    pub initial_state: CreatureState,
    #[serde(rename = "finalState")]
    pub final_state: CreatureState,
    // actions taken is complex in TS, simplified here for now or omitted if not needed for input
    // In TS: actions: { action: FinalAction, targets: Map<string, number> }[]
    // We probably don't need to deserialize this from input, but we might need it for internal state.
    // #[serde(skip)] - We need this for the results!
    pub actions: Vec<CombattantAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombattantAction {
    pub action: Action, // Should be FinalAction
    pub targets: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Encounter {
    pub monsters: Vec<Creature>,
    #[serde(rename = "playersSurprised")]
    pub players_surprised: Option<bool>,
    #[serde(rename = "monstersSurprised")]
    pub monsters_surprised: Option<bool>,
    #[serde(rename = "shortRest")]
    pub short_rest: Option<bool>,
    #[serde(rename = "playersPrecast")]
    pub players_precast: Option<bool>, // New field
    #[serde(rename = "monstersPrecast")]
    pub monsters_precast: Option<bool>, // New field
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationRunData {
    pub encounters: Vec<EncounterResult>,
    #[serde(default)]
    pub score: Option<f64>,
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
