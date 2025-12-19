pub mod dice;
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
pub mod quintile_analysis;
pub mod combat_stats;
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


use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::model::{Creature, Encounter, SimulationResult, Combattant, CreatureState};
use crate::user_interaction::ScenarioParameters;
use crate::execution::ActionExecutionEngine;
use crate::storage_manager::StorageManager;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

#[wasm_bindgen]
pub fn run_simulation_wasm(players: JsValue, encounters: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    let runs = run_event_driven_simulation_rust(players, encounters, iterations, false);

    // Extract results from runs for backward compatibility
    let results: Vec<SimulationResult> = runs.into_iter().map(|run| run.result).collect();

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

#[wasm_bindgen]
pub fn run_simulation_with_callback(
    players: JsValue,
    encounters: JsValue,
    iterations: usize,
    callback: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    let mut all_events = Vec::new();
    let mut results = Vec::new();

    let batch_size = (iterations / 20).max(1); // Report progress every 5%

    for i in 0..iterations {
        let (result, events) = run_single_event_driven_simulation(&players, &encounters, i == 0);
        results.push(result);

        if i == 0 {
            all_events = events;
        }

        if (i + 1) % batch_size == 0 || i == iterations - 1 {
            let progress = (i + 1) as f64 / iterations as f64;
            let this = JsValue::NULL;
            let js_progress = JsValue::from_f64(progress);
            let js_completed = JsValue::from_f64((i + 1) as f64);
            let js_total = JsValue::from_f64(iterations as f64);
            
            let _ = callback.call3(&this, &js_progress, &js_completed, &js_total);
        }
    }

    // Sort results by score (worst to best)
    results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // 1. Run Overall Analysis (Adventure-wide)
    let overall = quintile_analysis::run_quintile_analysis(&results, "Current Scenario", players.len());
    
    // 2. Run Per-Encounter Analysis
    let num_encounters = results.first().map(|r| r.len()).unwrap_or(0);
    let mut encounters_analysis = Vec::new();
    
    for i in 0..num_encounters {
        let encounter_name = format!("Encounter {}", i + 1);
        let analysis = quintile_analysis::run_encounter_analysis(&results, i, &encounter_name, players.len());
        encounters_analysis.push(analysis);
    }

    // 3. Filter results to only keep the 5 representative runs for the UI
    // Indices: 125, 627, 1255, 1882, 2384
    let representative_indices = [125, 627, 1255, 1882, 2384];
    let mut reduced_results = Vec::new();
    
    for &idx in &representative_indices {
        if idx < results.len() {
            reduced_results.push(results[idx].clone());
        }
    }

    #[derive(serde::Serialize)]
    struct FullAnalysisOutput {
        overall: crate::quintile_analysis::AggregateOutput,
        encounters: Vec<crate::quintile_analysis::AggregateOutput>,
    }

    #[derive(serde::Serialize)]
    struct FullSimulationOutput {
        results: Vec<SimulationResult>, // Now only contains 5 representative runs
        analysis: FullAnalysisOutput,
        first_run_events: Vec<crate::events::Event>,
    }

    let output = FullSimulationOutput {
        results: reduced_results,
        analysis: FullAnalysisOutput {
            overall,
            encounters: encounters_analysis,
        },
        first_run_events: all_events,
    };

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&output, &serializer)
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

/// Public Rust function for event-driven simulation (for CLI/testing)
/// Returns all simulation runs with their results and events
pub fn run_event_driven_simulation_rust(
    players: Vec<Creature>,
    encounters: Vec<Encounter>,
    iterations: usize,
    _log_enabled: bool,
) -> Vec<crate::model::SimulationRun> {
    let mut all_runs = Vec::new();

    for i in 0..iterations {
        let (result, events) = run_single_event_driven_simulation(&players, &encounters, i == 0);
        let run = crate::model::SimulationRun {
            result,
            events,
        };
        all_runs.push(run);
    }

    // Sort results by score (worst to best) with safe comparison
    all_runs.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(&a.result);
        let score_b = crate::aggregation::calculate_score(&b.result);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    all_runs
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
                id: id.clone(),
                creature: p.clone(),
                initiative: crate::utilities::roll_initiative(&p),
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
                    id: id.clone(),
                    creature: m.clone(),
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
            // combatant.creature.hp = state.current_hp; // Removed: creature.hp should remain max HP
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

// Global storage manager for WASM interface
static STORAGE_MANAGER: OnceLock<Mutex<StorageManager>> = OnceLock::new();

/// Initialize or get the global storage manager
fn get_storage_manager() -> &'static Mutex<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| Mutex::new(StorageManager::default()))
}

// Disk storage functions removed as they are no longer needed
// The system now operates purely in RAM

#[wasm_bindgen]
pub fn run_quintile_analysis_wasm(results: JsValue, scenario_name: &str, _party_size: usize) -> Result<JsValue, JsValue> {
    // Add debug logging
    console::log_1(&"=== Quintile Analysis WASM Debug ===".into());
    
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
        if let Some(first_encounter) = first_result.first() {
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
    let overall = quintile_analysis::run_quintile_analysis(&results, scenario_name, actual_party_size);
    
    // 2. Run Per-Encounter Analysis
    // Determine number of encounters from the first result
    let num_encounters = results.first().map(|r| r.len()).unwrap_or(0);
    let mut encounters = Vec::new();
    
    for i in 0..num_encounters {
        let encounter_name = format!("Encounter {}", i + 1);
        let analysis = quintile_analysis::run_encounter_analysis(&results, i, &encounter_name, actual_party_size);
        encounters.push(analysis);
    }
    
    console::log_1(&format!("Generated overall analysis + {} encounter analyses", encounters.len()).into());

    #[derive(serde::Serialize)]
    struct FullAnalysisOutput {
        overall: crate::quintile_analysis::AggregateOutput,
        encounters: Vec<crate::quintile_analysis::AggregateOutput>,
    }

    let output = FullAnalysisOutput {
        overall,
        encounters,
    };
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
        
    serde::Serialize::serialize(&output, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize quintile analysis: {}", e)))
}

// ===== PHASE 3: GUI INTEGRATION WASM BINDINGS =====

use crate::display_manager::{DisplayManager, DisplayMode, DisplayConfig};
use crate::progress_ui::{ProgressUIManager, ProgressUIConfig};
use crate::user_interaction::{UserInteractionManager, UserEvent, UserInteractionConfig};
use crate::background_simulation::{BackgroundSimulationEngine, SimulationPriority};
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
        
        // Create background simulation engine
        let (simulation_engine, _progress_receiver) = BackgroundSimulationEngine::new();
        let simulation_engine_arc = Arc::new(Mutex::new(simulation_engine));
        
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
        let storage_manager_arc = Arc::new(Mutex::new(storage_copy.clone()));

        // Create user interaction manager
        let interaction_config = UserInteractionConfig::default();
        let user_interaction_manager = UserInteractionManager::new(
            display_manager_arc.clone(),
            progress_ui_manager_arc.clone(),
            simulation_engine_arc.clone(),
            queue_manager_arc.clone(),
            interaction_config,
        );
        
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
pub fn get_display_results(players: JsValue, encounters: JsValue, iterations: usize) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;
    
    let display_result = {
        let mut display_manager = gui.display_manager.lock().unwrap();
        display_manager.get_display_results(&players, &encounters, iterations)
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
    encounters: JsValue, 
    iterations: usize,
    priority_str: &str
) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;
    
    let priority = match priority_str {
        "Low" => SimulationPriority::Low,
        "Normal" => SimulationPriority::Normal,
        "High" => SimulationPriority::High,
        "Critical" => SimulationPriority::Critical,
        _ => return Err(JsValue::from_str(&format!("Invalid priority: {}", priority_str))),
    };
    
    let event = UserEvent::RequestSimulation {
        parameters: ScenarioParameters {
            players: players.clone(),
            encounters: encounters.clone(),
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
    encounters: JsValue,
    iterations: usize,
) -> Result<JsValue, JsValue> {
    let gui = get_gui_integration().lock().unwrap();
    
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;
    
    let event = UserEvent::ParametersChanged {
        parameters: ScenarioParameters {
            players: players.clone(),
            encounters: encounters.clone(),
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
