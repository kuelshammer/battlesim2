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
pub mod events;
pub mod context;
pub mod reactions;
pub mod execution;
pub mod action_resolver;
pub mod validation; // New module for requirement validation


use wasm_bindgen::prelude::*;
use crate::model::{Creature, Encounter, SimulationResult, Combattant, CreatureState};
use crate::execution::ActionExecutionEngine;
use std::collections::{HashMap, HashSet};

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

// Store last simulation events for retrieval
static mut LAST_SIMULATION_EVENTS: Option<Vec<String>> = None;

#[wasm_bindgen]
pub fn run_event_driven_simulation(players: JsValue, encounters: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    let mut all_events = Vec::new();
    let mut results = Vec::new();

    for i in 0..iterations {
        let (result, events) = run_single_event_driven_simulation(&players, &encounters, i == 0);
        results.push(result);

        if i == 0 {
            all_events = events;
        }
    }

    // Store events for retrieval
    unsafe {
        LAST_SIMULATION_EVENTS = Some(all_events.clone());
    }

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn get_last_simulation_events() -> Result<JsValue, JsValue> {
    unsafe {
        match &LAST_SIMULATION_EVENTS {
            Some(events) => {
                let serializer = serde_wasm_bindgen::Serializer::new()
                    .serialize_maps_as_objects(false);
                serde::Serialize::serialize(&events, &serializer)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
            }
            None => Ok(JsValue::from_str(&"No simulation events available")),
        }
    }
}

fn run_single_event_driven_simulation(players: &[Creature], encounters: &[Encounter], _log_enabled: bool) -> (SimulationResult, Vec<String>) {
    let mut all_events = Vec::new();
    let mut players_with_state = Vec::new();

    // Initialize players with state
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
            let mut p = player.clone();
            p.name = name;
            let id = format!("{}-{}-{}", player.id, group_idx, i);

            // Create CreatureState
            let state = CreatureState {
                current_hp: p.hp,
                temp_hp: None,
                buffs: HashMap::new(),
                remaining_uses: HashMap::new(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: None,
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
            };

            // Create Combattant for ActionExecutionEngine
            let combattant = Combattant {
                id: id.clone(),
                creature: p.clone(),
                initiative: 10.0, // Default initiative
                initial_state: state.clone(),
                final_state: state,
                actions: Vec::new(),
            };

            players_with_state.push(combattant);
        }
    }

    let mut encounter_results = Vec::new();

    for (encounter_idx, encounter) in encounters.iter().enumerate() {
        // Create enemy combatants
        let mut enemies = Vec::new();
        for (group_idx, monster) in encounter.monsters.iter().enumerate() {
            for i in 0..monster.count as i32 {
                let name = if monster.count > 1.0 { format!("{} {}", monster.name, i + 1) } else { monster.name.clone() };
                let mut m = monster.clone();
                m.name = name;
                let id = format!("{}-{}-{}", monster.id, group_idx, i);

                let enemy_state = CreatureState {
                    current_hp: m.hp,
                    temp_hp: None,
                    buffs: HashMap::new(),
                    remaining_uses: HashMap::new(),
                    upcoming_buffs: HashMap::new(),
                    used_actions: HashSet::new(),
                    concentrating_on: None,
                    actions_used_this_encounter: HashSet::new(),
                    bonus_action_used: false,
                };

                let enemy_combattant = Combattant {
                    id: id.clone(),
                    creature: m.clone(),
                    initiative: 10.0, // Default initiative
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
        let mut engine = ActionExecutionEngine::new(all_combatants);

        // Run encounter using the ActionExecutionEngine
        let encounter_result = engine.execute_encounter();

        // Collect events from this encounter
        let encounter_events: Vec<String> = encounter_result.event_history
            .iter()
            .map(|event| format!("{:?}", event))
            .collect();

        all_events.extend(encounter_events);

        // Convert to old format for compatibility
        let legacy_result = convert_to_legacy_simulation_result(&encounter_result, encounter_idx);
        encounter_results.push(legacy_result);

        // Update player states for next encounter (simple recovery)
        if encounter_idx < encounters.len() - 1 {
            // This is simplified - in a full implementation would handle short/long rests
            players_with_state = update_player_states_for_next_encounter(&players_with_state, &encounter_result);
        }
    }

    // SimulationResult is just Vec<EncounterResult>
    (encounter_results, all_events)
}

fn convert_to_legacy_simulation_result(_encounter_result: &crate::execution::EncounterResult, _encounter_idx: usize) -> crate::model::EncounterResult {
    // This converts the new ActionExecutionEngine result to the old format for compatibility
    // This is a simplified conversion - in a full implementation would preserve more details

    crate::model::EncounterResult {
        stats: HashMap::new(), // Would convert from encounter_result.statistics
        rounds: Vec::new(), // Would convert from encounter_result.turn_results
    }
}

fn update_player_states_for_next_encounter(players: &[Combattant], _encounter_result: &crate::execution::EncounterResult) -> Vec<Combattant> {
    // Simple implementation - just return the same players
    // In a full implementation would handle rest mechanics, HP recovery, etc.
    players.to_vec()
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
