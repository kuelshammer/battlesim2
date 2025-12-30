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
