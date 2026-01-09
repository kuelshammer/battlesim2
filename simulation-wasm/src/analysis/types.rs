use serde::{Deserialize, Serialize};

/// Archetype classification for encounters based on lethality and attrition patterns
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EncounterArchetype {
    Trivial,
    Skirmish,
    Standard,
    TheGrind,
    EliteChallenge,
    BossFight,
    MeatGrinder,
    NovaTrap,
    Broken,
    CoinFlip,
}

impl std::fmt::Display for EncounterArchetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncounterArchetype::Trivial => write!(f, "Trivial"),
            EncounterArchetype::Skirmish => write!(f, "Skirmish"),
            EncounterArchetype::Standard => write!(f, "Standard"),
            EncounterArchetype::TheGrind => write!(f, "The Grind"),
            EncounterArchetype::EliteChallenge => write!(f, "Elite Challenge"),
            EncounterArchetype::BossFight => write!(f, "Boss Fight"),
            EncounterArchetype::MeatGrinder => write!(f, "Meat Grinder"),
            EncounterArchetype::NovaTrap => write!(f, "Nova Trap"),
            EncounterArchetype::Broken => write!(f, "Broken"),
            EncounterArchetype::CoinFlip => write!(f, "Coin Flip"),
        }
    }
}

/// Intensity tier for encounter difficulty
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IntensityTier {
    Tier1, // Trivial
    Tier2, // Light
    Tier3, // Moderate
    Tier4, // Heavy
    Tier5, // Extreme
}

impl std::fmt::Display for IntensityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntensityTier::Tier1 => write!(f, "Tier 1 (Trivial)"),
            IntensityTier::Tier2 => write!(f, "Tier 2 (Light)"),
            IntensityTier::Tier3 => write!(f, "Tier 3 (Moderate)"),
            IntensityTier::Tier4 => write!(f, "Tier 4 (Heavy)"),
            IntensityTier::Tier5 => write!(f, "Tier 5 (Extreme)"),
        }
    }
}

/// Human-readable encounter labels
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EncounterLabel {
    EpicChallenge,
    TacticalGrinder,
    ActionMovie,
    TheTrap,
    TheSlog,
    Standard,
    TrivialMinions,
    TPKRisk,
    Broken,
}

impl std::fmt::Display for EncounterLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncounterLabel::EpicChallenge => write!(f, "The Epic Challenge"),
            EncounterLabel::TacticalGrinder => write!(f, "The Tactical Grinder"),
            EncounterLabel::ActionMovie => write!(f, "The Action Movie"),
            EncounterLabel::TheTrap => write!(f, "The Trap"),
            EncounterLabel::TheSlog => write!(f, "The Slog"),
            EncounterLabel::Standard => write!(f, "Standard Encounter"),
            EncounterLabel::TrivialMinions => write!(f, "Trivial / Minions"),
            EncounterLabel::TPKRisk => write!(f, "TPK Risk"),
            EncounterLabel::Broken => write!(f, "Broken / Impossible"),
        }
    }
}

/// Combatant state for UI visualization
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CombatantVisualization {
    pub name: String,
    pub max_hp: u32,
    pub start_hp: u32,
    pub current_hp: u32,
    pub is_dead: bool,
    pub is_player: bool,
    pub hp_percentage: f64,
}

/// Statistics for a decile (percentile bucket)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecileStats {
    pub decile: usize,
    pub label: String,
    pub median_survivors: usize,
    pub party_size: usize,
    pub total_hp_lost: f64,
    pub hp_lost_percent: f64,
    pub win_rate: f64,
    pub median_run_visualization: Vec<CombatantVisualization>,
    pub median_run_data: Option<crate::model::EncounterResult>,
    pub battle_duration_rounds: usize,
    pub resource_timeline: Vec<f64>, // Array of EHP % after each step
    pub vitality_timeline: Vec<f64>, // Array of Vitality % after each step
    pub power_timeline: Vec<f64>, // Array of Power % after each step
}

/// Percentile range for timelines
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimelineRange {
    pub p25: Vec<f64>,
    pub p75: Vec<f64>,
}

/// Vital metrics for encounter analysis
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vitals {
    pub lethality_index: f64, // Probability of 1+ death/KO (0.0 - 1.0)
    pub tpk_risk: f64,        // Probability of TPK (0.0 - 1.0)
    pub attrition_score: f64, // % of daily budget burned (0.0 - 1.0)
    pub volatility_index: f64, // Difference between P10 and P50 cost
    pub doom_horizon: f64,    // Projected encounters until failure
    pub deaths_door_index: f64, // Average rounds spent at <25% HP (Thrilling metric)
    pub archetype: EncounterArchetype,
    pub is_volatile: bool,
}

/// Pacing analysis for multi-encounter days
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayPacing {
    pub archetype: String,
    pub director_score: f64,
    pub rhythm_score: f64,
    pub attrition_score: f64,
    pub recovery_score: f64,
}

/// Main output structure for analysis
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggregateOutput {
    pub scenario_name: String,
    pub total_runs: usize,
    pub deciles: Vec<DecileStats>,
    pub global_median: Option<DecileStats>,
    pub vitality_range: Option<TimelineRange>,
    pub power_range: Option<TimelineRange>,
    pub decile_logs: Vec<Vec<crate::events::Event>>, // 11 logs: [P5, P15, ..., P50, ..., P95]
    pub battle_duration_rounds: usize,
    pub intensity_tier: IntensityTier,
    pub encounter_label: EncounterLabel,
    pub analysis_summary: String,
    pub tuning_suggestions: Vec<String>,
    pub is_good_design: bool,
    pub stars: usize,
    pub tdnw: f64, // Total Daily Net Worth
    pub num_encounters: usize,
    pub skyline: Option<crate::percentile_analysis::SkylineAnalysis>,
    pub vitals: Option<Vitals>,
    pub pacing: Option<DayPacing>,
}
