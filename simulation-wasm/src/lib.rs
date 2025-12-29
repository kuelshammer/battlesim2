pub mod dice;
pub mod rng;
pub mod actions;
pub mod targeting;
pub mod enums;
pub mod model;
pub mod aggregation;
pub mod cleanup;
pub mod resolution;
pub mod resources;
pub mod events;
pub mod context;
pub mod reactions;
pub mod execution;
pub mod action_resolver;
pub mod validation; // New module for requirement validation
pub mod utilities;
pub mod decile_analysis;
pub mod combat_stats;
pub mod scoring_test;
pub mod creature_adjustment;
pub mod adjustment_test;
pub mod auto_balancer;
pub mod dice_reconstruction;
pub mod intensity_calculation;
#[cfg(test)]
mod intensity_test;
pub mod error_handling; // Enhanced error handling system
pub mod enhanced_validation; // Comprehensive validation
pub mod recovery; // Error recovery mechanisms
pub mod safe_aggregation; // Safe aggregation functions
pub mod monitoring; // Success metrics and monitoring
pub mod background_simulation; // Background simulation engine
pub mod queue_manager; // Queue management system
pub mod progress_communication; // Progress communication system
pub mod display_manager; // Display mode management
pub mod progress_ui; // Progress UI components
pub mod user_interaction; // User interaction flows
pub mod config; // Configuration system
pub mod phase3_gui_integration; // Phase 3 GUI Integration demonstration
pub mod phase3_working; // Phase 3 GUI Integration working implementation
pub mod storage; // Stub storage module
pub mod storage_manager; // Stub storage manager module
pub mod storage_integration; // Stub storage integration module
pub mod cache;
pub mod log_reproduction_test;


