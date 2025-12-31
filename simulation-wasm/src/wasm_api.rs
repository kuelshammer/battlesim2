use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::model::{Creature, SimulationResult, TimelineStep, SimulationRun};
use crate::storage_manager::StorageManager;
use crate::display_manager::{DisplayManager, DisplayMode, DisplayConfig};
use crate::progress_ui::{ProgressUIManager, ProgressUIConfig};
use crate::user_interaction::{UserInteractionManager, UserEvent, UserInteractionConfig};
#[cfg(not(target_arch = "wasm32"))]
use crate::background_simulation::BackgroundSimulationEngine;
use crate::queue_manager::{QueueManager, QueueManagerConfig};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError, Arc};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FullAnalysisOutput {
    overall: crate::decile_analysis::AggregateOutput,
    encounters: Vec<crate::decile_analysis::AggregateOutput>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FullSimulationOutput {
    results: Vec<SimulationResult>,
    analysis: FullAnalysisOutput,
    first_run_events: Vec<crate::events::Event>,
}

#[wasm_bindgen]
pub struct ChunkedSimulationRunner {
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    total_iterations: usize,
    current_iteration: usize,
    summarized_results: Vec<crate::model::SimulationResult>,
    lightweight_runs: Vec<crate::model::LightweightRun>,
    base_seed: u64,
}

#[wasm_bindgen]
impl ChunkedSimulationRunner {
    #[wasm_bindgen(constructor)]
    pub fn new(players: JsValue, timeline: JsValue, iterations: usize, seed: Option<u64>) -> Result<ChunkedSimulationRunner, JsValue> {
        let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
        let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

        Ok(ChunkedSimulationRunner {
            players,
            timeline,
            total_iterations: iterations,
            current_iteration: 0,
            summarized_results: Vec::with_capacity(iterations),
            lightweight_runs: Vec::with_capacity(iterations),
            base_seed: seed.unwrap_or(0),
        })
    }

    pub fn run_chunk(&mut self, chunk_size: usize) -> f64 {
        let end = (self.current_iteration + chunk_size).min(self.total_iterations);
        
        for i in self.current_iteration..end {
            let seed = self.base_seed.wrapping_add(i as u64);
            crate::rng::seed_rng(seed);

            let (result, _) = crate::simulation::run_single_event_driven_simulation(&self.players, &self.timeline, false);

            let score = crate::aggregation::calculate_score(&result);
            let mut encounter_scores = Vec::new();
            for (idx, _) in result.encounters.iter().enumerate() {
                encounter_scores.push(crate::aggregation::calculate_cumulative_score(&result, idx));
            }

            let has_death = result.encounters.iter().any(|e| {
                e.rounds.last().map(|r| r.team1.iter().any(|c| c.final_state.current_hp == 0)).unwrap_or(false)
            });

            self.lightweight_runs.push(crate::model::LightweightRun {
                seed,
                encounter_scores,
                final_score: score,
                total_hp_lost: 0.0,
                total_survivors: 0,
                has_death,
                first_death_encounter: None,
            });

            self.summarized_results.push(crate::utils::summarize_result(result));
        }

        self.current_iteration = end;
        (self.current_iteration as f64 / self.total_iterations as f64) * 0.8
    }

    pub fn finalize(&mut self) -> Result<JsValue, JsValue> {
        // Phase 2: Selection
        let selected_seeds = crate::seed_selection::select_interesting_seeds_with_tiers(&self.lightweight_runs);
        let interesting_seeds: Vec<u64> = selected_seeds.iter().map(|s| s.seed).collect();

        // Phase 3: Deep Dive
        let mut seed_to_events = HashMap::new();
        let mut median_run_events = Vec::new();

        let mut global_scores: Vec<(usize, f64)> = self.lightweight_runs.iter().enumerate()
            .map(|(i, r)| (i, r.final_score)).collect();
        global_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let median_seed = self.lightweight_runs[global_scores[global_scores.len() / 2].0].seed;

        for &seed in &interesting_seeds {
            crate::rng::seed_rng(seed);
            let (_, events) = crate::simulation::run_single_event_driven_simulation(&self.players, &self.timeline, true);

            if seed == median_seed {
                median_run_events = events.clone();
            }
            seed_to_events.insert(seed, events);
        }

        let mut final_runs: Vec<SimulationRun> = std::mem::take(&mut self.summarized_results).into_iter()
            .zip(std::mem::take(&mut self.lightweight_runs))
            .map(|(result, light)| {
                let events = seed_to_events.get(&light.seed).cloned().unwrap_or_default();
                SimulationRun { result, events }
            }).collect();

        let overall = crate::decile_analysis::run_decile_analysis_with_logs(&mut final_runs, "Current Scenario", self.players.len());

        let num_encounters = final_runs.first().map(|r| r.result.encounters.len()).unwrap_or(0);
        let mut encounters_analysis = Vec::new();
        for i in 0..num_encounters {
            let analysis = crate::decile_analysis::run_encounter_analysis_with_logs(&mut final_runs, i, &format!("Encounter {}", i + 1), self.players.len());
            encounters_analysis.push(analysis);
        }

        final_runs.sort_by(|a, b| {
            let score_a = crate::aggregation::calculate_score(&a.result);
            let score_b = crate::aggregation::calculate_score(&b.result);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_runs = final_runs.len();
        let median_idx = total_runs / 2;
        let decile = total_runs as f64 / 10.0;
        let representative_indices = vec![
            (decile * 0.5) as usize,
            (decile * 2.5) as usize,
            median_idx,
            (decile * 7.5) as usize,
            (decile * 9.5) as usize
        ];

        let mut reduced_results = Vec::new();
        for &idx in &representative_indices {
            if idx < total_runs { reduced_results.push(final_runs[idx].result.clone()); }
        }

        let output = FullSimulationOutput {
            results: reduced_results,
            analysis: FullAnalysisOutput {
                overall,
                encounters: encounters_analysis,
            },
            first_run_events: if median_run_events.is_empty() {
                 final_runs[median_idx].events.clone()
            } else {
                 median_run_events
            },
        };

        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        serde::Serialize::serialize(&output, &serializer)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAdjustmentResult {
    pub monsters: Vec<Creature>,
    pub analysis: crate::decile_analysis::AggregateOutput,
}

#[wasm_bindgen]
pub fn auto_adjust_encounter_wasm(players: JsValue, monsters: JsValue, timeline: JsValue, encounter_index: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let monsters: Vec<Creature> = serde_wasm_bindgen::from_value(monsters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse monsters: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let balancer = crate::auto_balancer::AutoBalancer::new();
    let (optimized_monsters, analysis) = balancer.balance_encounter(players, monsters, timeline, encounter_index);

    let result = AutoAdjustmentResult {
        monsters: optimized_monsters,
        analysis,
    };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Initialize memory guardrails for safe simulation
///
/// Call this once at application startup to set up:
/// - Panic hooks for user-friendly OOM error messages
/// - Memory safety checks for large simulations
#[wasm_bindgen]
pub fn init_memory_guardrails() {
    crate::memory_guardrails::init_memory_guardrails();
}

/// Check if a simulation size requires lightweight mode
///
/// Returns true if iterations > 1000, which means full event logging
/// should be disabled to prevent out-of-memory errors.
#[wasm_bindgen]
pub fn should_force_lightweight_mode(iterations: usize) -> bool {
    crate::memory_guardrails::should_force_lightweight_mode(iterations)
}

#[wasm_bindgen]
pub fn run_simulation_wasm(players: JsValue, timeline: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    // Memory guardrail: Warn for large simulations using old API
    if crate::memory_guardrails::should_force_lightweight_mode(iterations) {
        let msg = crate::memory_guardrails::get_lightweight_mode_message(iterations);
        web_sys::console::warn_1(&msg.into());
    }

    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, None);

    // Extract results from runs for backward compatibility
    let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn run_simulation_with_callback(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
    callback: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    // Memory guardrail: Warn for large simulations
    if crate::memory_guardrails::should_force_lightweight_mode(iterations) {
        let msg = crate::memory_guardrails::get_lightweight_mode_message(iterations);
        web_sys::console::warn_1(&msg.into());
    }

    // Phase 1: Survey Pass (All iterations, results only, no events)
    let mut summarized_results = Vec::with_capacity(iterations);
    let mut lightweight_runs = Vec::with_capacity(iterations);

    let batch_size = (iterations / 20).max(1); // Report progress every 5%

    for i in 0..iterations {
        let seed = i as u64; // Simple deterministic seed for now
        crate::rng::seed_rng(seed);

        let (result, _) = crate::simulation::run_single_event_driven_simulation(&players, &timeline, false);

        // Store for full analysis later (summarized to save memory)
        let score = crate::aggregation::calculate_score(&result);

        // Create lightweight representation for seed selection
        let mut encounter_scores = Vec::new();
        for (idx, _) in result.encounters.iter().enumerate() {
            encounter_scores.push(crate::aggregation::calculate_cumulative_score(&result, idx));
        }

        let has_death = result.encounters.iter().any(|e| {
            e.rounds.last().map(|r| r.team1.iter().any(|c| c.final_state.current_hp == 0)).unwrap_or(false)
        });

        lightweight_runs.push(crate::model::LightweightRun {
            seed,
            encounter_scores,
            final_score: score,
            total_hp_lost: 0.0, // Calculated during analysis
            total_survivors: 0, // Calculated during analysis
            has_death,
            first_death_encounter: None, // Simplified for now
        });

        summarized_results.push(crate::utils::summarize_result(result));

        let is_last_iteration = i == iterations - 1;
        if (i + 1) % batch_size == 0 || is_last_iteration {
            let progress = ((i + 1) as f64 / iterations as f64) * 0.8; // First pass is 80% of total progress
            let this = JsValue::NULL;
            let js_progress = JsValue::from_f64(progress);
            let js_completed = JsValue::from_f64((i + 1) as f64);
            let js_total = JsValue::from_f64(iterations as f64);
            let js_partial_data = JsValue::NULL;
            let _ = callback.call4(&this, &js_progress, &js_completed, &js_total, &js_partial_data);
        }
    }

    // Phase 2: Selection (using new 1% granularity selection)
    let selected_seeds = crate::seed_selection::select_interesting_seeds_with_tiers(&lightweight_runs);
    let interesting_seeds: Vec<u64> = selected_seeds.iter().map(|s| s.seed).collect();

    // Phase 3: Deep Dive (Re-run interesting seeds for events)
    let mut seed_to_events = HashMap::new();
    let mut median_run_events = Vec::new();

    // Sort global scores to find true median events for the fallback
    let mut global_scores: Vec<(usize, f64)> = lightweight_runs.iter().enumerate()
        .map(|(i, r)| (i, r.final_score)).collect();
    global_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let median_seed = lightweight_runs[global_scores[global_scores.len() / 2].0].seed;

    for (idx, &seed) in interesting_seeds.iter().enumerate() {
        crate::rng::seed_rng(seed);
        let (_, events) = crate::simulation::run_single_event_driven_simulation(&players, &timeline, true);

        if seed == median_seed {
            median_run_events = events.clone();
        }
        seed_to_events.insert(seed, events);

        // Progress for Phase 3 (the remaining 20%)
        let progress = 0.8 + ((idx + 1) as f64 / interesting_seeds.len() as f64) * 0.2;
        let this = JsValue::NULL;
        let js_progress = JsValue::from_f64(progress);
        let js_completed = JsValue::from_f64(iterations as f64); // "Completed" is still iterations
        let js_total = JsValue::from_f64(iterations as f64);
        let js_partial_data = JsValue::NULL;
        let _ = callback.call4(&this, &js_progress, &js_completed, &js_total, &js_partial_data);
    }

    // Combine results and events into SimulationRun objects
    // decile_analysis needs a slice of runs to pick logs from
    let mut final_runs: Vec<SimulationRun> = summarized_results.into_iter().zip(lightweight_runs)
        .map(|(result, light)| {
            let events = seed_to_events.get(&light.seed).cloned().unwrap_or_default();
            SimulationRun { result, events }
        }).collect();

    // FINAL ANALYSIS
    let overall = crate::decile_analysis::run_decile_analysis_with_logs(&mut final_runs, "Current Scenario", players.len());

    let num_encounters = final_runs.first().map(|r| r.result.encounters.len()).unwrap_or(0);
    let mut encounters_analysis = Vec::new();
    for i in 0..num_encounters {
        let analysis = crate::decile_analysis::run_encounter_analysis_with_logs(&mut final_runs, i, &format!("Encounter {}", i + 1), players.len());
        encounters_analysis.push(analysis);
    }

    // Sort for representative results extraction
    final_runs.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(&a.result);
        let score_b = crate::aggregation::calculate_score(&b.result);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_runs = final_runs.len();
    let median_idx = total_runs / 2;
    let decile = total_runs as f64 / 10.0;
    let representative_indices = vec![
        (decile * 0.5) as usize,
        (decile * 2.5) as usize,
        median_idx,
        (decile * 7.5) as usize,
        (decile * 9.5) as usize
    ];

    let mut reduced_results = Vec::new();
    for &idx in &representative_indices {
        if idx < total_runs { reduced_results.push(final_runs[idx].result.clone()); }
    }

    let output = FullSimulationOutput {
        results: reduced_results,
        analysis: FullAnalysisOutput {
            overall,
            encounters: encounters_analysis,
        },
        first_run_events: if median_run_events.is_empty() {
             // Fallback if median wasn't in interesting seeds (unlikely)
             final_runs[median_idx].events.clone()
        } else {
             median_run_events
        },
    };

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    serde::Serialize::serialize(&output, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

// Store last simulation events for retrieval (thread-safe)
static LAST_SIMULATION_EVENTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

#[wasm_bindgen]
pub fn run_event_driven_simulation(players: JsValue, timeline: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let mut all_events = Vec::new();
    let mut results = Vec::new();

    for i in 0..iterations {
        let (result, events) = crate::simulation::run_single_event_driven_simulation(&players, &timeline, i == 0);
        results.push(result);

        if i == 0 {
            all_events = events;
        }
    }

    // Format events for WASM storage (legacy behavior)
    let formatted_events: Vec<String> = if !results.is_empty() {
        all_events.iter()
            .map(|e| serde_json::to_string(e).unwrap_or_default())
            .collect()
    } else {
        Vec::new()
    };

    // Store events for retrieval (thread-safe)
    if let Ok(mut events_guard) = LAST_SIMULATION_EVENTS.lock() {
        *events_guard = Some(formatted_events);
    }

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn get_last_simulation_events() -> Result<JsValue, JsValue> {
    match LAST_SIMULATION_EVENTS.lock() {
        Ok(events_guard) => {
            match events_guard.as_ref() {
                Some(events) => {
                    let serializer = serde_wasm_bindgen::Serializer::new()
                        .serialize_maps_as_objects(true);
                    serde::Serialize::serialize(&events, &serializer)
                        .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
                }
                None => Ok(JsValue::from_str("No simulation events available")),
            }
        }
        Err(_) => Ok(JsValue::from_str("Error accessing simulation events")),
    }
}

/// O(1) memory WASM simulation using rolling aggregation
/// Returns SimulationSummary with aggregated statistics instead of all individual results
#[wasm_bindgen]
pub fn run_simulation_wasm_rolling_stats(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
    seed: Option<u64>,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    // Use the new three-tier system with 1% granularity
    let summary = crate::two_pass::run_simulation_with_three_tier(players, timeline, iterations, false, seed);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&summary, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize summary: {}", e)))
}

/// Run a batch of simulations synchronously
#[wasm_bindgen]
pub fn run_batch_simulation_wasm(
    batch_request: JsValue,
) -> Result<JsValue, JsValue> {
    let request: crate::model::BatchSimulationRequest = serde_wasm_bindgen::from_value(batch_request)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse batch request: {}", e)))?;

    let mut results = Vec::with_capacity(request.jobs.len());

    for job in request.jobs {
        // Use the new three-tier system with 1% granularity
        let summary = crate::two_pass::run_simulation_with_three_tier(
            job.players,
            job.timeline,
            job.iterations,
            false,
            job.seed,
        );
        results.push(crate::model::BatchSimulationResult {
            id: job.id,
            summary,
        });
    }

    let response = crate::model::BatchSimulationResponse { results };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&response, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize batch response: {}", e)))
}

/// Run a batch of simulations with progress callback
#[wasm_bindgen]
pub fn run_batch_simulation_with_callback(
    batch_request: JsValue,
    callback: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let request: crate::model::BatchSimulationRequest = serde_wasm_bindgen::from_value(batch_request)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse batch request: {}", e)))?;

    let total_jobs = request.jobs.len();
    let mut results = Vec::with_capacity(total_jobs);

    for (i, job) in request.jobs.into_iter().enumerate() {
        // Use the new three-tier system with 1% granularity
        let summary = crate::two_pass::run_simulation_with_three_tier(
            job.players,
            job.timeline,
            job.iterations,
            false,
            job.seed,
        );

        results.push(crate::model::BatchSimulationResult {
            id: job.id,
            summary,
        });

        // Report progress
        let progress = (i + 1) as f64 / total_jobs as f64;
        let this = JsValue::NULL;
        let js_progress = JsValue::from_f64(progress);
        let js_completed = JsValue::from_f64((i + 1) as f64);
        let js_total = JsValue::from_f64(total_jobs as f64);
        let js_partial_data = JsValue::NULL;

        let _ = callback.call4(&this, &js_progress, &js_completed, &js_total, &js_partial_data);
    }

    let response = crate::model::BatchSimulationResponse { results };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&response, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize batch response: {}", e)))
}


#[wasm_bindgen]
pub fn clear_simulation_cache() {
    crate::cache::clear_cache();
}

/// Get cache statistics for memory monitoring
/// Returns an object with entry_count and estimated_bytes
#[wasm_bindgen]
pub fn get_cache_stats() -> JsValue {
    let (entry_count, estimated_bytes) = crate::cache::get_cache_stats();
    let stats = serde_json::json!({
        "entryCount": entry_count,
        "estimatedBytes": estimated_bytes
    });
    JsValue::from_str(&stats.to_string())
}

/// Public Rust function for event-driven simulation (for CLI/testing)
/// Returns all simulation runs with their results and events
pub fn run_event_driven_simulation_rust(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> Vec<crate::model::SimulationRun> {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    let mut all_runs = Vec::new();

    for i in 0..iterations {
        // If a seed is provided, use it with the iteration index for determinism
        // This ensures each iteration is deterministic but different from others
        if let Some(s) = seed {
            crate::rng::seed_rng(s.wrapping_add(i as u64));
        }

        let (result, events) = crate::simulation::run_single_event_driven_simulation(&players, &timeline, true);
        let run = crate::model::SimulationRun {
            result,
            events,
        };
        all_runs.push(run);
    }

    // Clear the seeded RNG after simulation completes
    if seed.is_some() {
        crate::rng::clear_rng();
    }

    // Sort results by score (worst to best) with safe comparison
    all_runs.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(&a.result);
        let score_b = crate::aggregation::calculate_score(&b.result);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    all_runs
}


// Global storage manager for WASM interface
static STORAGE_MANAGER: OnceLock<Mutex<StorageManager>> = OnceLock::new();

/// Initialize or get the global storage manager
fn get_storage_manager() -> &'static Mutex<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| Mutex::new(StorageManager::default()))
}

#[wasm_bindgen]
pub fn run_decile_analysis_wasm(results: JsValue, scenario_name: &str, _party_size: usize) -> Result<JsValue, JsValue> {
    // Add debug logging
    console::log_1(&"=== Decile Analysis WASM Debug ===".into());

    let mut results: Vec<SimulationResult> = serde_wasm_bindgen::from_value(results)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse results: {}", e)))?;

    console::log_1(&format!("Received {} simulation results", results.len()).into());

    // Sort results by score from worst to best performance with safe comparison
    results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Calculate party size from first result (use actual data instead of parameter)
    let actual_party_size = if let Some(first_result) = results.first() {
        if let Some(first_encounter) = first_result.encounters.first() {
            first_encounter.rounds.first()
                .map(|first_round| first_round.team1.len())
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    console::log_1(&format!("Calculated party size: {}", actual_party_size).into());

    // 1. Run Overall Analysis (Adventure-wide)
    let overall = crate::decile_analysis::run_decile_analysis(&results, scenario_name, actual_party_size);

    // 2. Run Per-Encounter Analysis
    // Determine number of encounters from the first result
    let num_encounters = results.first().map(|r| r.encounters.len()).unwrap_or(0);
    let mut encounters = Vec::new();

    for i in 0..num_encounters {
        let encounter_name = format!("Encounter {}", i + 1);
        let analysis = crate::decile_analysis::run_encounter_analysis(&results, i, &encounter_name, actual_party_size);
        encounters.push(analysis);
    }

    console::log_1(&format!("Generated overall analysis + {} encounter analyses", encounters.len()).into());

    let output = FullAnalysisOutput {
        overall,
        encounters,
    };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&output, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize decile analysis: {}", e)))
}

// ===== PHASE 3: GUI INTEGRATION WASM BINDINGS =====

static GUI_INTEGRATION: OnceLock<Mutex<GuiIntegration>> = OnceLock::new();

/// Combined GUI integration system
#[allow(dead_code)]
struct GuiIntegration {
    display_manager: Arc<Mutex<DisplayManager>>,
    progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
    user_interaction_manager: Arc<Mutex<UserInteractionManager>>,
}

/// Initialize the GUI integration system
#[wasm_bindgen]
pub fn initialize_gui_integration() -> Result<JsValue, JsValue> {
    GUI_INTEGRATION.get_or_init(|| {
        // Get storage manager reference (clone for separate instances as in original code)
        let storage_copy = get_storage_manager().lock().unwrap_or_else(PoisonError::into_inner).clone();

        // Create display manager
        let display_config = DisplayConfig::default();
        let display_manager = DisplayManager::new(
            storage_copy.clone(),
            display_config,
        );
        let display_manager_arc = Arc::new(Mutex::new(display_manager));

        // Create queue manager
        let queue_config = QueueManagerConfig::default();
        let queue_manager = QueueManager::new(queue_config);
        let queue_manager_arc = Arc::new(Mutex::new(queue_manager));

        // Create progress UI manager
        let progress_ui_manager = ProgressUIManager::new(ProgressUIConfig::default());
        let progress_ui_manager_arc = Arc::new(Mutex::new(progress_ui_manager));

        // Create storage manager arc for user interaction
        let _storage_manager_arc = Arc::new(Mutex::new(storage_copy.clone()));

        // Create user interaction manager (conditional compilation for background simulation)
        let interaction_config = UserInteractionConfig::default();
        #[cfg(not(target_arch = "wasm32"))]
        let user_interaction_manager = {
            // Create background simulation engine
            let (simulation_engine, _progress_receiver) = BackgroundSimulationEngine::new();
            let simulation_engine_arc = Arc::new(Mutex::new(simulation_engine));

            UserInteractionManager::new_with_simulation(
                display_manager_arc.clone(),
                progress_ui_manager_arc.clone(),
                simulation_engine_arc,
                queue_manager_arc.clone(),
                interaction_config,
            )
        };
        #[cfg(target_arch = "wasm32")]
        let user_interaction_manager = {
            UserInteractionManager::new(
                display_manager_arc.clone(),
                progress_ui_manager_arc.clone(),
                queue_manager_arc.clone(),
                interaction_config,
            )
        };

        Mutex::new(GuiIntegration {
            display_manager: display_manager_arc,
            progress_ui_manager: progress_ui_manager_arc,
            user_interaction_manager: Arc::new(Mutex::new(user_interaction_manager)),
        })
    });

    Ok(JsValue::from_str("GUI integration initialized"))
}

/// Get the GUI integration system
fn get_gui_integration() -> &'static Mutex<GuiIntegration> {
    GUI_INTEGRATION.get().expect("GUI Integration not initialized")
}

/// Get display results for current parameters
#[wasm_bindgen]
pub fn get_display_results(players: JsValue, timeline: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let display_result = {
        let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
        display_manager.get_display_results(&players, &timeline, iterations)
    };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&display_result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize display result: {}", e)))
}

/// Set display mode
#[wasm_bindgen]
pub fn set_display_mode(mode_str: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let mode = match mode_str {
        "ShowNewest" => DisplayMode::ShowNewest,
        "ShowMostSimilar" => DisplayMode::ShowMostSimilar,
        "LetUserChoose" => DisplayMode::LetUserChoose,
        "PrimaryOnly" => DisplayMode::PrimaryOnly,
        "SecondaryOnly" => DisplayMode::SecondaryOnly,
        _ => return Err(JsValue::from_str(&format!("Invalid display mode: {}", mode_str))),
    };

    let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    display_manager.set_display_mode(mode);

    Ok(JsValue::from_str(&format!("Display mode set to {:?}", mode)))
}

/// Get current display mode
#[wasm_bindgen]
pub fn get_display_mode() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let mode = display_manager.get_display_mode();

    let mode_str = match mode {
        DisplayMode::ShowNewest => "ShowNewest",
        DisplayMode::ShowMostSimilar => "ShowMostSimilar",
        DisplayMode::LetUserChoose => "LetUserChoose",
        DisplayMode::PrimaryOnly => "PrimaryOnly",
        DisplayMode::SecondaryOnly => "SecondaryOnly",
    };

    Ok(JsValue::from_str(mode_str))
}

/// User selected a specific slot
#[wasm_bindgen]
pub fn user_selected_slot(slot_str: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let slot_selection = match slot_str {
        "Primary" => crate::storage::SlotSelection::Primary,
        "Secondary" => crate::storage::SlotSelection::Secondary,
        _ => return Err(JsValue::from_str(&format!("Invalid slot: {}", slot_str))),
    };

    let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let display_result = display_manager.user_selected_slot(slot_selection);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&display_result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize display result: {}", e)))
}

/// Start a background simulation
#[wasm_bindgen]
pub fn start_background_simulation(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
    priority_str: &str
) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let priority = match priority_str {
        "Low" => crate::background_simulation::SimulationPriority::Low,
        "Normal" => crate::background_simulation::SimulationPriority::Normal,
        "High" => crate::background_simulation::SimulationPriority::High,
        "Critical" => crate::background_simulation::SimulationPriority::Critical,
        _ => return Err(JsValue::from_str(&format!("Invalid priority: {}", priority_str))),
    };

    let event = crate::user_interaction::UserEvent::RequestSimulation {
        parameters: crate::user_interaction::ScenarioParameters {
            players: players.clone(),
            timeline: timeline.clone(),
            iterations,
        },
        priority,
    };

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let result = user_interaction.handle_event(event);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get progress information for all active simulations
#[wasm_bindgen]
pub fn get_all_progress() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let progress_list = progress_ui.get_all_progress();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&progress_list, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize progress list: {}", e)))
}

/// Get progress information for a specific simulation
#[wasm_bindgen]
pub fn get_progress(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());

    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let progress_info = progress_ui.get_progress(&sim_id);

    match progress_info {
        Some(info) => {
            let serializer = serde_wasm_bindgen::Serializer::new()
                .serialize_maps_as_objects(true);

            serde::Serialize::serialize(&info, &serializer)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize progress info: {}", e)))
        },
        None => Ok(JsValue::NULL),
    }
}

/// Create HTML progress bar for a simulation
#[wasm_bindgen]
pub fn create_progress_bar(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());

    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let progress_info = progress_ui.get_progress(&sim_id);

    match progress_info {
        Some(info) => {
            let html = progress_ui.create_progress_bar_html(&info);
            Ok(JsValue::from_str(&html))
        },
        None => Ok(JsValue::from_str("")),
    }
}

/// Create compact progress indicator for a simulation
#[wasm_bindgen]
pub fn create_compact_indicator(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());

    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let progress_info = progress_ui.get_progress(&sim_id);

    match progress_info {
        Some(info) => {
            let html = progress_ui.create_compact_indicator(&info);
            Ok(JsValue::from_str(&html))
        },
        None => Ok(JsValue::from_str("")),
    }
}

/// Cancel a running simulation
#[wasm_bindgen]
pub fn cancel_simulation(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());

    let event = UserEvent::CancelSimulation {
        simulation_id: sim_id,
    };

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let result = user_interaction.handle_event(event);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Clear simulation cache
#[wasm_bindgen]
pub fn clear_simulation_cache_gui() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let event = UserEvent::ClearCache;

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let result = user_interaction.handle_event(event);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get pending user confirmations
#[wasm_bindgen]
pub fn get_pending_confirmations() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let confirmations = user_interaction.get_pending_confirmations();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&confirmations, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize confirmations: {}", e)))
}

/// Answer a confirmation request
#[wasm_bindgen]
pub fn answer_confirmation(confirmation_id: &str, confirmed: bool) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let result = user_interaction.answer_confirmation(confirmation_id, confirmed);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get current user interaction state
#[wasm_bindgen]
pub fn get_user_interaction_state() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let state = user_interaction.get_state();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&state, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize state: {}", e)))
}

/// Update GUI configuration
#[wasm_bindgen]
pub fn update_gui_configuration(
    display_config_json: Option<JsValue>,
    progress_config_json: Option<JsValue>,
    interaction_config_json: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    // Update display configuration
    if let Some(config_js) = display_config_json {
        let config: DisplayConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse display config: {}", e)))?;

        let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
        display_manager.update_config(config);
    }

    // Update progress configuration
    if let Some(config_js) = progress_config_json {
        let config: ProgressUIConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse progress config: {}", e)))?;

        let mut progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
        progress_ui.update_config(config);
    }

    // Update interaction configuration
    if let Some(config_js) = interaction_config_json {
        let config: UserInteractionConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse interaction config: {}", e)))?;

        let mut user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
        user_interaction.update_config(config);
    }

    Ok(JsValue::from_str("Configuration updated"))
}

/// Get progress summary for dashboard
#[wasm_bindgen]
pub fn get_progress_summary() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let summary = progress_ui.get_progress_summary();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&summary, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize summary: {}", e)))
}

/// Handle parameter change event
#[wasm_bindgen]
pub fn handle_parameters_changed(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap_or_else(PoisonError::into_inner);

    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let event = crate::user_interaction::UserEvent::ParametersChanged {
        parameters: crate::user_interaction::ScenarioParameters {
            players: players.clone(),
            timeline: timeline.clone(),
            iterations,
        },
    };

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    let result = user_interaction.handle_event(event);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}
