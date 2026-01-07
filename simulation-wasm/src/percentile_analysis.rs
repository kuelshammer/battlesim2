//! 100-bucket (1% percentile) aggregation for Skyline Spectrogram UI
//!
//! This module provides functions to aggregate simulation results into 100 buckets
//! (one per percentile), with per-character HP and resource metrics for visualization.

use crate::model::*;
use crate::intensity_calculation::*;
use crate::aggregation::{calculate_score, calculate_encounter_score};
use crate::resources::{ResourceLedger, ResetType};
use serde::{Deserialize, Serialize};

/// Per-character bucket data for one percentile
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterBucketData {
    /// Character name
    pub name: String,
    /// Character ID
    pub id: String,
    /// Maximum HP (for percentage calculation)
    pub max_hp: u32,
    /// HP remaining as percentage (0-100)
    pub hp_percent: f64,
    /// Weighted resource sum as percentage (0-100)
    pub resource_percent: f64,
    /// Detailed resource breakdown (for tooltip)
    pub resource_breakdown: ResourceBreakdown,
    /// Round of death (null if alive)
    pub death_round: Option<usize>,
    /// Whether the character is dead in this bucket's median run
    pub is_dead: bool,
}

/// Detailed resource breakdown for tooltip display
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceBreakdown {
    /// Spell slots remaining per level
    pub spell_slots: Vec<SpellSlotLevel>,
    /// Short-rest features remaining
    pub short_rest_features: Vec<String>,
    /// Long-rest features remaining
    pub long_rest_features: Vec<String>,
    /// Hit dice remaining
    pub hit_dice: f64,
    /// Max hit dice
    pub hit_dice_max: f64,
    /// Total weighted EHP value (for debugging)
    pub total_ehp: i32,
    /// Max weighted EHP value (for percentage)
    pub max_ehp: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellSlotLevel {
    pub level: usize,
    pub remaining: f64,
    pub max: f64,
}

/// One percentile bucket containing median data for ~1% of runs
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PercentileBucket {
    /// Percentile (1-100, where 1 = worst outcomes, 100 = best)
    pub percentile: usize,
    /// Number of runs in this bucket
    pub run_count: usize,
    /// Per-character data
    pub characters: Vec<CharacterBucketData>,
    /// Total party HP percentage (for quick reference)
    pub party_hp_percent: f64,
    /// Total party resource percentage (for quick reference)
    pub party_resource_percent: f64,
    /// Number of deaths in this bucket's median run
    pub death_count: usize,
}

/// Complete 100-bucket analysis output for Skyline UI
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkylineAnalysis {
    /// 100 percentile buckets
    pub buckets: Vec<PercentileBucket>,
    /// Total number of runs analyzed
    pub total_runs: usize,
    /// Party size
    pub party_size: usize,
    /// Encounter index (if per-encounter analysis)
    pub encounter_index: Option<usize>,
}

impl SkylineAnalysis {
    /// Create a new SkylineAnalysis with 100 empty buckets
    fn new(party_size: usize, encounter_index: Option<usize>) -> Self {
        Self {
            buckets: (1..=100).map(|p| PercentileBucket {
                percentile: p,
                run_count: 0,
                characters: Vec::new(),
                party_hp_percent: 0.0,
                party_resource_percent: 0.0,
                death_count: 0,
            }).collect(),
            total_runs: 0,
            party_size,
            encounter_index,
        }
    }
}

/// Analyze simulation results and create 100 percentile buckets
///
/// # Arguments
/// * `results` - Sorted simulation results (worst to best)
/// * `party_size` - Number of player characters
/// * `encounter_index` - Optional encounter index for per-encounter analysis
///
/// # Returns
/// `SkylineAnalysis` with 100 buckets containing per-character data
pub fn run_skyline_analysis(
    results: &[SimulationResult],
    party_size: usize,
    encounter_index: Option<usize>,
) -> SkylineAnalysis {
    if results.is_empty() {
        return SkylineAnalysis::new(party_size, encounter_index);
    }

    let total_runs = results.len();
    let mut analysis = SkylineAnalysis::new(party_size, encounter_index);
    analysis.total_runs = total_runs;

    // Calculate bucket size (approximately 1% of runs per bucket)
    let bucket_size = (total_runs as f64 / 100.0).ceil() as usize;

    // Process each percentile bucket
    for percentile in 1..=100 {
        let start_idx = ((percentile - 1) * bucket_size).min(total_runs - 1);
        let end_idx = (percentile * bucket_size).min(total_runs);
        let bucket_runs = &results[start_idx..end_idx];

        if bucket_runs.is_empty() {
            continue;
        }

        // Find median run in this bucket
        let median_idx = bucket_runs.len() / 2;
        let median_run = &bucket_runs[median_idx];

        // Compute bucket data from median run
        let bucket = compute_bucket_from_median(
            median_run,
            percentile,
            bucket_runs.len(),
            encounter_index,
        );

        analysis.buckets[percentile - 1] = bucket;
    }

    analysis
}

/// Compute bucket data from a single median simulation result
fn compute_bucket_from_median(
    result: &SimulationResult,
    percentile: usize,
    run_count: usize,
    encounter_index: Option<usize>,
) -> PercentileBucket {
    // Determine which encounter(s) to analyze
    let encounter_slice = if let Some(idx) = encounter_index {
        if idx < result.encounters.len() {
            &result.encounters[idx..=idx]
        } else {
            &[]
        }
    } else {
        &result.encounters[..]
    };

    // Get final state from last encounter in slice
    let final_encounter = encounter_slice.last();
    let first_encounter = encounter_slice.first();

    let mut characters = Vec::new();
    let mut party_hp_sum = 0.0;
    let mut party_resource_sum = 0.0;
    let mut death_count = 0;

    if let (Some(first_enc), Some(last_enc)) = (first_encounter, final_encounter) {
        // Get initial state from first round of first encounter
        let initial_round = first_enc.rounds.first();
        let final_round = last_enc.rounds.last();

        if let (Some(init_round), Some(fin_round)) = (initial_round, final_round) {
            // Create ID -> initial resources map
            let mut initial_resources: std::collections::HashMap<String, _> = std::collections::HashMap::new();
            for c in &init_round.team1 {
                initial_resources.insert(c.id.clone(), c.initial_state.resources.clone());
            }

            // Process each player character
            for c in &fin_round.team1 {
                let initial_ledger = c.creature.initialize_ledger();
                let max_hp = c.creature.hp;
                let current_hp = c.final_state.current_hp;

                // Calculate HP percentage
                let hp_percent = if max_hp > 0 {
                    (current_hp as f64 / max_hp as f64) * 100.0
                } else {
                    0.0
                };

                // Calculate resource percentage using EHP points
                let current_ehp = calculate_serializable_ehp(
                    current_hp,
                    c.final_state.temp_hp.unwrap_or(0),
                    &c.final_state.resources,
                    &initial_ledger.reset_rules,
                );

                let max_ehp = calculate_ledger_max_ehp(&c.creature, &initial_ledger);
                let resource_percent = if max_ehp > 0.0 {
                    (current_ehp / max_ehp) * 100.0
                } else {
                    100.0
                };

                // Determine death round
                let (death_round, is_dead) = find_death_round(&encounter_slice.iter().collect::<Vec<_>>(), &c.id);

                // Extract resource breakdown
                let breakdown = extract_resource_breakdown(
                    &c.final_state.resources,
                    &initial_ledger,
                    max_hp,
                );

                party_hp_sum += hp_percent;
                party_resource_sum += resource_percent;
                if is_dead {
                    death_count += 1;
                }

                characters.push(CharacterBucketData {
                    name: c.creature.name.clone(),
                    id: c.id.clone(),
                    max_hp,
                    hp_percent,
                    resource_percent,
                    resource_breakdown: breakdown,
                    death_round,
                    is_dead,
                });
            }
        }
    }

    let char_count = characters.len() as f64;
    PercentileBucket {
        percentile,
        run_count,
        characters,
        party_hp_percent: if char_count > 0.0 { party_hp_sum / char_count } else { 0.0 },
        party_resource_percent: if char_count > 0.0 { party_resource_sum / char_count } else { 0.0 },
        death_count,
    }
}

/// Find the round in which a character died
fn find_death_round(
    encounters: &[&EncounterResult],
    character_id: &str,
) -> (Option<usize>, bool) {
    let mut global_round = 0;

    for enc in encounters {
        for (round_idx, round) in enc.rounds.iter().enumerate() {
            if let Some(c) = round.team1.iter().find(|c| c.id == character_id) {
                if c.final_state.current_hp == 0 {
                    return (Some(global_round + round_idx + 1), true);
                }
            }
        }
        global_round += enc.rounds.len();
    }

    (None, false)
}

/// Extract detailed resource breakdown for tooltip display
fn extract_resource_breakdown(
    resources: &SerializableResourceLedger,
    ledger: &ResourceLedger,
    max_hp: u32,
) -> ResourceBreakdown {
    let mut spell_slots: Vec<SpellSlotLevel> = Vec::new();
    let mut short_rest_features = Vec::new();
    let mut long_rest_features = Vec::new();
    let mut hit_dice = 0.0;
    let mut hit_dice_max = 0.0;

    // Process each resource
    for (key, &max_val) in &resources.max {
        let current_val = resources.current.get(key).cloned().unwrap_or(0.0);

        if key.starts_with("SpellSlot") {
            // Extract level from "SpellSlot(Level)"
            if let Some(level_str) = extract_level_from_key(key) {
                if let Ok(level) = level_str.parse::<usize>() {
                    // Find existing slot level entry or create new
                    if let Some(slot) = spell_slots.iter_mut().find(|s| s.level == level) {
                        slot.remaining += current_val;
                        slot.max += max_val;
                    } else {
                        spell_slots.push(SpellSlotLevel {
                            level,
                            remaining: current_val,
                            max: max_val,
                        });
                    }
                }
            }
        } else if key.starts_with("HitDice") {
            hit_dice += current_val;
            hit_dice_max += max_val;
        } else if key.starts_with("ClassResource") || key.starts_with("Custom") {
            // Categorize by reset rule
            if let Some(reset) = ledger.reset_rules.get(key) {
                let feature_name = key.replace("ClassResource(", "")
                    .replace("Custom(", "")
                    .replace(")", "")
                    .replace("_", " ");

                match reset {
                    ResetType::ShortRest => short_rest_features.push(format!("{}: {}/{}", feature_name, current_val as i32, max_val as i32)),
                    ResetType::LongRest => long_rest_features.push(format!("{}: {}/{}", feature_name, current_val as i32, max_val as i32)),
                    _ => {}
                }
            }
        }
    }

    // Sort spell slots by level
    spell_slots.sort_by_key(|s| s.level);

    // Calculate total EHP for debugging
    let total_ehp = calculate_ehp_points(
        0,
        0,
        &resources.current,
        &ledger.reset_rules,
    );
    let max_ehp = calculate_ehp_points(
        max_hp,
        0,
        &resources.max,
        &ledger.reset_rules,
    );

    ResourceBreakdown {
        spell_slots,
        short_rest_features,
        long_rest_features,
        hit_dice,
        hit_dice_max,
        total_ehp: total_ehp as i32,
        max_ehp: max_ehp as i32,
    }
}

/// Helper to extract level from resource key like "SpellSlot(3)"
fn extract_level_from_key(key: &str) -> Option<&str> {
    let start = key.find('(')? + 1;
    let end = key.find(')')?;
    if start < end {
        Some(&key[start..end])
    } else {
        None
    }
}

/// Run skyline analysis with pre-sorted results
///
/// This is the main entry point for WASM bindings. Results should already
/// be sorted by score (worst to best) before calling this function.
pub fn run_skyline_analysis_sorted(
    mut results: Vec<SimulationResult>,
    party_size: usize,
    encounter_index: Option<usize>,
) -> SkylineAnalysis {
    // Sort results by the appropriate score
    if let Some(idx) = encounter_index {
        // Sort by this specific encounter's score
        results.sort_by(|a, b| {
            let encounter_a = a.encounters.get(idx);
            let encounter_b = b.encounters.get(idx);

            match (encounter_a, encounter_b) {
                (Some(ea), Some(eb)) => {
                    let score_a = calculate_encounter_score(ea);
                    let score_b = calculate_encounter_score(eb);
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    } else {
        // Sort by overall score
        results.sort_by(|a, b| {
            let score_a = calculate_score(a);
            let score_b = calculate_score(b);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    run_skyline_analysis(&results, party_size, encounter_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skyline_analysis_empty() {
        let analysis = run_skyline_analysis(&[], 4, None);
        assert_eq!(analysis.total_runs, 0);
        assert_eq!(analysis.buckets.len(), 100);
    }

    #[test]
    fn test_skyline_analysis_single_run() {
        // This test would require creating mock SimulationResult data
        // For now, just test that the function doesn't panic
        let analysis = run_skyline_analysis(&[], 4, None);
        assert_eq!(analysis.buckets.len(), 100);
    }

    #[test]
    fn test_extract_level_from_key() {
        assert_eq!(extract_level_from_key("SpellSlot(1)"), Some("1"));
        assert_eq!(extract_level_from_key("SpellSlot(9)"), Some("9"));
        assert_eq!(extract_level_from_key("ClassResource"), None);
    }

    #[test]
    fn test_bucket_percentile_range() {
        let analysis = SkylineAnalysis::new(4, None);
        assert_eq!(analysis.buckets.len(), 100);
        assert_eq!(analysis.buckets[0].percentile, 1);
        assert_eq!(analysis.buckets[99].percentile, 100);
    }
}
