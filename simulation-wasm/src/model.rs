use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::*;

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
    // Template actions are resolved before simulation, so we might not need them here if we only receive "FinalAction"
    // But the Creature struct has "actions: z.array(ActionSchema)", which includes TemplateAction.
    // However, the simulation logic usually works with FinalAction.
    // Let's include Template for completeness if needed, but for now I'll stick to what's needed for simulation.
    // Actually, the input to simulation is Creature, which has ActionSchema.
    // But the simulation converts them to FinalAction. Ideally the frontend sends resolved actions?
    // Looking at simulation.ts: `combattant.creature.actions.map(getFinalAction)`.
    // `getFinalAction` is in `../data/actions`.
    // If I want to run this in Rust, I either need to port `getFinalAction` and the templates, OR ensure the frontend sends resolved actions.
    // The prompt says "mirror the TS interfaces".
    // Let's assume for now we might need to handle templates or the frontend will send resolved creatures.
    // Given the complexity of porting all templates, it would be better if the frontend resolved them.
    // BUT: The user wants to keep the frontend "exactly the same".
    // So I should probably handle what comes in.
    // However, `getFinalAction` seems to just look up templates.
    // Let's define the structs for the specific actions first.
    // ... (Action enum definition)
}

impl Action {
    pub fn base(&self) -> ActionBase {
        match self {
            Action::Atk(a) => a.base(),
            Action::Heal(a) => a.base(),
            Action::Buff(a) => a.base(),
            Action::Debuff(a) => a.base(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBase {
    pub id: String,
    pub name: String,
    #[serde(rename = "actionSlot")]
    pub action_slot: i32,
    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtkAction {
    pub id: String,
    pub name: String,
    #[serde(rename = "actionSlot")]
    pub action_slot: i32,
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
    #[serde(rename = "actionSlot")]
    pub action_slot: i32,
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
    #[serde(rename = "actionSlot")]
    pub action_slot: i32,
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
    #[serde(rename = "actionSlot")]
    pub action_slot: i32,
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
    pub cost: Option<ActionSlot>, // e.g. Reaction (4)
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureState {
    #[serde(rename = "currentHP")]
    pub current_hp: f64,
    #[serde(rename = "tempHP")]
    pub temp_hp: Option<f64>,
    pub buffs: HashMap<String, Buff>,
    #[serde(rename = "remainingUses")]
    pub remaining_uses: HashMap<String, f64>,
    #[serde(rename = "upcomingBuffs")]
    pub upcoming_buffs: HashMap<String, Buff>,
    #[serde(rename = "usedActions")]
    pub used_actions: HashSet<String>,
    #[serde(rename = "concentratingOn")]
    pub concentrating_on: Option<String>,
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