use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::model::{Creature, SimulationResult, Combattant, CreatureState, TimelineStep, SimulationRun};
use crate::execution::ActionExecutionEngine;
use crate::storage_manager::StorageManager;
use crate::resources::ResetType;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PartialOutput {
    results: Vec<SimulationResult>,
    analysis: FullAnalysisOutput,
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
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

#[wasm_bindgen]
pub fn run_simulation_wasm(players: JsValue, timeline: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, None);

    // Extract results from runs for backward compatibility
    let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

/// Reduces memory usage of a SimulationResult by keeping only the first and last rounds 
/// of each encounter, which are sufficient for decile analysis and visualization.
fn summarize_result(mut result: SimulationResult) -> SimulationResult {
    for encounter in &mut result.encounters {
        if encounter.rounds.len() > 2 {
            let first = encounter.rounds.first().cloned().unwrap();
            let last = encounter.rounds.last().cloned().unwrap();
            encounter.rounds = vec![first, last];
        }
    }
    result
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

    // Phase 1: Survey Pass (All iterations, results only, no events)
    let mut summarized_results = Vec::with_capacity(iterations);
    let mut lightweight_runs = Vec::with_capacity(iterations);

    let batch_size = (iterations / 20).max(1); // Report progress every 5%

    for i in 0..iterations {
        let seed = i as u64; // Simple deterministic seed for now
        crate::rng::seed_rng(seed);
        
        let (result, _) = run_single_event_driven_simulation(&players, &timeline, false);
        
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

        summarized_results.push(summarize_result(result));

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

    // Phase 2: Selection
    let interesting_seeds = select_interesting_seeds(&lightweight_runs);
    
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
        let (_, events) = run_single_event_driven_simulation(&players, &timeline, true);
        
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
    let overall = decile_analysis::run_decile_analysis_with_logs(&mut final_runs, "Current Scenario", players.len());
    
    let num_encounters = final_runs.first().map(|r| r.result.encounters.len()).unwrap_or(0);
    let mut encounters_analysis = Vec::new();
    for i in 0..num_encounters {
        let analysis = decile_analysis::run_encounter_analysis_with_logs(&mut final_runs, i, &format!("Encounter {}", i + 1), players.len());
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

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(false);
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
        let (result, events) = run_single_event_driven_simulation(&players, &timeline, i == 0);
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
        .serialize_maps_as_objects(false);

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
                        .serialize_maps_as_objects(false);
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

    let summary = run_simulation_with_rolling_stats(players, timeline, iterations, false, seed);

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

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
        let summary = run_simulation_with_rolling_stats(
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
        .serialize_maps_as_objects(false);

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
        // Run simulation
        let summary = run_simulation_with_rolling_stats(
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
        .serialize_maps_as_objects(false);

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

        let (result, events) = run_single_event_driven_simulation(&players, &timeline, true);
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

/// Two-Pass Deterministic Re-simulation implementation
/// Phase 1: Lightweight survey pass (no events, ~70 KB memory)
/// Phase 2: Identify interesting seeds (deciles, deaths, extremes)
/// Phase 3: Re-run only interesting seeds with full events (~5 MB memory)
/// Total memory: ~5 MB (vs ~15-20 MB for storing all runs)
pub fn run_simulation_with_rolling_stats(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> crate::model::SimulationSummary {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    // Phase 1: Survey pass - lightweight simulation for all iterations
    let base_seed = seed.unwrap_or(0);
    let lightweight_runs = run_survey_pass(players.clone(), timeline.clone(), iterations, Some(base_seed));

    // Phase 2: Identify interesting seeds for re-simulation
    let interesting_seeds = select_interesting_seeds(&lightweight_runs);

    // Phase 3: Deep dive pass - re-run interesting seeds with full events
    let mut sample_runs = Vec::new();
    for seed in &interesting_seeds {
        crate::rng::seed_rng(*seed);
        let (result, events) = run_single_event_driven_simulation(&players, &timeline, false);
        sample_runs.push(crate::model::SimulationRun { result, events });
    }

    // Clear the seeded RNG after simulation completes
    crate::rng::clear_rng();

    // Calculate statistics from lightweight runs
    let mut sorted_scores: Vec<f64> = lightweight_runs.iter().map(|r| r.final_score).collect();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut score_sum = 0.0;
    let mut score_sum_squared = 0.0;
    let score_min = *sorted_scores.first().unwrap_or(&0.0);
    let score_max = *sorted_scores.last().unwrap_or(&0.0);

    for &score in &sorted_scores {
        score_sum += score;
        score_sum_squared += score * score;
    }

    let mean = if iterations > 0 { score_sum / iterations as f64 } else { 0.0 };
    let variance = if iterations > 0 { (score_sum_squared / iterations as f64) - (mean * mean) } else { 0.0 };
    let std_dev = variance.sqrt().max(0.0);

    let median = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() / 2] } else { 0.0 };
    let p25 = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() / 4] } else { 0.0 };
    let p75 = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() * 3 / 4] } else { 0.0 };

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

/// Three-Tier Two-Pass Deterministic Re-simulation implementation with 1% granularity
/// Phase 1: Lightweight survey pass (10,100 iterations, ~323 KB memory)
/// Phase 2: Identify interesting seeds with 1% bucket granularity and tier classification
/// Phase 3: Re-run only interesting seeds with tier-appropriate event collection
///   - Tier A: Full events for 11 decile logs (~2.2 MB)
///   - Tier B: Lean events for 100 1% medians (~2 MB)
///   - Tier C: No events, use lightweight data (~2 KB)
/// Total memory: ~4.5 MB (vs ~15-20 MB for all full events)
pub fn run_simulation_with_three_tier(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    _log_enabled: bool,
    seed: Option<u64>,
) -> crate::model::SimulationSummary {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    let _ = console_log::init_with_level(log::Level::Info);

    // Phase 1: Survey pass - lightweight simulation for all iterations
    let base_seed = seed.unwrap_or(0);
    let lightweight_runs = run_survey_pass(players.clone(), timeline.clone(), iterations, Some(base_seed));

    // Phase 2: Identify interesting seeds with 1% granularity and tier classification
    let selected_seeds = select_interesting_seeds_with_tiers(&lightweight_runs);

    // Phase 3: Deep dive pass - re-run selected seeds with tier-appropriate event collection
    let mut sample_runs = Vec::new();
    for selected_seed in &selected_seeds {
        crate::rng::seed_rng(selected_seed.seed);

        match selected_seed.tier {
            crate::model::InterestingSeedTier::TierA => {
                // Full events for decile logs
                let (result, events) = run_single_event_driven_simulation(&players, &timeline, false);
                sample_runs.push(crate::model::SimulationRun { result, events });
            }
            crate::model::InterestingSeedTier::TierB => {
                // Lean events for 1% medians
                // TODO: For now, we run full events but store fewer runs
                // In a future update, we'd use execute_encounter_lean() for true lean collection
                let (result, events) = run_single_event_driven_simulation(&players, &timeline, false);
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
    let mut sorted_scores: Vec<f64> = lightweight_runs.iter().map(|r| r.final_score).collect();
    sorted_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut score_sum = 0.0;
    let mut score_sum_squared = 0.0;
    let score_min = *sorted_scores.first().unwrap_or(&0.0);
    let score_max = *sorted_scores.last().unwrap_or(&0.0);

    for &score in &sorted_scores {
        score_sum += score;
        score_sum_squared += score * score;
    }

    let mean = if iterations > 0 { score_sum / iterations as f64 } else { 0.0 };
    let variance = if iterations > 0 { (score_sum_squared / iterations as f64) - (mean * mean) } else { 0.0 };
    let std_dev = variance.sqrt().max(0.0);

    let median = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() / 2] } else { 0.0 };
    let p25 = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() / 4] } else { 0.0 };
    let p75 = if !sorted_scores.is_empty() { sorted_scores[sorted_scores.len() * 3 / 4] } else { 0.0 };

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


pub fn run_single_event_driven_simulation(players: &[Creature], timeline: &[crate::model::TimelineStep], _log_enabled: bool) -> (SimulationResult, Vec<crate::events::Event>) {
    let mut all_events = Vec::new();
    let mut players_with_state = Vec::new();

    // Initialize players with state - IDs are prefixed with 'p-' to ensure they are unique
    // and carried over correctly across encounters.
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
            let mut p = player.clone();
            p.name = name;
            p.mode = "player".to_string(); 
            let id = format!("p-{}-{}-{}", group_idx, i, player.id);

            // Create CreatureState
            let state = CreatureState {
                current_hp: p.hp,
                temp_hp: None,
                buffs: HashMap::new(),
                resources: {
                    let mut r = crate::model::SerializableResourceLedger::from(p.initialize_ledger());
                    // Initialize per-action resources (1/fight, 1/day, Limited, Recharge)
                    let action_uses = crate::actions::get_remaining_uses(&p, "long rest", None);
                    for (action_id, uses) in action_uses {
                        r.current.insert(action_id, uses);
                    }
                    r
                },
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: None,
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
            };

            // Create Combattant for ActionExecutionEngine
            let combattant = Combattant {
                team: 0,
                id: id.clone(),
                creature: std::sync::Arc::new(p.clone()),
                initiative: crate::utilities::roll_initiative(&p),
                initial_state: state.clone(),
                final_state: state,
                actions: Vec::new(),
            };

            players_with_state.push(combattant);
        }
    }

    let mut encounter_results = Vec::new();
    let num_combat_encounters = timeline.iter().filter(|step| matches!(step, crate::model::TimelineStep::Combat(_))).count();

    for (step_idx, step) in timeline.iter().enumerate() {
        match step {
            crate::model::TimelineStep::Combat(encounter) => {
                // Create enemy combatants - IDs include encounter index to be globally unique
                let mut enemies = Vec::new();
                for (group_idx, monster) in encounter.monsters.iter().enumerate() {
                    for i in 0..monster.count as i32 {
                        let name = if monster.count > 1.0 { format!("{} {}", monster.name, i + 1) } else { monster.name.clone() };
                        let mut m = monster.clone();
                        m.name = name;
                        let id = format!("step{}-m-{}-{}-{}", step_idx, group_idx, i, monster.id);

                        let enemy_state = CreatureState {
                            current_hp: m.hp,
                            temp_hp: None,
                            buffs: HashMap::new(),
                            resources: {
                                let mut r = crate::model::SerializableResourceLedger::from(m.initialize_ledger());
                                // Initialize per-action resources (1/fight, 1/day, Limited, Recharge)
                                let action_uses = crate::actions::get_remaining_uses(&m, "long rest", None);
                                for (action_id, uses) in action_uses {
                                    r.current.insert(action_id, uses);
                                }
                                r
                            },
                            upcoming_buffs: HashMap::new(),
                            used_actions: HashSet::new(),
                            concentrating_on: None,
                            actions_used_this_encounter: HashSet::new(),
                            bonus_action_used: false,
                            known_ac: HashMap::new(),
                            arcane_ward_hp: None,
                        };

                        let enemy_combattant = Combattant {
                            team: 1,
                            id: id.clone(),
                            creature: std::sync::Arc::new(m.clone()),
                            initiative: crate::utilities::roll_initiative(&m),
                            initial_state: enemy_state.clone(),
                            final_state: enemy_state,
                            actions: Vec::new(),
                        };

                        enemies.push(enemy_combattant);
                    }
                }

                // Combine all combatants for this encounter
                let mut all_combatants = players_with_state.clone();
                all_combatants.extend(enemies);

                // Create ActionExecutionEngine
                let mut engine = ActionExecutionEngine::new(all_combatants.clone(), true);

                // Run encounter using the ActionExecutionEngine
                let encounter_result = engine.execute_encounter();

                // Collect events (raw)
                all_events.extend(encounter_result.event_history.clone());

                // Convert to old format for compatibility
                let legacy_result = convert_to_legacy_simulation_result(&encounter_result, step_idx, encounter.target_role.clone());
                encounter_results.push(legacy_result);

                // Update player states for next encounter (no rest here, rest is its own step)
                players_with_state = update_player_states_for_next_encounter(&players_with_state, &encounter_result, false);
            },
            crate::model::TimelineStep::ShortRest(_) => {
                // Apply standalone short rest recovery
                players_with_state = apply_short_rest_standalone(&players_with_state, &mut all_events);
                
                // Add an encounter result with one round snapshot to capture the state after rest
                let after_rest_team1 = players_with_state.to_vec();
                
                encounter_results.push(crate::model::EncounterResult {
                    stats: HashMap::new(),
                    rounds: vec![crate::model::Round {
                        team1: after_rest_team1,
                        team2: Vec::new(),
                    }],
                    target_role: crate::model::TargetRole::Standard,
                });
            }
        }
    }

    // SimulationResult is now SimulationRunData struct
    let mut result = SimulationResult { 
        encounters: encounter_results, 
        score: None,
        num_combat_encounters
    };
    
    // Calculate efficiency score
    let score = crate::aggregation::calculate_efficiency_score(&result, &all_events);
    result.score = Some(score);

    (result, all_events)
}

