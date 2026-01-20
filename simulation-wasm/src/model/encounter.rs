use super::creature::Creature;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

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

        if challenging_deaths_p99
            && challenging_deaths_p50
            && challenging_deaths_p1
            && challenging_drain
        {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Encounter {
    pub monsters: Vec<Creature>,
    #[serde(rename = "playersSurprised")]
    pub players_surprised: Option<bool>,
    #[serde(rename = "monstersSurprised")]
    pub monsters_surprised: Option<bool>,
    #[serde(rename = "playersPrecast")]
    pub players_precast: Option<bool>,
    #[serde(rename = "monstersPrecast")]
    pub monsters_precast: Option<bool>,
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
}
