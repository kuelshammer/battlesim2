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

    // Format events for WASM storage (legacy behavior)
    let formatted_events: Vec<String> = if !results.is_empty() {
        // Build name map from first result
        let mut combatant_names = HashMap::new();
        // We need to extract names from the result structure. 
        // Iterating all rounds/combatants is safe enough.
        if let Some(encounter) = results.first().and_then(|r| r.first()) {
             if let Some(round) = encounter.rounds.first() {
                 for c in round.team1.iter().chain(round.team2.iter()) {
                     combatant_names.insert(c.id.clone(), c.creature.name.clone());
                 }
             }
             // Also check final states if no rounds
             // But simpler: we can't easily get ALL names if some died early?
             // Actually, the best way is to replicate the logic inside run_single...
             // But since we are refactoring, let's just format them inside the loop above?
             // No, we returned raw events.
             
             // Let's traverse the result to find names
             for encounter_res in results.first().unwrap() {
                 for round in &encounter_res.rounds {
                     for c in round.team1.iter().chain(round.team2.iter()) {
                         combatant_names.insert(c.id.clone(), c.creature.name.clone());
                     }
                 }
             }
        }
        
        all_events.iter()
            .filter_map(|e| e.format_for_log(&combatant_names))
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

/// Public Rust function for event-driven simulation (for CLI/testing)
/// Returns (results, events) where events are from the first run only
pub fn run_event_driven_simulation_rust(
    players: Vec<Creature>,
    encounters: Vec<Encounter>,
    iterations: usize,
    _log_enabled: bool,
) -> (Vec<SimulationResult>, Vec<crate::events::Event>) {
    let mut all_events = Vec::new();
    let mut results = Vec::new();

    for i in 0..iterations {
        let (result, events) = run_single_event_driven_simulation(&players, &encounters, i == 0);
        results.push(result);

        if i == 0 {
            all_events = events;
        }
    }

    // Sort results by score (worst to best)
    results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    (results, all_events)
}

fn run_single_event_driven_simulation(players: &[Creature], encounters: &[Encounter], _log_enabled: bool) -> (SimulationResult, Vec<crate::events::Event>) {
    let mut all_events = Vec::new();
    let mut players_with_state = Vec::new();

    // Initialize players with state
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
            let mut p = player.clone();
            p.name = name;
            p.mode = "player".to_string(); // Explicitly set mode for team assignment
            let id = format!("{}-{}-{}", player.id, group_idx, i);

            // Create CreatureState
            let state = CreatureState {
                current_hp: p.hp,
                temp_hp: None,
                buffs: HashMap::new(),
                resources: p.initialize_ledger().into(),
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
                id: id.clone(),
                creature: p.clone(),
                initiative: crate::simulation::roll_initiative(&p),
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
                m.mode = "monster".to_string(); // Explicitly set mode for team assignment
                let id = format!("{}-{}-{}", monster.id, group_idx, i);

                let enemy_state = CreatureState {
                    current_hp: m.hp,
                    temp_hp: None,
                    buffs: HashMap::new(),
                    resources: m.initialize_ledger().into(),
                    upcoming_buffs: HashMap::new(),
                    used_actions: HashSet::new(),
                    concentrating_on: None,
                    actions_used_this_encounter: HashSet::new(),
                    bonus_action_used: false,
                    known_ac: HashMap::new(),
                    arcane_ward_hp: None,
                };

                let enemy_combattant = Combattant {
                    id: id.clone(),
                    creature: m.clone(),
                    initiative: crate::simulation::roll_initiative(&m),
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
        let mut engine = ActionExecutionEngine::new(all_combatants.clone());

        // Run encounter using the ActionExecutionEngine
        let encounter_result = engine.execute_encounter();

        // Collect events (raw)
        all_events.extend(encounter_result.event_history.clone());

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

fn reconstruct_actions(event_history: &[crate::events::Event]) -> HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> {
    let mut actions_by_round_actor: HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> = HashMap::new();
    let mut current_round = 0;
    let mut current_actor_actions: HashMap<String, (String, HashMap<String, i32>)> = HashMap::new();

    for event in event_history {
        match event {
            crate::events::Event::RoundStarted { round_number } => {
                current_round = *round_number;
            },
            crate::events::Event::ActionStarted { actor_id, action_id } => {
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
            crate::events::Event::AttackMissed { attacker_id, target_id } => {
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

fn convert_to_legacy_simulation_result(encounter_result: &crate::execution::EncounterResult, _encounter_idx: usize) -> crate::model::EncounterResult {
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
            combatant.creature.hp = state.current_hp; // Update creature HP to current value
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

            // Check mode
            let is_player = state.base_combatant.creature.mode.as_str() == "player";

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
            combatant.creature.hp = state.current_hp; // Update creature HP to current value
            combatant.final_state = final_creature_state;
            
            let is_player = state.base_combatant.creature.mode.as_str() == "player";
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
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
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
