pub mod dice;
pub mod actions;
pub mod targeting;
pub mod enums;
pub mod model;
pub mod aggregation;
pub mod cleanup;
pub mod resolution;
pub mod simulation;
pub mod resources;


use wasm_bindgen::prelude::*;
use crate::model::{Creature, Encounter, SimulationResult};

#[wasm_bindgen]
pub fn run_simulation_wasm(players: JsValue, encounters: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    let results = simulation::run_monte_carlo(&players, &encounters, iterations);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
        
    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn aggregate_simulation_results(results: JsValue) -> Result<JsValue, JsValue> {
    let results: Vec<SimulationResult> = serde_wasm_bindgen::from_value(results)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse results: {}", e)))?;
        
    let aggregated = aggregation::aggregate_results(&results);
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
        
    serde::Serialize::serialize(&aggregated, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize aggregated results: {}", e)))
}
