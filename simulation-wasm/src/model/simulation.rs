use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::creature::Combattant;
use super::encounter::TargetRole;

// Re-export API DTOs for backward compatibility
pub use crate::api::dto::{
    SimulationResult, SimulationRunData, SimulationRun,
    LightweightRun, SelectedSeed, InterestingSeedTier,
    LeanRunLog, LeanRoundSummary, LeanDeathEvent,
    SimulationSummary, ScorePercentiles,
    BatchSimulationJob, BatchSimulationRequest, BatchSimulationResult, BatchSimulationResponse,
};

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
