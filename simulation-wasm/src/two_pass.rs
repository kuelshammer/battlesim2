//! Two-Pass Deterministic Re-simulation system
//!
//! This module implements the Two-Pass system for memory-efficient simulation:
//!
//! **Phase 1: Lightweight Survey Pass**
//! - Run all iterations with minimal memory overhead (~323 KB for 10,100 runs)
//! - Track only scores and deaths, no event collection
//! - Cache results for re-use
//!
//! **Phase 2: Seed Selection**
//! - Identify interesting seeds for re-simulation
//! - Use 1% granularity bucketing with three-tier classification
//! - Select ~170 seeds from 10,100 runs
//!
//! **Phase 3: Deep Dive Pass**
//! - Re-run only selected seeds with appropriate event collection
//! - Tier A: Full events for 11 decile logs (~2.2 MB)
//! - Tier B: Lean events for 100 1% medians (~2 MB)
//! - Tier C: No events, use lightweight data (~2 KB)
//!
//! Total memory: ~4.5 MB (vs ~15-20 MB for storing all runs)

use crate::model::{Creature, TimelineStep};

/// Two-Pass Deterministic Re-simulation implementation
///
/// Original Two-Pass system with 10% granularity and simple seed selection.
/// Memory efficient: ~5 MB total vs ~15-20 MB for storing all runs.
///
/// # Arguments
/// * `players` - Player creatures participating in the simulation
/// * `timeline` - Timeline of encounters and rest steps
/// * `iterations` - Number of iterations to run
/// * `_log_enabled` - Whether to enable logging (unused)
/// * `seed` - Optional base seed for deterministic results
///
/// # Returns
/// `SimulationSummary` with aggregated statistics and sample runs
pub fn run_simulation_with_rolling_stats(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> crate::model::SimulationSummary {
    let iterations = iterations.max(100);
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    // Phase 1: Survey pass - lightweight simulation for all iterations
    let base_seed = seed.unwrap_or(0);
    let lightweight_runs = crate::run_survey_pass(
        players.clone(),
        timeline.clone(),
        iterations,
        Some(base_seed),
    );

    // Phase 2: Identify interesting seeds for re-simulation
    let interesting_seeds: Vec<u64> =
        crate::seed_selection::select_interesting_seeds_with_tiers(&lightweight_runs)
            .iter()
            .map(|s| s.seed)
            .collect();

    // Phase 3: Deep dive pass - re-run interesting seeds with full events
    let mut sample_runs = Vec::new();
    for seed in &interesting_seeds {
        crate::rng::seed_rng(*seed);
        let (result, events) =
            crate::run_single_event_driven_simulation(&players, &timeline, false);
        sample_runs.push(crate::model::SimulationRun { result, events });
    }

    // Clear the seeded RNG after simulation completes
    crate::rng::clear_rng();

    // Calculate statistics from lightweight runs
    let mut sorted_scores: Vec<f64> =
        lightweight_runs.iter().map(|r| r.final_score).collect();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut score_sum = 0.0;
    let mut score_sum_squared = 0.0;
    let score_min = *sorted_scores.first().unwrap_or(&0.0);
    let score_max = *sorted_scores.last().unwrap_or(&0.0);

    for &score in &sorted_scores {
        score_sum += score;
        score_sum_squared += score * score;
    }

    let mean = if iterations > 0 {
        score_sum / iterations as f64
    } else {
        0.0
    };
    let variance = if iterations > 0 {
        (score_sum_squared / iterations as f64) - (mean * mean)
    } else {
        0.0
    };
    let std_dev = variance.sqrt().max(0.0);

    let median = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() / 2]
    } else {
        0.0
    };
    let p25 = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() / 4]
    } else {
        0.0
    };
    let p75 = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() * 3 / 4]
    } else {
        0.0
    };

    crate::model::SimulationSummary {
        total_iterations: iterations,
        successful_iterations: iterations, // All runs "succeed" in the sense they complete
        aggregated_encounters: Vec::new(),
        score_percentiles: crate::model::ScorePercentiles {
            min: score_min,
            max: score_max,
            median,
            p25,
            p75,
            mean,
            std_dev,
        },
        sample_runs,
    }
}

