use crate::context::CombattantState;
use crate::events::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of executing a single action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub actor_id: String,
    pub action_id: String,
    pub success: bool,
    pub events_generated: Vec<Event>,
    pub reactions_triggered: Vec<ReactionResult>,
    pub error: Option<String>,
}

/// Result of a reaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionResult {
    pub combatant_id: String,
    pub reaction_id: String,
    pub success: bool,
    pub events_generated: Vec<Event>,
    pub error: Option<String>,
}

/// Result of executing a complete turn for a combatant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub combatant_id: String,
    pub round_number: u32,
    pub action_results: Vec<ActionResult>,
    pub effects_applied: Vec<String>, // Effect IDs applied during this turn
    pub start_hp: u32,
    pub end_hp: u32,
}

/// Result of a complete encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterResult {
    pub winner: Option<String>,
    pub total_rounds: u32,
    pub total_turns: u32,
    pub final_combatant_states: Vec<CombattantState>,
    pub round_snapshots: Vec<Vec<CombattantState>>, // Snapshots of all combatants at end of each round
    pub event_history: Vec<Event>,
    pub statistics: EncounterStatistics,
}

/// Statistics collected during an encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterStatistics {
    pub total_damage_dealt: HashMap<String, f64>,
    pub total_healing_dealt: HashMap<String, f64>,
    pub attacks_landed: HashMap<String, u32>,
    pub attacks_missed: HashMap<String, u32>,
    pub reactions_triggered: u32,
    pub critical_hits: u32,
    pub total_actions_executed: u32,
}