/// Lightweight simulation that tracks only scores and deaths, no event collection
/// Used in Phase 1 of Two-Pass system to identify interesting runs for re-simulation
pub fn run_single_lightweight_simulation(
    players: &[Creature],
    timeline: &[crate::model::TimelineStep],
    seed: u64,
) -> crate::model::LightweightRun {
    // Seed RNG for deterministic results
    crate::rng::seed_rng(seed);

    let mut players_with_state = Vec::new();
    let mut encounter_scores = Vec::new();
    let mut has_death = false;
    let mut first_death_encounter: Option<usize> = None;

    // Initialize players with state - IDs are prefixed with 'p-' to ensure they are unique
    // and carried over correctly across encounters.
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
            let mut p = player.clone();
            p.name = name;
            p.mode = "player".to_string();
            let id = format!("p-{}-{}-{}", group_idx, i, player.id);

            // Create CreatureState
            let state = CreatureState {
                current_hp: p.hp,
                temp_hp: None,
                buffs: HashMap::new(),
                resources: {
                    let mut r = crate::model::SerializableResourceLedger::from(p.initialize_ledger());
                    // Initialize per-action resources (1/fight, 1/day, Limited, Recharge)
                    let action_uses = crate::actions::get_remaining_uses(&p, "long rest", None);
                    for (action_id, uses) in action_uses {
                        r.current.insert(action_id, uses);
                    }
                    r
                },
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: None,
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
            };

            // Create Combattant for ActionExecutionEngine
            let combattant = Combattant {
                team: 0,
                id: id.clone(),
                creature: std::sync::Arc::new(p.clone()),
                initiative: crate::utilities::roll_initiative(&p),
                initial_state: state.clone(),
                final_state: state,
                actions: Vec::new(),
            };

            players_with_state.push(combattant);
        }
    }

    for (step_idx, step) in timeline.iter().enumerate() {
        match step {
            crate::model::TimelineStep::Combat(encounter) => {
                // Create enemy combatants - IDs include encounter index to be globally unique
                let mut enemies = Vec::new();
                for (group_idx, monster) in encounter.monsters.iter().enumerate() {
                    for i in 0..monster.count as i32 {
                        let name = if monster.count > 1.0 { format!("{} {}", monster.name, i + 1) } else { monster.name.clone() };
                        let mut m = monster.clone();
                        m.name = name;
                        let id = format!("step{}-m-{}-{}-{}", step_idx, group_idx, i, monster.id);

                        let enemy_state = CreatureState {
                            current_hp: m.hp,
                            temp_hp: None,
                            buffs: HashMap::new(),
                            resources: {
                                let mut r = crate::model::SerializableResourceLedger::from(m.initialize_ledger());
                                // Initialize per-action resources (1/fight, 1/day, Limited, Recharge)
                                let action_uses = crate::actions::get_remaining_uses(&m, "long rest", None);
                                for (action_id, uses) in action_uses {
                                    r.current.insert(action_id, uses);
                                }
                                r
                            },
                            upcoming_buffs: HashMap::new(),
                            used_actions: HashSet::new(),
                            concentrating_on: None,
                            actions_used_this_encounter: HashSet::new(),
                            bonus_action_used: false,
                            known_ac: HashMap::new(),
                            arcane_ward_hp: None,
                        };

                        let enemy_combattant = Combattant {
                            team: 1,
                            id: id.clone(),
                            creature: std::sync::Arc::new(m.clone()),
                            initiative: crate::utilities::roll_initiative(&m),
                            initial_state: enemy_state.clone(),
                            final_state: enemy_state,
                            actions: Vec::new(),
                        };

                        enemies.push(enemy_combattant);
                    }
                }

                // Combine all combatants for this encounter
                let mut all_combatants = players_with_state.clone();
                all_combatants.extend(enemies);

                // Create ActionExecutionEngine
                let mut engine = ActionExecutionEngine::new(all_combatants.clone(), false);

                // Run encounter using the ActionExecutionEngine (NO event collection, NO snapshots)
                let encounter_result = engine.execute_encounter();

                // Track cumulative score after this combat encounter
                let score = crate::safe_aggregation::calculate_lightweight_score(&encounter_result.final_combatant_states);
                encounter_scores.push(score);

                // Check for deaths in this encounter
                if !has_death {
                    for combatant in &encounter_result.final_combatant_states {
                        if combatant.current_hp == 0 && combatant.base_combatant.team == 0 {
                            has_death = true;
                            first_death_encounter = Some(encounter_scores.len() - 1);
                            break;
                        }
                    }
                }

                // Update player states for next encounter (no rest here, rest is its own step)
                players_with_state = update_player_states_for_next_encounter(&players_with_state, &encounter_result, false);
            },
            crate::model::TimelineStep::ShortRest(_) => {
                // Apply standalone short rest recovery (NO event collection)
                players_with_state = apply_short_rest_standalone_no_events(&players_with_state);
            }
        }
    }

    // Calculate final stats from the last state of players
    let total_survivors = players_with_state.iter().filter(|p| p.final_state.current_hp > 0).count();
    
    // final_score is the score of the last completed encounter
    let final_score = encounter_scores.last().cloned().unwrap_or(0.0);
    
    // total_hp_lost is (Total Daily Net Worth EHP) - (Final EHP)
    let tdnw = crate::decile_analysis::calculate_tdnw_lightweight(players);
    let mut final_ehp = 0.0;
    for p in &players_with_state {
        let ledger = p.creature.initialize_ledger();
        final_ehp += crate::intensity_calculation::calculate_serializable_ehp(
            p.final_state.current_hp,
            p.final_state.temp_hp.unwrap_or(0),
            &p.final_state.resources,
            &ledger.reset_rules
        );
    }
    let total_hp_lost = (tdnw - final_ehp).max(0.0);

    // Clear the seeded RNG after simulation completes
    crate::rng::clear_rng();

    crate::model::LightweightRun {
        seed,
        encounter_scores,
        final_score,
        total_hp_lost,
        total_survivors,
        has_death,
        first_death_encounter,
    }
}

