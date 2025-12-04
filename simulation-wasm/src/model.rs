use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::*;
use crate::resources::{ActionCost, ActionRequirement, ActionTag};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiceFormula {
    Value(f64),
    Expr(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiderEffect {
    pub dc: f64,
    pub buff: Buff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOptions {
    #[serde(rename = "templateName")]
    pub template_name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub action: Action, // The action to execute when triggered
    pub cost: Option<i32>, // e.g. Reaction (4)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupInstruction {
    RemoveAllBuffsFromSource(String), // Combatant ID of the source that died
    BreakConcentration(String, String), // (Combatant ID of concentrator, Buff ID)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: String,
    pub arrival: Option<i32>,
    #[serde(default)]
    pub mode: String, // "player", "monster", "custom"
    pub name: String,
    pub count: f64, // TS uses number, but usually integer.
    pub hp: f64,
    #[serde(rename = "AC")]
    pub ac: f64,
    #[serde(rename = "speed_fly")]
    pub speed_fly: Option<f64>,
    #[serde(rename = "saveBonus")]
    pub save_bonus: f64,
    #[serde(default)]
    #[serde(rename = "initiativeBonus")]
    pub initiative_bonus: f64,
    #[serde(default)]
    #[serde(rename = "initiativeAdvantage")]
    pub initiative_advantage: bool,
    #[serde(default)]
    #[serde(rename = "conSaveBonus")]
    pub con_save_bonus: Option<f64>,
    pub actions: Vec<Action>, // This might need to be flexible if templates are involved
    #[serde(default)]
    pub triggers: Vec<ActionTrigger>,
    #[serde(rename = "spellSlots")]
    pub spell_slots: Option<HashMap<String, i32>>,
    #[serde(rename = "classResources")]
    pub class_resources: Option<HashMap<String, i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureState {
    #[serde(rename = "currentHP")]
    pub current_hp: f64,
    #[serde(rename = "tempHP")]
    pub temp_hp: Option<f64>,
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
}

fn default_serializable_resource_ledger() -> SerializableResourceLedger {
    SerializableResourceLedger::from(crate::resources::ResourceLedger::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableResourceLedger {
    pub current: HashMap<String, f64>,
    pub max: HashMap<String, f64>,
}

impl From<crate::resources::ResourceLedger> for SerializableResourceLedger {
    fn from(ledger: crate::resources::ResourceLedger) -> Self {
        let current = ledger.current.into_iter()
            .map(|(k, v)| (format!("{:?}", k), v))
            .collect();
            
        let max = ledger.max.into_iter()
            .map(|(k, v)| (format!("{:?}", k), v))
            .collect();
            
        SerializableResourceLedger {
            current,
            max,
        }
    }
}

// We also need a way to convert back if needed, or just use it for output
// For now, it's mostly for output to frontend.

impl Default for CreatureState {
    fn default() -> Self {
        CreatureState {
            current_hp: 0.0,
            temp_hp: None,
            buffs: HashMap::new(),
            resources: default_serializable_resource_ledger(),
            upcoming_buffs: HashMap::new(),
            used_actions: HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: HashSet::new(),
            bonus_action_used: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Combattant {
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombattantAction {
    pub action: Action, // Should be FinalAction
    pub targets: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub monsters: Vec<Creature>,
    #[serde(rename = "playersSurprised")]
    pub players_surprised: Option<bool>,
    #[serde(rename = "monstersSurprised")]
    pub monsters_surprised: Option<bool>,
    #[serde(rename = "shortRest")]
    pub short_rest: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    pub team1: Vec<Combattant>,
    pub team2: Vec<Combattant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterResult {
    pub stats: HashMap<String, EncounterStats>,
    pub rounds: Vec<Round>,
}

pub type SimulationResult = Vec<EncounterResult>;
