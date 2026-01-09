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

    // NEW: Experience-based metrics
    pub near_death_survivors: f64, // Average characters ending at 1-10 HP (0.0 - party_size)
    pub crisis_participation_rate: f64, // % of party who hit <25% HP at some point (0.0 - 1.0)
    pub min_hp_threshold: f64, // Lowest HP % anyone reached during fight (0.0 - 1.0)
    pub avg_unconscious_rounds: f64, // Average rounds per player at 0 HP (agency loss measure)

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

/// Metrics calculated from a single simulation run
/// Replaces the 7-tuple return type with named, self-documenting fields
#[derive(Debug, Clone)]
pub struct RunMetrics {
    /// Resources burned (EHP consumed)
    pub burned: f64,
    /// Number of surviving combatants
    pub survivors: usize,
    /// Battle duration in rounds
    pub duration: usize,
    /// EHP percentage timeline (per encounter step)
    pub ehp_timeline: Vec<f64>,
    /// Vitality percentage timeline (per encounter step)
    pub vitality_timeline: Vec<f64>,
    /// Strategic power percentage timeline (per encounter step)
    pub power_timeline: Vec<f64>,
}

impl RunMetrics {
    /// Create empty/default metrics
    pub fn empty() -> Self {
        Self {
            burned: 0.0,
            survivors: 0,
            duration: 0,
            ehp_timeline: Vec::new(),
            vitality_timeline: Vec::new(),
            power_timeline: Vec::new(),
        }
    }
}

/// Game balance configuration constants
///
/// These thresholds control encounter classification and pacing assessment.
/// Extracted to enable A/B testing and tuning without code changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBalance {
    // === Archetype Assessment Thresholds ===

    /// TPK risk threshold for "Broken" archetype
    pub tpk_broken_threshold: f64,

    /// Volatility index threshold for detecting high volatility
    pub volatility_high_threshold: f64,

    /// Lethality index threshold for Coin Flip detection
    pub coin_flip_lethality_threshold: f64,

    /// TPK risk threshold for "Meat Grinder" archetype
    pub tpk_meat_grinder_threshold: f64,

    // Lethality thresholds for various archetypes
    pub lethality_boss_threshold: f64,
    pub lethality_elite_threshold: f64,
    pub lethality_standard_threshold: f64,
    pub lethality_skirmish_threshold: f64,

    // Attrition thresholds for archetype differentiation
    pub attrition_nova_trap_threshold: f64,
    pub attrition_grind_high_threshold: f64,
    pub attrition_grind_low_threshold: f64,
    pub attrition_standard_threshold: f64,
    pub attrition_skirmish_threshold: f64,

    // === Pacing/Attrition Scoring Thresholds ===

    /// Resource percentage considered "TPK/Total Exhaustion"
    pub pacing_exhaustion_pct: f64,

    /// Resource percentage considered "Tense but acceptable"
    pub pacing_tense_pct: f64,

    /// Resource percentage upper bound for "Sweet spot"
    pub pacing_sweet_spot_high_pct: f64,

    /// Resource percentage lower bound for "Sweet spot"
    pub pacing_sweet_spot_low_pct: f64,

    /// Resource percentage considered "A bit easy"
    pub pacing_easy_pct: f64,

    // === Rhythm/Difficulty Escalation ===

    /// Tolerance factor for detecting "dips" in difficulty (0.0-1.0)
    /// A dip is when weight < max_weight_so_far * dip_tolerance
    pub dip_tolerance: f64,

    /// Score penalty per "excess" dip (beyond the allowed breather)
    pub dip_penalty: f64,

    // === Volatility Detection ===

    /// Volatility index threshold for marking an encounter as "volatile"
    pub is_volatile_threshold: f64,

    // === Scoring Weights ===

    /// Weight for rhythm score in Director's Score calculation
    pub director_rhythm_weight: f64,

    /// Weight for attrition score in Director's Score calculation
    pub director_attrition_weight: f64,

    /// Weight for recovery score in Director's Score calculation
    pub director_recovery_weight: f64,

    // === Intensity Tier Thresholds ===

    /// Multipliers for intensity tier boundaries
    pub intensity_tier1_multiplier: f64,  // < 0.2 * target
    pub intensity_tier2_multiplier: f64,  // < 0.6 * target
    pub intensity_tier3_multiplier: f64,  // < 1.3 * target
    pub intensity_tier4_multiplier: f64,  // < 2.0 * target
}

impl Default for GameBalance {
    fn default() -> Self {
        Self {
            // Archetype thresholds - based on original code values
            tpk_broken_threshold: 0.5,
            volatility_high_threshold: 0.15,
            coin_flip_lethality_threshold: 0.05,
            tpk_meat_grinder_threshold: 0.1,
            lethality_boss_threshold: 0.5,
            lethality_elite_threshold: 0.3,
            lethality_standard_threshold: 0.15,
            lethality_skirmish_threshold: 0.05,
            attrition_nova_trap_threshold: 0.2,
            attrition_grind_high_threshold: 0.4,
            attrition_grind_low_threshold: 0.3,
            attrition_standard_threshold: 0.3,
            attrition_skirmish_threshold: 0.1,

            // Pacing thresholds
            pacing_exhaustion_pct: 0.0,
            pacing_tense_pct: 10.0,
            pacing_sweet_spot_high_pct: 35.0,
            pacing_sweet_spot_low_pct: 10.0,
            pacing_easy_pct: 60.0,

            // Rhythm thresholds
            dip_tolerance: 0.9,
            dip_penalty: 30.0,

            // Volatility detection
            is_volatile_threshold: 0.20,

            // Director's Score weights
            director_rhythm_weight: 0.4,
            director_attrition_weight: 0.4,
            director_recovery_weight: 0.2,

            // Intensity tier multipliers
            intensity_tier1_multiplier: 0.2,
            intensity_tier2_multiplier: 0.6,
            intensity_tier3_multiplier: 1.3,
            intensity_tier4_multiplier: 2.0,
        }
    }
}