/// Short rest without event collection - used by lightweight simulation
fn apply_short_rest_standalone_no_events(players: &[Combattant]) -> Vec<Combattant> {
    let mut updated_players = Vec::new();

    for player in players {
        let mut updated_player = player.clone();

        let mut current_hp = player.final_state.current_hp;
        let mut resources = crate::resources::ResourceLedger::from(player.final_state.resources.clone());

        // 1. Reset Short Rest resources
        resources.reset_by_type(&ResetType::ShortRest);

        // 2. Basic Short Rest healing
        if current_hp < player.creature.hp {
            if current_hp == 0 {
                current_hp = 1; // Wake up
            }
            let max_hp = player.creature.hp;
            let heal_amount = (max_hp / 4).max(1);
            current_hp = (current_hp + heal_amount).min(max_hp);
        }

        // Update state
        let next_state = crate::model::CreatureState {
            current_hp,
            temp_hp: None, // Temp HP lost on rest
            resources: resources.into(),
            ..player.final_state.clone()
        };

        updated_player.initial_state = next_state.clone();
        updated_player.final_state = next_state;

        updated_players.push(updated_player);
    }

    updated_players
}

/// Phase 1: Survey pass - runs all iterations with lightweight simulation (no event collection)
/// Returns all LightweightRun results (~70 KB for 2511 iterations)
pub fn run_survey_pass(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    base_seed: Option<u64>,
) -> Vec<crate::model::LightweightRun> {
    let mut all_runs = Vec::with_capacity(iterations);
    let scenario_hash = crate::cache::get_scenario_hash(&players, &timeline);

    for i in 0..iterations {
        // Use base_seed + i as the seed for this iteration
        let seed = base_seed.unwrap_or(i as u64).wrapping_add(i as u64);

        // Check cache first
        if let Some(cached_run) = crate::cache::get_cached_run(scenario_hash, seed) {
            all_runs.push(cached_run);
            continue;
        }

        let lightweight_run = run_single_lightweight_simulation(&players, &timeline, seed);
        
        // Store in cache
        crate::cache::insert_cached_run(scenario_hash, seed, lightweight_run.clone());
        
        all_runs.push(lightweight_run);
    }

    all_runs
}