/// Three-Tier Two-Pass Deterministic Re-simulation with 1% granularity
///
/// Enhanced Two-Pass system with 1% granularity and three-tier event collection:
/// - Phase 1: Lightweight survey pass (10,100 iterations, ~323 KB memory)
/// - Phase 2: Identify interesting seeds with 1% bucket granularity and tier classification
/// - Phase 3: Re-run only interesting seeds with tier-appropriate event collection
///
/// Total memory: ~4.5 MB (vs ~15-20 MB for all full events)
///
/// # Arguments
/// * `players` - Player creatures participating in the simulation
/// * `timeline` - Timeline of encounters and rest steps
/// * `iterations` - Number of iterations to run (should be 10,100 for 1% granularity)
/// * `_log_enabled` - Whether to enable logging (unused)
/// * `seed` - Optional base seed for deterministic results
///
/// # Returns
/// `SimulationSummary` with aggregated statistics and tier-appropriate sample runs
pub fn run_simulation_with_three_tier(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> crate::model::SimulationSummary {
    let iterations = iterations.max(100);
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    // Phase 1: Survey pass - lightweight simulation for all iterations
    let base_seed = seed.unwrap_or(0);
    let lightweight_runs = crate::run_survey_pass(
        players.clone(),
        timeline.clone(),
        iterations,
        Some(base_seed),
    );

    // Phase 2: Identify interesting seeds with 1% granularity and tier classification
    let selected_seeds =
        crate::seed_selection::select_interesting_seeds_with_tiers(&lightweight_runs);

    // Phase 3: Deep dive pass - re-run selected seeds with tier-appropriate event collection
    let mut sample_runs = Vec::new();
    for selected_seed in &selected_seeds {
        crate::rng::seed_rng(selected_seed.seed);

        match selected_seed.tier {
            crate::model::InterestingSeedTier::TierA => {
                // Full events for decile logs
                let (result, events) =
                    crate::run_single_event_driven_simulation(&players, &timeline, false);
                sample_runs.push(crate::model::SimulationRun { result, events });
            }
            crate::model::InterestingSeedTier::TierB => {
                // Lean events for 1% medians
                // TODO: For now, we run full events but store fewer runs
                // In a future update, we'd use execute_encounter_lean() for true lean collection
                let (result, events) =
                    crate::run_single_event_driven_simulation(&players, &timeline, false);
                sample_runs.push(crate::model::SimulationRun { result, events });
            }
            crate::model::InterestingSeedTier::TierC => {
                // No events needed - use lightweight data only
                // Don't add to sample_runs since we already have the lightweight data
                continue;
            }
        }
    }

    // Clear the seeded RNG after simulation completes
    crate::rng::clear_rng();

    // Calculate statistics from lightweight runs
    let mut sorted_scores: Vec<f64> =
        lightweight_runs.iter().map(|r| r.final_score).collect();
    sorted_scores
        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut score_sum = 0.0;
    let mut score_sum_squared = 0.0;
    let score_min = *sorted_scores.first().unwrap_or(&0.0);
    let score_max = *sorted_scores.last().unwrap_or(&0.0);

    for &score in &sorted_scores {
        score_sum += score;
        score_sum_squared += score * score;
    }

    let mean = if iterations > 0 {
        score_sum / iterations as f64
    } else {
        0.0
    };
    let variance = if iterations > 0 {
        (score_sum_squared / iterations as f64) - (mean * mean)
    } else {
        0.0
    };
    let std_dev = variance.sqrt().max(0.0);

    let median = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() / 2]
    } else {
        0.0
    };
    let p25 = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() / 4]
    } else {
        0.0
    };
    let p75 = if !sorted_scores.is_empty() {
        sorted_scores[sorted_scores.len() * 3 / 4]
    } else {
        0.0
    };

    crate::model::SimulationSummary {
        total_iterations: iterations,
        successful_iterations: iterations,
        aggregated_encounters: Vec::new(),
        score_percentiles: crate::model::ScorePercentiles {
            min: score_min,
            max: score_max,
            median,
            p25,
            p75,
            mean,
            std_dev,
        },
        sample_runs,
    }
}
