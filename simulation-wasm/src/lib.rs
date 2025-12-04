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
use std::sync::Mutex;

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

// Store last simulation events for retrieval (thread-safe)
static LAST_SIMULATION_EVENTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

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

    // Store events for retrieval (thread-safe)
    if let Ok(mut events_guard) = LAST_SIMULATION_EVENTS.lock() {
        *events_guard = Some(all_events.clone());
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
                resources: crate::resources::ResourceLedger::new().into(),
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
                    resources: crate::resources::ResourceLedger::new().into(),
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

fn convert_to_legacy_simulation_result(encounter_result: &crate::execution::EncounterResult, _encounter_idx: usize) -> crate::model::EncounterResult {
    // Convert final states to Combattants
    let mut team1 = Vec::new(); // Players
    let mut team2 = Vec::new(); // Monsters
    
    // We need to reconstruct Combattants from CombattantStates
    // Note: This is imperfect because we don't have the original Creature definitions easily accessible here
    // unless we pass them down. But CombattantState has base_combatant!
    
    for state in &encounter_result.final_combatant_states {
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
        };

        let mut combatant = state.base_combatant.clone();
        combatant.final_state = final_creature_state;
        
        // Determine team based on ID (hacky but works for now)
        // Players have IDs like "player-..." or just UUIDs
        // Monsters have IDs like "monster-..."
        // Or we can check if it was in the original players list?
        // For now, let's assume if it has "Monster" in name or ID it's team 2?
        // Actually, base_combatant.creature.mode tells us!
        
        // Check mode
        // Note: Creature struct has 'mode' field
        let is_monster = match state.base_combatant.creature.mode.as_str() {
            "monster" => true,
            _ => false,
        };

        if is_monster {
            team2.push(combatant);
        } else {
            team1.push(combatant);
        }
    }

    // Create a single "Final Round" to represent the end state
    let final_round = crate::model::Round {
        team1,
        team2,
    };

    crate::model::EncounterResult {
        stats: HashMap::new(), // Would convert from encounter_result.statistics
        rounds: vec![final_round],
    }
}

fn update_player_states_for_next_encounter(players: &[Combattant], encounter_result: &crate::execution::EncounterResult) -> Vec<Combattant> {
    // Update players with their final state from the encounter
    let mut updated_players = Vec::new();
    
    for player in players {
        // Find corresponding final state
        if let Some(final_state) = encounter_result.final_combatant_states.iter().find(|s| s.id == player.id) {
             let mut updated_player = player.clone();
             
             // Update state
             updated_player.initial_state = crate::model::CreatureState {
                current_hp: final_state.current_hp,
                temp_hp: Some(final_state.temp_hp),
                buffs: HashMap::new(),
                resources: final_state.resources.clone().into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: final_state.concentration.clone(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
             };
             
             updated_players.push(updated_player);
        } else {
            // Should not happen, but keep original if not found
            updated_players.push(player.clone());
        }
    }
    
    updated_players
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