/// Phase 2: Analyze lightweight runs and identify interesting seeds for re-simulation
/// Returns a deduplicated set of seeds that should be re-run with full event logging
pub fn select_interesting_seeds(
    lightweight_runs: &[crate::model::LightweightRun],
) -> Vec<u64> {
    use std::collections::HashSet;

    let mut interesting_seeds = HashSet::new();
    let num_encounters = lightweight_runs
        .first()
        .map(|r| r.encounter_scores.len())
        .unwrap_or(0);

    let total_runs = lightweight_runs.len();

    // Helper to get index at percentile
    let get_percentile_index = |len: usize, percentile: f64| -> usize {
        if len == 0 { return 0; }
        let idx = (len as f64 * percentile / 100.0).floor() as usize;
        idx.min(len - 1)
    };

    // 1. GLOBAL PERCENTILES (needed for decile logs and medians)
    let mut global_scored_runs: Vec<(usize, f64)> = lightweight_runs
        .iter()
        .enumerate()
        .map(|(i, run)| (i, run.final_score))
        .collect();
    
    // Sort by global score
    global_scored_runs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // A. The 11 standard decile log seeds (P5, P15, ..., P50, ..., P95)
    let log_indices = if total_runs >= 11 && (total_runs - 1).is_multiple_of(10) {
        // Perfect 10n + 1 system
        let n = (total_runs - 1) / 10;
        vec![
            n / 2,           // 5%
            n + n / 2,       // 15%
            2 * n + n / 2,   // 25%
            3 * n + n / 2,   // 35%
            4 * n + n / 2,   // 45%
            5 * n,           // 50% (True Median)
            5 * n + n / 2 + 1, // 55%
            6 * n + n / 2 + 1, // 65%
            7 * n + n / 2 + 1, // 75%
            8 * n + n / 2 + 1, // 85%
            9 * n + n / 2 + 1, // 95%
        ]
    } else {
        // Fallback for non-perfect counts
        vec![
            (total_runs as f64 * 0.05) as usize,
            (total_runs as f64 * 0.15) as usize,
            (total_runs as f64 * 0.25) as usize,
            (total_runs as f64 * 0.35) as usize,
            (total_runs as f64 * 0.45) as usize,
            (total_runs as f64 * 0.50) as usize,
            (total_runs as f64 * 0.55) as usize,
            (total_runs as f64 * 0.65) as usize,
            (total_runs as f64 * 0.75) as usize,
            (total_runs as f64 * 0.85) as usize,
            (total_runs as f64 * 0.95) as usize,
        ]
    };

    for idx in log_indices {
        if let Some((run_idx, _)) = global_scored_runs.get(idx) {
            interesting_seeds.insert(lightweight_runs[*run_idx].seed);
        }
    }

    // B. The 10 decile medians (used for BattleCards)
    if total_runs >= 11 && (total_runs - 1).is_multiple_of(10) {
        let n = (total_runs - 1) / 10;
        for i in 0..10 {
            let slice_median_idx = if i < 5 { i * n + n / 2 } else { i * n + n / 2 + 1 };
            if let Some((run_idx, _)) = global_scored_runs.get(slice_median_idx) {
                interesting_seeds.insert(lightweight_runs[*run_idx].seed);
            }
        }
    } else {
        let decile_size = total_runs as f64 / 10.0;
        for i in 0..10 {
            let slice_median_idx = ((i as f64 + 0.5) * decile_size).floor() as usize;
            if let Some((run_idx, _)) = global_scored_runs.get(slice_median_idx) {
                interesting_seeds.insert(lightweight_runs[*run_idx].seed);
            }
        }
    }

    // 2. PER-ENCOUNTER PERCENTILES
    for encounter_idx in 0..num_encounters {
        let mut scored_runs: Vec<(usize, f64)> = lightweight_runs
            .iter()
            .enumerate()
            .filter_map(|(i, run)| {
                run.encounter_scores.get(encounter_idx)
                    .copied()
                    .map(|score| (i, score))
            })
            .collect();

        // Sort by score at this encounter
        scored_runs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let len = scored_runs.len();

        // Select seeds at key percentiles: 0, 10, 25, 50, 75, 90, 100
        let percentiles = vec![0.0, 10.0, 25.0, 50.0, 75.0, 90.0, 100.0];
        for percentile in percentiles {
            let idx = get_percentile_index(len, percentile);
            if let Some((run_idx, _)) = scored_runs.get(idx) {
                interesting_seeds.insert(lightweight_runs[*run_idx].seed);
            }
        }
    }

    // 3. EXTREMES & DEATHS
    // Include overall best and worst by final_score (already handled by percentiles but just to be sure)
    if let Some((best_idx, _)) = global_scored_runs.last() {
        interesting_seeds.insert(lightweight_runs[*best_idx].seed);
    }
    if let Some((worst_idx, _)) = global_scored_runs.first() {
        interesting_seeds.insert(lightweight_runs[*worst_idx].seed);
    }

    // Include all runs with deaths (for TPK analysis)
    for run in lightweight_runs {
        if run.has_death {
            interesting_seeds.insert(run.seed);
        }
    }

    // Convert to Vec and return
    interesting_seeds.into_iter().collect()
}

