//! Simulation runners - core simulation execution logic
//!
//! Contains the multi-phase simulation strategy:
//! - Survey pass (lightweight, all iterations)
//! - Selection (identify interesting seeds)
//! - Deep dive (full events for selected runs)

use crate::aggregation::{calculate_cumulative_score, calculate_score};
use crate::model::{Creature, LightweightRun, SimulationResult, SimulationRun, TimelineStep};
use crate::utils::summarize_result;
use js_sys::Function;
use std::collections::HashMap;

/// Run event-driven simulation (Rust-native, for CLI/testing)
/// Returns all simulation runs with their results and events
pub fn run_event_driven_simulation_rust(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> Vec<SimulationRun> {
    let iterations = iterations.max(100);

    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    let mut all_runs = Vec::new();

    for i in 0..iterations {
        // If a seed is provided, use it with the iteration index for determinism
        if let Some(s) = seed {
            crate::rng::seed_rng(s.wrapping_add(i as u64));
        }

        let (result, events) = crate::run_single_event_driven_simulation(&players, &timeline, true);
        let run = SimulationRun { result, events };
        all_runs.push(run);
    }

    // Clear the seeded RNG after simulation completes
    if seed.is_some() {
        crate::rng::clear_rng();
    }

    // Sort results by score (worst to best)
    all_runs.sort_by(|a, b| {
        let score_a = calculate_score(&a.result);
        let score_b = calculate_score(&b.result);
        score_a
            .partial_cmp(&score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    all_runs
}

/// Three-phase simulation with progress callback support
///
/// Phase 1: Survey Pass - Run all iterations, collect lightweight results
/// Phase 2: Selection - Identify interesting seeds using 1% granularity
/// Phase 3: Deep Dive - Re-run interesting seeds with full event logging
pub struct ThreePhaseSimulation {
    pub players: Vec<Creature>,
    pub timeline: Vec<TimelineStep>,
    pub iterations: usize,
}

impl ThreePhaseSimulation {
    pub fn new(players: Vec<Creature>, timeline: Vec<TimelineStep>, iterations: usize) -> Self {
        Self {
            players,
            timeline,
            iterations: iterations.max(100),
        }
    }

    /// Run Phase 1: Survey pass
    /// Returns (summarized_results, lightweight_runs)
    pub fn run_survey_pass(&self) -> (Vec<SimulationResult>, Vec<LightweightRun>) {
        let mut summarized_results = Vec::with_capacity(self.iterations);
        let mut lightweight_runs = Vec::with_capacity(self.iterations);

        for i in 0..self.iterations {
            let seed = i as u64;
            crate::rng::seed_rng(seed);

            let (result, _) =
                crate::run_single_event_driven_simulation(&self.players, &self.timeline, false);
            let score = calculate_score(&result);

            // Create lightweight representation for seed selection
            let mut encounter_scores = Vec::new();
            for (idx, _) in result.encounters.iter().enumerate() {
                encounter_scores.push(calculate_cumulative_score(&result, idx));
            }

            let has_death = result.encounters.iter().any(|e| {
                e.rounds
                    .last()
                    .map(|r| r.team1.iter().any(|c| c.final_state.current_hp == 0))
                    .unwrap_or(false)
            });

            lightweight_runs.push(LightweightRun {
                seed,
                encounter_scores,
                final_score: score,
                total_hp_lost: 0.0,
                total_survivors: 0,
                has_death,
                first_death_encounter: None,
            });

            summarized_results.push(summarize_result(result, seed));
        }

        (summarized_results, lightweight_runs)
    }

    /// Run Phase 2: Selection
    /// Returns interesting seeds based on 1% granularity tiers
    pub fn run_selection(&self, lightweight_runs: &[LightweightRun]) -> Vec<u64> {
        let selected = crate::seed_selection::select_interesting_seeds_with_tiers(lightweight_runs);
        selected.iter().map(|s| s.seed).collect()
    }

    /// Run Phase 3: Deep dive on selected seeds
    /// Returns map of seed -> events
    pub fn run_deep_dive(
        &self,
        interesting_seeds: &[u64],
    ) -> HashMap<u64, Vec<crate::events::Event>> {
        let mut seed_to_events = HashMap::new();

        for &seed in interesting_seeds {
            crate::rng::seed_rng(seed);
            let (_, events) =
                crate::run_single_event_driven_simulation(&self.players, &self.timeline, true);
            seed_to_events.insert(seed, events);
        }

        seed_to_events
    }

    /// Find the median seed from lightweight runs
    pub fn find_median_seed(&self, lightweight_runs: &[LightweightRun]) -> u64 {
        let mut global_scores: Vec<(usize, f64)> = lightweight_runs
            .iter()
            .enumerate()
            .map(|(i, r)| (i, r.final_score))
            .collect();

        global_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        lightweight_runs[global_scores[global_scores.len() / 2].0].seed
    }
}

/// Combine survey results with deep-dive events into final SimulationRun objects
pub fn combine_results_with_events(
    summarized_results: Vec<SimulationResult>,
    lightweight_runs: Vec<LightweightRun>,
    seed_to_events: &HashMap<u64, Vec<crate::events::Event>>,
) -> Vec<SimulationRun> {
    summarized_results
        .into_iter()
        .zip(lightweight_runs)
        .map(|(result, light)| {
            let events = seed_to_events.get(&light.seed).cloned().unwrap_or_default();
            SimulationRun { result, events }
        })
        .collect()
}

/// Run survey pass with progress callback (for WASM interface)
pub fn run_survey_with_progress(
    sim: &ThreePhaseSimulation,
    callback: &Function,
    batch_size: usize,
) -> (Vec<SimulationResult>, Vec<LightweightRun>) {
    let mut summarized_results = Vec::with_capacity(sim.iterations);
    let mut lightweight_runs = Vec::with_capacity(sim.iterations);

    for i in 0..sim.iterations {
        let seed = i as u64;
        crate::rng::seed_rng(seed);

        let (result, _) =
            crate::run_single_event_driven_simulation(&sim.players, &sim.timeline, false);
        let score = calculate_score(&result);

        // Create lightweight representation for seed selection
        let mut encounter_scores = Vec::new();
        for (idx, _) in result.encounters.iter().enumerate() {
            encounter_scores.push(calculate_cumulative_score(&result, idx));
        }

        let has_death = result.encounters.iter().any(|e| {
            e.rounds
                .last()
                .map(|r| r.team1.iter().any(|c| c.final_state.current_hp == 0))
                .unwrap_or(false)
        });

        lightweight_runs.push(LightweightRun {
            seed,
            encounter_scores,
            final_score: score,
            total_hp_lost: 0.0,
            total_survivors: 0,
            has_death,
            first_death_encounter: None,
        });

        summarized_results.push(summarize_result(result, seed));

        if (i + 1) % batch_size == 0 || i == sim.iterations - 1 {
            let progress = ((i + 1) as f64 / sim.iterations as f64) * 0.8;
            report_progress(callback, progress, (i + 1) as f64, sim.iterations as f64);
        }
    }

    (summarized_results, lightweight_runs)
}

/// Run deep dive on selected seeds with progress callback (for WASM interface)
pub fn run_deep_dive_with_progress(
    sim: &ThreePhaseSimulation,
    interesting_seeds: &[u64],
    callback: &Function,
    iterations: usize,
) -> HashMap<u64, Vec<crate::events::Event>> {
    let mut seed_to_events = HashMap::new();

    for (idx, &seed) in interesting_seeds.iter().enumerate() {
        crate::rng::seed_rng(seed);
        let (_, events) =
            crate::run_single_event_driven_simulation(&sim.players, &sim.timeline, true);
        seed_to_events.insert(seed, events);

        let progress = 0.8 + ((idx + 1) as f64 / interesting_seeds.len() as f64) * 0.2;
        report_progress(callback, progress, iterations as f64, iterations as f64);
    }

    seed_to_events
}

/// Extract representative results from sorted runs
pub fn extract_representative_results(final_runs: &[SimulationRun]) -> Vec<SimulationResult> {
    let total_runs = final_runs.len();
    let decile = total_runs as f64 / 10.0;
    let indices = vec![
        (decile * 0.5) as usize,
        (decile * 2.5) as usize,
        total_runs / 2,
        (decile * 7.5) as usize,
        (decile * 9.5) as usize,
    ];

    indices
        .iter()
        .filter(|&&idx| idx < total_runs)
        .map(|&idx| final_runs[idx].result.clone())
        .collect()
}

/// Report progress via callback
fn report_progress(callback: &Function, progress: f64, completed: f64, total: f64) {
    use wasm_bindgen::JsValue;
    let this = JsValue::NULL;
    let _ = callback.call4(
        &this,
        &JsValue::from_f64(progress),
        &JsValue::from_f64(completed),
        &JsValue::from_f64(total),
        &JsValue::NULL,
    );
}
