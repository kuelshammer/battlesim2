use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::creature::Combattant;
use super::encounter::{TargetRole, TimelineStep};
use crate::events::Event;

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
    pub events: Vec<Event>,
}

/// A lightweight representation of a simulation run for Two-Pass analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightRun {
    pub seed: u64,
    pub encounter_scores: Vec<f64>,
    pub final_score: f64,
    pub total_hp_lost: f64,
    pub total_survivors: usize,
    pub has_death: bool,
    pub first_death_encounter: Option<usize>,
}

/// Tier classification for selected seeds in Three-Tier Phase 3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InterestingSeedTier {
    TierA,
    TierB,
    TierC,
}

/// A selected seed with its tier classification and bucket label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedSeed {
    pub seed: u64,
    pub tier: InterestingSeedTier,
    pub bucket_label: String,
}

/// Lean run summary for Tier B event collection (1% medians)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanRunLog {
    pub seed: u64,
    pub final_score: f64,
    pub encounter_scores: Vec<f64>,
    pub round_summaries: Vec<LeanRoundSummary>,
    pub deaths: Vec<LeanDeathEvent>,
    pub tpk_encounter: Option<usize>,
    pub final_hp: HashMap<String, u32>,
    pub survivors: Vec<String>,
}

/// Per-round aggregate summary for lean event collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanRoundSummary {
    pub round_number: u32,
    pub encounter_index: usize,
    pub total_damage: HashMap<String, f64>,
    pub total_healing: HashMap<String, f64>,
    pub deaths_this_round: Vec<String>,
    pub survivors_this_round: Vec<String>,
}

/// Lean death event tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanDeathEvent {
    pub combatant_id: String,
    pub round: u32,
    pub encounter_index: usize,
    pub was_player: bool,
}

/// Aggregated statistics from multiple simulation runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSummary {
    pub total_iterations: usize,
    pub successful_iterations: usize,
    pub aggregated_encounters: Vec<EncounterResult>,
    pub score_percentiles: ScorePercentiles,
    #[serde(default)]
    pub sample_runs: Vec<SimulationRun>,
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
    pub players: Vec<super::creature::Creature>,
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