/// Phase 2: Analyze lightweight runs and identify interesting seeds with 1% granularity
/// Returns SelectedSeed objects with tier classification for three-tier Phase 3
/// This is the new version that supports 10,100 runs with 1% bucket granularity
pub fn select_interesting_seeds_with_tiers(
    lightweight_runs: &[crate::model::LightweightRun],
) -> Vec<crate::model::SelectedSeed> {
    use std::collections::HashSet;
    use crate::model::{SelectedSeed, InterestingSeedTier};

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

fn apply_short_rest_standalone(players: &[Combattant], events: &mut Vec<crate::events::Event>) -> Vec<Combattant> {
    let mut updated_players = Vec::new();
    
    for player in players {
        let mut updated_player = player.clone();
        
        let mut current_hp = player.final_state.current_hp;
        let mut resources = crate::resources::ResourceLedger::from(player.final_state.resources.clone());

        // 1. Reset Short Rest resources
        resources.reset_by_type(&ResetType::ShortRest);
        
        // 2. Basic Short Rest healing
        if current_hp < player.creature.hp {
            if current_hp == 0 {
                current_hp = 1; // Wake up
            }
            let max_hp = player.creature.hp;
            let heal_amount = (max_hp / 4).max(1);
            current_hp = (current_hp + heal_amount).min(max_hp);
            
            // Emit recovery events
            events.push(crate::events::Event::ResourceConsumed {
                unit_id: player.id.clone(),
                resource_type: "HitDice".to_string(),
                amount: 1.0,
            });
            events.push(crate::events::Event::HealingApplied {
                target_id: player.id.clone(),
                amount: heal_amount as f64,
                source_id: player.id.clone(),
            });
        }

        // Update state
        let next_state = crate::model::CreatureState {
            current_hp,
            temp_hp: None, // Temp HP lost on rest
            resources: resources.into(),
            ..player.final_state.clone()
        };
        
        updated_player.initial_state = next_state.clone();
        updated_player.final_state = next_state;
        
        updated_players.push(updated_player);
    }
    
    updated_players
}

fn reconstruct_actions(event_history: &[crate::events::Event]) -> HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> {
    let mut actions_by_round_actor: HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> = HashMap::new();
    let mut current_round = 0;
    let mut current_actor_actions: HashMap<String, (String, HashMap<String, i32>)> = HashMap::new();

    for event in event_history {
        match event {
            crate::events::Event::RoundStarted { round_number } => {
                current_round = *round_number;
            },
            crate::events::Event::ActionStarted { actor_id, action_id, .. } => {
                if let Some((prev_action_id, prev_targets)) = current_actor_actions.remove(actor_id) {
                     actions_by_round_actor.entry((current_round, actor_id.clone()))
                        .or_default()
                        .push((prev_action_id, prev_targets));
                }
                current_actor_actions.insert(actor_id.clone(), (action_id.clone(), HashMap::new()));
            },
            crate::events::Event::TurnEnded { unit_id, .. } => {
                if let Some((prev_action_id, prev_targets)) = current_actor_actions.remove(unit_id) {
                     actions_by_round_actor.entry((current_round, unit_id.clone()))
                        .or_default()
                        .push((prev_action_id, prev_targets));
                }
            },
            crate::events::Event::AttackHit { attacker_id, target_id, .. } | 
            crate::events::Event::AttackMissed { attacker_id, target_id, .. } => {
                if let Some((_, targets)) = current_actor_actions.get_mut(attacker_id) {
                    *targets.entry(target_id.clone()).or_insert(0) += 1;
                }
            },
            crate::events::Event::HealingApplied { source_id, target_id, .. } |
            crate::events::Event::BuffApplied { source_id, target_id, .. } |
            crate::events::Event::ConditionAdded { source_id, target_id, .. } => {
                 if let Some((_, targets)) = current_actor_actions.get_mut(source_id) {
                    *targets.entry(target_id.clone()).or_insert(0) += 1;
                }
            },
            _ => {}
        }
    }
    
    for (actor_id, (action_id, targets)) in current_actor_actions {
         actions_by_round_actor.entry((current_round, actor_id))
            .or_default()
            .push((action_id, targets));
    }
    
    actions_by_round_actor
}

fn convert_to_legacy_simulation_result(encounter_result: &crate::execution::EncounterResult, _encounter_idx: usize, target_role: crate::model::TargetRole) -> crate::model::EncounterResult {
    let mut rounds = Vec::new();
    
    // Reconstruct actions from event history
    let actions_by_round_actor = reconstruct_actions(&encounter_result.event_history);

    // Iterate through round snapshots to reconstruct history
    for (round_idx, snapshot) in encounter_result.round_snapshots.iter().enumerate() {
        let mut team1 = Vec::new(); // Players
        let mut team2 = Vec::new(); // Monsters
        let current_round_num = (round_idx + 1) as u32;

        for state in snapshot {
            // Map context::CombattantState to model::CreatureState
            let final_creature_state = crate::model::CreatureState {
                current_hp: state.current_hp,
                temp_hp: Some(state.temp_hp),
                buffs: HashMap::new(), // TODO: Convert active effects to buffs if needed
                resources: state.resources.clone().into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: state.concentration.clone(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
            };

            let mut combatant = state.base_combatant.clone();
            // combatant.creature.hp = state.current_hp; // Removed: creature.hp should remain max HP
            combatant.final_state = final_creature_state;

            // Populate actions for this round
            if let Some(raw_actions) = actions_by_round_actor.get(&(current_round_num, combatant.id.clone())) {
                for (action_id, targets) in raw_actions {
                    if let Some(action) = combatant.creature.actions.iter().find(|a| a.base().id == *action_id) {
                        combatant.actions.push(crate::model::CombattantAction {
                            action: action.clone(),
                            targets: targets.clone(),
                        });
                    }
                }
            }

            // Check side
            let is_player = state.side == 0;

            if is_player {
                team1.push(combatant);
            } else {
                team2.push(combatant);
            }
        }

        rounds.push(crate::model::Round {
            team1,
            team2,
        });
    }

    // If no rounds (e.g. empty encounter), create at least one final state round
    if rounds.is_empty() {
        let mut team1 = Vec::new();
        let mut team2 = Vec::new();
        
        for state in &encounter_result.final_combatant_states {
            let final_creature_state = crate::model::CreatureState {
                current_hp: state.current_hp,
                temp_hp: Some(state.temp_hp),
                buffs: HashMap::new(),
                resources: state.resources.clone().into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: state.concentration.clone(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
            };

            let mut combatant = state.base_combatant.clone();
            // combatant.creature.hp = state.current_hp; // Removed: creature.hp should remain max HP
            combatant.final_state = final_creature_state;
            
            let is_player = state.side == 0;
            if is_player {
                team1.push(combatant);
            } else {
                team2.push(combatant);
            }
        }
        
        rounds.push(crate::model::Round {
            team1,
            team2,
        });
    }

    crate::model::EncounterResult {
        stats: HashMap::new(), // Would convert from encounter_result.statistics
        rounds,
        target_role,
    }
}

fn update_player_states_for_next_encounter(
    players: &[Combattant], 
    encounter_result: &crate::execution::EncounterResult,
    short_rest: bool
) -> Vec<Combattant> {
    // Update players with their final state from the encounter
    let mut updated_players = Vec::new();
    
    for player in players {
        // Find corresponding final state
        if let Some(final_state) = encounter_result.final_combatant_states.iter().find(|s| s.id == player.id) {
             let mut updated_player = player.clone();
             
             let mut current_hp = final_state.current_hp;
             let mut temp_hp = final_state.temp_hp;
             let mut resources = final_state.resources.clone();

             if short_rest {
                 // 1. Reset Short Rest resources
                 resources.reset_by_type(&ResetType::ShortRest);
                 
                 // 2. Basic Short Rest healing (Simplification of Hit Dice)
                 if current_hp == 0 {
                     current_hp = 1; // Wake up
                 }
                 let max_hp = player.creature.hp;
                 let heal_amount = (max_hp / 4).max(1); // Heal 25% of Max HP
                 current_hp = (current_hp + heal_amount).min(max_hp);
                 
                 temp_hp = 0; // Temp HP lost on rest
             }

             // Update state
             updated_player.initial_state = crate::model::CreatureState {
                current_hp,
                temp_hp: if temp_hp > 0 { Some(temp_hp) } else { None },
                buffs: HashMap::new(),
                resources: resources.into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: final_state.concentration.clone(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: final_state.known_ac.clone(),
                arcane_ward_hp: final_state.arcane_ward_hp,
             };
             
             updated_players.push(updated_player);
        } else {
            // Should not happen, but keep original if not found
            updated_players.push(player.clone());
        }
    }
    
    updated_players
}

// Global storage manager for WASM interface
static STORAGE_MANAGER: OnceLock<Mutex<StorageManager>> = OnceLock::new();

/// Initialize or get the global storage manager
fn get_storage_manager() -> &'static Mutex<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| Mutex::new(StorageManager::default()))
}

// Disk storage functions removed as they are no longer needed
// The system now operates purely in RAM

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
    let overall = decile_analysis::run_decile_analysis(&results, scenario_name, actual_party_size);
    
    // 2. Run Per-Encounter Analysis
    // Determine number of encounters from the first result
    let num_encounters = results.first().map(|r| r.encounters.len()).unwrap_or(0);
    let mut encounters = Vec::new();
    
    for i in 0..num_encounters {
        let encounter_name = format!("Encounter {}", i + 1);
        let analysis = decile_analysis::run_encounter_analysis(&results, i, &encounter_name, actual_party_size);
        encounters.push(analysis);
    }
    
    console::log_1(&format!("Generated overall analysis + {} encounter analyses", encounters.len()).into());

    let output = FullAnalysisOutput {
        overall,
        encounters,
    };
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
        
    serde::Serialize::serialize(&output, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize decile analysis: {}", e)))
}

// ===== PHASE 3: GUI INTEGRATION WASM BINDINGS =====

use crate::display_manager::{DisplayManager, DisplayMode, DisplayConfig};
use crate::progress_ui::{ProgressUIManager, ProgressUIConfig};
use crate::user_interaction::{UserInteractionManager, UserEvent, UserInteractionConfig};
#[cfg(not(target_arch = "wasm32"))]
use crate::background_simulation::BackgroundSimulationEngine;
use crate::queue_manager::{QueueManager, QueueManagerConfig};
use crate::storage_integration::StorageIntegration;
use std::sync::Arc;

static GUI_INTEGRATION: OnceLock<Mutex<GuiIntegration>> = OnceLock::new();

/// Combined GUI integration system
#[allow(dead_code)]
struct GuiIntegration {
    display_manager: Arc<Mutex<DisplayManager>>,
    progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
    user_interaction_manager: Arc<Mutex<UserInteractionManager>>,
    storage_integration: Arc<Mutex<StorageIntegration>>,
}

/// Initialize the GUI integration system
#[wasm_bindgen]
pub fn initialize_gui_integration() -> Result<JsValue, JsValue> {
    GUI_INTEGRATION.get_or_init(|| {
        // Get storage manager reference (clone for separate instances as in original code)
        let storage_copy = get_storage_manager().lock().unwrap().clone();

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

        // Create simulation queue for storage integration (separate instance)
        let storage_queue = crate::queue_manager::SimulationQueue::new(100); // max 100 items

        // Create progress communication
        let progress_comm = crate::progress_communication::ProgressCommunication::default();

        // Create storage integration
        let storage_integration = StorageIntegration::new(
            storage_copy.clone(),
            storage_queue,
            progress_comm,
            crate::storage_integration::StorageIntegrationConfig::default(),
        );
        let storage_integration_arc = Arc::new(Mutex::new(storage_integration));

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
            storage_integration: storage_integration_arc,
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
    let gui = get_gui_integration().lock().unwrap();
    
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;
    
    let display_result = {
        let mut display_manager = gui.display_manager.lock().unwrap();
        display_manager.get_display_results(&players, &timeline, iterations)
    };
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&display_result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize display result: {}", e)))
}

/// Set display mode
#[wasm_bindgen]
pub fn set_display_mode(mode_str: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let mode = match mode_str {
        "ShowNewest" => DisplayMode::ShowNewest,
        "ShowMostSimilar" => DisplayMode::ShowMostSimilar,
        "LetUserChoose" => DisplayMode::LetUserChoose,
        "PrimaryOnly" => DisplayMode::PrimaryOnly,
        "SecondaryOnly" => DisplayMode::SecondaryOnly,
        _ => return Err(JsValue::from_str(&format!("Invalid display mode: {}", mode_str))),
    };
    
    let mut display_manager = gui.display_manager.lock().unwrap();
    display_manager.set_display_mode(mode);
    
    Ok(JsValue::from_str(&format!("Display mode set to {:?}", mode)))
}

/// Get current display mode
#[wasm_bindgen]
pub fn get_display_mode() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let display_manager = gui.display_manager.lock().unwrap();
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
    let gui = get_gui_integration().lock().unwrap();
    
    let slot_selection = match slot_str {
        "Primary" => crate::storage::SlotSelection::Primary,
        "Secondary" => crate::storage::SlotSelection::Secondary,
        _ => return Err(JsValue::from_str(&format!("Invalid slot: {}", slot_str))),
    };
    
    let mut display_manager = gui.display_manager.lock().unwrap();
    let display_result = display_manager.user_selected_slot(slot_selection);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
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
    let gui = get_gui_integration().lock().unwrap();
    
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
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let result = user_interaction.handle_event(event);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get progress information for all active simulations
#[wasm_bindgen]
pub fn get_all_progress() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let progress_ui = gui.progress_ui_manager.lock().unwrap();
    let progress_list = progress_ui.get_all_progress();
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&progress_list, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize progress list: {}", e)))
}

/// Get progress information for a specific simulation
#[wasm_bindgen]
pub fn get_progress(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());
    
    let progress_ui = gui.progress_ui_manager.lock().unwrap();
    let progress_info = progress_ui.get_progress(&sim_id);
    
    match progress_info {
        Some(info) => {
            let serializer = serde_wasm_bindgen::Serializer::new()
                .serialize_maps_as_objects(false);
            
            serde::Serialize::serialize(&info, &serializer)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize progress info: {}", e)))
        },
        None => Ok(JsValue::NULL),
    }
}

/// Create HTML progress bar for a simulation
#[wasm_bindgen]
pub fn create_progress_bar(simulation_id: &str) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());
    
    let progress_ui = gui.progress_ui_manager.lock().unwrap();
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
    let gui = get_gui_integration().lock().unwrap();
    
    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());
    
    let progress_ui = gui.progress_ui_manager.lock().unwrap();
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
    let gui = get_gui_integration().lock().unwrap();
    
    let sim_id = crate::background_simulation::BackgroundSimulationId(simulation_id.to_string());
    
    let event = UserEvent::CancelSimulation {
        simulation_id: sim_id,
    };
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let result = user_interaction.handle_event(event);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Clear simulation cache
#[wasm_bindgen]
pub fn clear_simulation_cache_gui() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let event = UserEvent::ClearCache;
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let result = user_interaction.handle_event(event);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get pending user confirmations
#[wasm_bindgen]
pub fn get_pending_confirmations() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let confirmations = user_interaction.get_pending_confirmations();
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&confirmations, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize confirmations: {}", e)))
}

