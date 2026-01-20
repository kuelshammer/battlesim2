//! WASM API - Thin bindings layer for JavaScript interop
//!
//! This module provides WASM bindings that delegate to the orchestration layer.
//! Keep this file minimal - all business logic belongs in orchestration/.

use crate::model::{Creature, SimulationResult, TimelineStep};
use crate::orchestration::{balancer, runners, simulation};
use wasm_bindgen::prelude::*;

// ============================================================================
// Core Simulation WASM Bindings
// ============================================================================

#[wasm_bindgen]
pub fn auto_adjust_encounter_wasm(
    players: JsValue,
    monsters: JsValue,
    timeline: JsValue,
    encounter_index: usize,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = parse_js_value(players, "players")?;
    let monsters: Vec<Creature> = parse_js_value(monsters, "monsters")?;
    let timeline: Vec<TimelineStep> = parse_js_value(timeline, "timeline")?;

    let result = balancer::run_auto_adjust_encounter_orchestration(
        players,
        monsters,
        timeline,
        encounter_index,
    )
    .map_err(|e| JsValue::from_str(&format!("Auto-adjustment failed: {}", e)))?;

    serialize_result(&result)
}

#[wasm_bindgen]
pub fn init_memory_guardrails() {
    crate::memory_guardrails::init_memory_guardrails();
}

#[wasm_bindgen]
pub fn should_force_lightweight_mode(iterations: usize) -> bool {
    crate::memory_guardrails::should_force_lightweight_mode(iterations)
}

#[wasm_bindgen]
pub fn run_simulation_wasm(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = parse_js_value(players, "players")?;
    let timeline: Vec<TimelineStep> = parse_js_value(timeline, "timeline")?;

    let runs =
        runners::run_event_driven_simulation_rust(players, timeline, iterations, false, None);
    let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

    serialize_result(&results)
}

#[wasm_bindgen]
pub fn run_simulation_with_callback(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
    callback: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = parse_js_value(players, "players")?;
    let timeline: Vec<TimelineStep> = parse_js_value(timeline, "timeline")?;

    let output = simulation::run_simulation_with_callback_orchestration(
        players, timeline, iterations, callback,
    )
    .map_err(|e| JsValue::from_str(&format!("Simulation failed: {}", e)))?;

    serialize_result(&output)
}

#[wasm_bindgen]
pub fn run_event_driven_simulation(
    players: JsValue,
    timeline: JsValue,
    iterations: usize,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = parse_js_value(players, "players")?;
    let timeline: Vec<TimelineStep> = parse_js_value(timeline, "timeline")?;

    let runs = runners::run_event_driven_simulation_rust(players, timeline, iterations, true, None);
    let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

    serialize_result(&results)
}

#[wasm_bindgen]
pub fn get_last_simulation_events() -> Result<JsValue, JsValue> {
    let events = crate::orchestration::state::get_stored_simulation_events();
    match events {
        Some(events) => serialize_result(&events),
        None => Ok(JsValue::from_str("No simulation events stored")),
    }
}

#[wasm_bindgen]
pub fn clear_simulation_cache() {
    crate::cache::clear_cache();
}

#[wasm_bindgen]
pub fn get_cache_stats() -> JsValue {
    let (entry_count, estimated_bytes) = crate::cache::get_cache_stats();
    JsValue::from_str(
        &serde_json::json!({
            "entryCount": entry_count,
            "estimatedBytes": estimated_bytes
        })
        .to_string(),
    )
}

#[wasm_bindgen]
pub fn run_skyline_analysis_wasm(
    results: JsValue,
    party_size: usize,
    encounter_index: Option<usize>,
) -> Result<JsValue, JsValue> {
    let results: Vec<SimulationResult> = parse_js_value(results, "results")?;

    let analysis =
        simulation::run_skyline_analysis_orchestration(results, party_size, encounter_index)
            .map_err(|e| JsValue::from_str(&format!("Skyline analysis failed: {}", e)))?;

    serialize_result(&analysis)
}

#[wasm_bindgen]
pub fn run_decile_analysis_wasm(
    results: JsValue,
    scenario_name: &str,
    party_size: usize,
) -> Result<JsValue, JsValue> {
    let results: Vec<SimulationResult> = parse_js_value(results, "results")?;

    let analysis =
        simulation::run_decile_analysis_orchestration(results, scenario_name, party_size)
            .map_err(|e| JsValue::from_str(&format!("Decile analysis failed: {}", e)))?;

    serialize_result(&analysis)
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_js_value<T: serde::de::DeserializeOwned>(
    value: JsValue,
    name: &str,
) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse {}: {}", name, e)))
}

fn serialize_result<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    serde::Serialize::serialize(value, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Report progress via callback
fn report_progress(callback: &js_sys::Function, progress: f64, completed: f64, total: f64) {
    let this = JsValue::NULL;
    let _ = callback.call4(
        &this,
        &JsValue::from_f64(progress),
        &JsValue::from_f64(completed),
        &JsValue::from_f64(total),
        &JsValue::NULL,
    );
}
