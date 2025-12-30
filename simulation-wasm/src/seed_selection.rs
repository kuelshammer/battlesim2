//! Seed selection algorithms for Two-Pass deterministic re-simulation
//!
//! This module provides algorithms for identifying interesting seeds from
//! lightweight simulation runs for re-simulation with full event logging.

use crate::model::{LightweightRun, SelectedSeed, InterestingSeedTier};
use std::collections::HashSet;

/// Selects interesting seeds with 1% granularity and three-tier classification
///
/// This function analyzes lightweight runs from Phase 1 and selects seeds
/// for Phase 3 re-simulation based on:
///
/// **1% Bucket Medians (Tier B - Lean Events):**
/// - Divides results into 100 equal buckets
/// - Selects median from each 1% percentile (P0-1, P1-2, ..., P99-100)
/// - Enables true 1% granularity analysis
///
/// **Global Deciles (Tier A - Full Events):**
/// - Selects P5, P15, P25, P35, P45, P50, P55, P65, P75, P85, P95
/// - These get full event logs for BattleCard playback
///
/// **Per-Encounter Extremes (Tier C - No Events):**
/// - Selects P0, P50, P100 for each encounter
/// - Used for encounter-specific analysis
///
/// **Death Runs (Tier B):**
/// - Includes all runs where a combatant died
/// - Important for TPK analysis
///
/// # Arguments
/// * `lightweight_runs` - Results from Phase 1 lightweight survey pass
///
/// # Returns
/// Vector of `SelectedSeed` objects with tier classification and bucket labels
pub fn select_interesting_seeds_with_tiers(
    lightweight_runs: &[LightweightRun],
) -> Vec<SelectedSeed> {
    let mut selected_seeds = Vec::new();
    let mut seen_seeds = HashSet::new();

    let num_encounters = lightweight_runs
        .first()
        .map(|r| r.encounter_scores.len())
        .unwrap_or(0);

    let total_runs = lightweight_runs.len();

    // Sort all runs by final score once
    let mut global_scored_runs: Vec<(usize, f64)> = lightweight_runs
        .iter()
        .enumerate()
        .map(|(i, run)| (i, run.final_score))
        .collect();

    global_scored_runs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Helper closure to add seed if not already seen
    let add_seed = |seed: u64, tier: InterestingSeedTier, label: String, selected_seeds: &mut Vec<SelectedSeed>, seen_seeds: &mut HashSet<u64>| {
        if !seen_seeds.contains(&seed) {
            selected_seeds.push(SelectedSeed {
                seed,
                tier,
                bucket_label: label,
            });
            seen_seeds.insert(seed);
        }
    };

    // 1. GLOBAL 1% BUCKETS (100 medians → Tier B - Lean Events)
    // Divide into 100 equal buckets and select median from each
    if total_runs >= 100 {
        let bucket_size = total_runs / 100;
        for i in 0..100 {
            // Select median from each bucket
            let median_idx = i * bucket_size + bucket_size / 2;
            if let Some((run_idx, _)) = global_scored_runs.get(median_idx) {
                let run = &lightweight_runs[*run_idx];
                add_seed(
                    run.seed,
                    InterestingSeedTier::TierB,  // Lean events for 1% medians
                    format!("P{}-{}", i, i + 1),
                    &mut selected_seeds,
                    &mut seen_seeds
                );
            }
        }
    }

    // 2. GLOBAL DECILES (11 seeds → Tier A - Full Events for decile logs)
    // P5, P15, P25, P35, P45, P50, P55, P65, P75, P85, P95
    let decile_percentiles = [5, 15, 25, 35, 45, 50, 55, 65, 75, 85, 95];
    for &percentile in &decile_percentiles {
        let idx = (total_runs * percentile) / 100;
        if let Some((run_idx, _)) = global_scored_runs.get(idx) {
            let run = &lightweight_runs[*run_idx];
            add_seed(
                run.seed,
                InterestingSeedTier::TierA,  // Full events for decile logs
                format!("P{}", percentile),
                &mut selected_seeds,
                &mut seen_seeds
            );
        }
    }

    // 3. PER-ENCOUNTER EXTREMES (Tier C - No events, just lightweight data)
    // Select P0, P50, P100 for each encounter
    for encounter_idx in 0..num_encounters {
        let mut encounter_scored: Vec<(usize, f64)> = lightweight_runs
            .iter()
            .enumerate()
            .filter_map(|(i, run)| {
                run.encounter_scores.get(encounter_idx)
                    .copied()
                    .map(|score| (i, score))
            })
            .collect();

        encounter_scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let len = encounter_scored.len();
        if len == 0 { continue; }

        // Select P0, P50, P100 for this encounter
        for &percentile in &[0, 50, 100] {
            let idx = (len * percentile) / 100;
            if let Some((run_idx, _)) = encounter_scored.get(idx) {
                let run = &lightweight_runs[*run_idx];
                add_seed(
                    run.seed,
                    InterestingSeedTier::TierC,  // No events, use lightweight data
                    format!("E{}-P{}", encounter_idx, percentile),
                    &mut selected_seeds,
                    &mut seen_seeds
                );
            }
        }
    }

    // 4. Include all runs with deaths as Tier B (important for TPK analysis)
    for run in lightweight_runs {
        if run.has_death {
            add_seed(
                run.seed,
                InterestingSeedTier::TierB,  // Lean events to track deaths
                format!("DEATH-E{}", run.first_death_encounter.unwrap_or(0)),
                &mut selected_seeds,
                &mut seen_seeds
            );
        }
    }

    selected_seeds
}