/// Answer a confirmation request
#[wasm_bindgen]
pub fn answer_confirmation(confirmation_id: &str, confirmed: bool) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let result = user_interaction.answer_confirmation(confirmation_id, confirmed);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

/// Get current user interaction state
#[wasm_bindgen]
pub fn get_user_interaction_state() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let state = user_interaction.get_state();
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
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
    let gui = get_gui_integration().lock().unwrap();
    
    // Update display configuration
    if let Some(config_js) = display_config_json {
        let config: DisplayConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse display config: {}", e)))?;
        
        let mut display_manager = gui.display_manager.lock().unwrap();
        display_manager.update_config(config);
    }
    
    // Update progress configuration
    if let Some(config_js) = progress_config_json {
        let config: ProgressUIConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse progress config: {}", e)))?;
        
        let mut progress_ui = gui.progress_ui_manager.lock().unwrap();
        progress_ui.update_config(config);
    }
    
    // Update interaction configuration
    if let Some(config_js) = interaction_config_json {
        let config: UserInteractionConfig = serde_wasm_bindgen::from_value(config_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse interaction config: {}", e)))?;
        
        let mut user_interaction = gui.user_interaction_manager.lock().unwrap();
        user_interaction.update_config(config);
    }
    
    Ok(JsValue::from_str("Configuration updated"))
}

/// Get progress summary for dashboard
#[wasm_bindgen]
pub fn get_progress_summary() -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let progress_ui = gui.progress_ui_manager.lock().unwrap();
    let summary = progress_ui.get_progress_summary();
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
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
    let gui = get_gui_integration().lock().unwrap();
    
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
    
    let user_interaction = gui.user_interaction_manager.lock().unwrap();
    let result = user_interaction.handle_event(event);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
    
    serde::Serialize::serialize(&result, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize event result: {}", e)))
}

// Phase 3 GUI Integration - Simple working interface
pub fn init_phase3_gui_integration() -> crate::phase3_gui_integration::Phase3Integration {
    crate::phase3_gui_integration::init_phase3_gui_integration()
}

// Phase 3 GUI Integration - Working implementation
pub fn init_phase3_gui() -> crate::phase3_working::Phase3Gui {
    crate::phase3_working::init_phase3_gui()
}
