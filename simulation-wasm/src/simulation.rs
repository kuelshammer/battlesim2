//! Core simulation execution functions
//!
//! This module contains the main simulation logic for running event-driven
//! and lightweight simulations. These functions handle:
//! - Single simulation execution with full event collection
//! - Lightweight survey pass simulations for Phase 1 of Two-Pass system
//! - Player state management across encounters
//! - Legacy format conversion for backward compatibility

use crate::model::{Creature, SimulationResult, Combattant, CreatureState};
use crate::execution::ActionExecutionEngine;
use crate::context::TurnContext;
use crate::resources::{ResetType, ResourceType};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Create a CreatureState from a Creature, initializing all resources and state
///
/// This helper function creates a CreatureState with:
/// - Current HP set to max HP
/// - Empty temp HP
/// - Empty buffs and upcoming buffs
/// - Initialized resources (including per-action uses)
/// - Empty action tracking sets
/// - No concentration
/// - No known AC values
/// - No arcane ward
fn create_creature_state(creature: &Creature) -> CreatureState {
    CreatureState {
        current_hp: creature.hp,
        temp_hp: None,
        buffs: HashMap::new(),
        resources: {
            let mut r = crate::model::SerializableResourceLedger::from(creature.initialize_ledger());
            // Initialize per-action resources (1/fight, 1/day, Limited, Recharge)
            let action_uses = crate::actions::get_remaining_uses(creature, "long rest", None);
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
        // Initialize Arcane Ward HP to max at encounter start
        arcane_ward_hp: creature.max_arcane_ward_hp,
        cumulative_spent: 0.0,
    }
}

/// Create a player Combattant from a Creature template
///
/// This helper function creates a player Combattant with:
/// - Properly formatted name (with count suffix if count > 1)
/// - Unique ID prefixed with 'p-' to ensure uniqueness across encounters
/// - Team 0 (players)
/// - Initialized CreatureState
/// - Rolled initiative
///
/// # Arguments
/// * `group_idx` - Index of the player group in the players array
/// * `i` - Instance index (0 to count-1)
/// * `player` - The Creature template to create from
///
/// # Returns
/// A fully initialized Combattant ready for simulation
fn create_player_combatant(group_idx: usize, i: i32, player: &Creature) -> Combattant {
    let name = if player.count > 1.0 {
        format!("{} {}", player.name, i + 1)
    } else {
        player.name.clone()
    };
    let mut p = player.clone();
    p.name = name;
    p.mode = "player".to_string();
    let id = format!("p-{}-{}-{}", group_idx, i, player.id);

    let state = create_creature_state(&p);

    Combattant {
        team: 0,
        id: id.clone(),
        creature: Arc::new(p.clone()),
        initiative: crate::utilities::roll_initiative(&p),
        initial_state: state.clone(),
        final_state: state,
        actions: Vec::new(),
    }
}

/// Create an enemy Combattant from a Creature template
///
/// This helper function creates an enemy Combattant with:
/// - Properly formatted name (with count suffix if count > 1)
/// - Unique ID including step index to be globally unique
/// - Team 1 (enemies)
/// - Initialized CreatureState
/// - Rolled initiative
///
/// # Arguments
/// * `step_idx` - Index of the timeline step (encounter)
/// * `group_idx` - Index of the monster group in the encounter
/// * `i` - Instance index (0 to count-1)
/// * `monster` - The Creature template to create from
///
/// # Returns
/// A fully initialized Combattant ready for simulation
fn create_enemy_combatant(step_idx: usize, group_idx: usize, i: i32, monster: &Creature) -> Combattant {
    let name = if monster.count > 1.0 {
        format!("{} {}", monster.name, i + 1)
    } else {
        monster.name.clone()
    };
    let mut m = monster.clone();
    m.name = name;
    let id = format!("step{}-m-{}-{}-{}", step_idx, group_idx, i, monster.id);

    let state = create_creature_state(&m);

    Combattant {
        team: 1,
        id: id.clone(),
        creature: Arc::new(m.clone()),
        initiative: crate::utilities::roll_initiative(&m),
        initial_state: state.clone(),
        final_state: state,
        actions: Vec::new(),
    }
}

/// Initialize players with state - IDs are prefixed with 'p-' to ensure they are unique
/// and carried over correctly across encounters.
fn initialize_players(players: &[Creature]) -> Vec<Combattant> {
    let mut players_with_state = Vec::new();
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let combattant = create_player_combatant(group_idx, i, player);
            players_with_state.push(combattant);
        }
    }
    players_with_state
}

/// Initialize enemies for a specific encounter - IDs include encounter index to be globally unique
fn initialize_enemies(step_idx: usize, monsters: &[Creature]) -> Vec<Combattant> {
    let mut enemies = Vec::new();
    for (group_idx, monster) in monsters.iter().enumerate() {
        for i in 0..monster.count as i32 {
            let enemy_combattant = create_enemy_combatant(step_idx, group_idx, i, monster);
            enemies.push(enemy_combattant);
        }
    }
    enemies
}

/// Run a single event-driven simulation with full event collection
///
/// This function executes a complete timeline (combat encounters and short rests)
/// and returns both the simulation result and all generated events.
///
/// # Arguments
/// * `players` - Player creatures participating in the simulation
/// * `timeline` - Timeline of encounters and rest steps
/// * `_log_enabled` - Whether to enable logging (unused, kept for compatibility)
///
/// # Returns
/// A tuple of (SimulationResult, Vec<Event>) containing the result and all events
pub fn run_single_event_driven_simulation(
    players: &[Creature],
    timeline: &[crate::model::TimelineStep],
    _log_enabled: bool,
) -> (SimulationResult, Vec<crate::events::Event>) {
    // Get the current RNG seed that was set before calling this function
    // This ensures the seed is preserved in the result for reproducibility
    let seed = crate::rng::get_current_seed();
    let mut all_events = Vec::new();
    let mut players_with_state = initialize_players(players);

    let mut encounter_results = Vec::new();
    let num_combat_encounters = timeline
        .iter()
        .filter(|step| matches!(step, crate::model::TimelineStep::Combat(_)))
        .count();

    for (step_idx, step) in timeline.iter().enumerate() {
        match step {
            crate::model::TimelineStep::Combat(encounter) => {
                // Create enemy combatants - IDs include encounter index to be globally unique
                let enemies = initialize_enemies(step_idx, &encounter.monsters);

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
                let legacy_result = convert_to_legacy_simulation_result(
                    &encounter_result,
                    step_idx,
                    encounter.target_role.clone(),
                );
                encounter_results.push(legacy_result);

                // Update player states for next encounter (no rest here, rest is its own step)
                players_with_state =
                    update_player_states_for_next_encounter(&players_with_state, &encounter_result, false);
            }
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
        num_combat_encounters,
        seed,
    };

    // Calculate efficiency score
    let score = crate::aggregation::calculate_efficiency_score(&result, &all_events);
    result.score = Some(score);

    (result, all_events)
}

/// Lightweight simulation that tracks only scores and deaths, no event collection
///
/// Used in Phase 1 of Two-Pass system to identify interesting runs for re-simulation.
/// This function runs a complete simulation but without collecting events or snapshots,
/// resulting in significantly lower memory usage.
///
/// # Arguments
/// * `players` - Player creatures participating in the simulation
/// * `timeline` - Timeline of encounters and rest steps
/// * `seed` - RNG seed for deterministic results
///
/// # Returns
/// A `LightweightRun` containing only the essential statistics
pub fn run_single_lightweight_simulation(
    players: &[Creature],
    timeline: &[crate::model::TimelineStep],
    seed: u64,
) -> crate::model::LightweightRun {
    // Seed RNG for deterministic results
    crate::rng::seed_rng(seed);

    let mut encounter_scores = Vec::new();
    let mut has_death = false;
    let mut first_death_encounter: Option<usize> = None;

    let mut players_with_state = initialize_players(players);

    for (step_idx, step) in timeline.iter().enumerate() {
        match step {
            crate::model::TimelineStep::Combat(encounter) => {
                // Create enemy combatants - IDs include encounter index to be globally unique
                let enemies = initialize_enemies(step_idx, &encounter.monsters);

                // Combine all combatants for this encounter
                let mut all_combatants = players_with_state.clone();
                all_combatants.extend(enemies);

                // Create ActionExecutionEngine
                let mut engine = ActionExecutionEngine::new(all_combatants.clone(), false);

                // Run encounter using the ActionExecutionEngine (NO event collection, NO snapshots)
                let encounter_result = engine.execute_encounter();

                // Track cumulative score after this combat encounter
                let score = crate::safe_aggregation::calculate_lightweight_score(
                    &encounter_result.final_combatant_states,
                );
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
                players_with_state = update_player_states_for_next_encounter(
                    &players_with_state,
                    &encounter_result,
                    false,
                );
            }
            crate::model::TimelineStep::ShortRest(_) => {
                // Apply standalone short rest recovery (NO event collection)
                players_with_state = apply_short_rest_standalone_no_events(&players_with_state);
            }
        }
    }

    // Calculate final stats from the last state of players
    let total_survivors = players_with_state
        .iter()
        .filter(|p| p.final_state.current_hp > 0)
        .count();

    // final_score is the score of the last completed encounter
    let final_score = encounter_scores.last().copied().unwrap_or(0.0);

    // total_hp_lost is (Total Daily Net Worth EHP) - (Final EHP)
    let sr_count = timeline.iter().filter(|s| matches!(s, crate::model::TimelineStep::ShortRest(_))).count();
    let tdnw = crate::decile_analysis::calculate_tdnw_lightweight(players, sr_count);
    let mut final_ehp = 0.0;
    for p in &players_with_state {
        let ledger = p.creature.initialize_ledger();
        final_ehp += crate::intensity_calculation::calculate_serializable_ehp(
            p.final_state.current_hp,
            p.final_state.temp_hp.unwrap_or(0),
            &p.final_state.resources,
            &ledger.reset_rules,
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
        let mut cumulative_spent = player.final_state.cumulative_spent;
        let mut resources = crate::resources::ResourceLedger::from(player.final_state.resources.clone());

        // 1. Reset Short Rest resources
        resources.reset_by_type(&ResetType::ShortRest);

        // 2. Iterative Hit Dice healing
        let max_hp = player.creature.hp;
        let con_mod = player.creature.con_modifier.unwrap_or(0.0);

        // Wake up if at 0
        if current_hp == 0 {
            current_hp = 1;
        }

        // Find available hit dice
        let hd_types = [
            ResourceType::HitDiceD12,
            ResourceType::HitDiceD10,
            ResourceType::HitDiceD8,
            ResourceType::HitDiceD6,
        ];

        while current_hp < max_hp {
            let mut used_die = false;
            
            for hd_type in &hd_types {
                if resources.has(hd_type.clone(), None, 1.0) {
                    let sides = match hd_type {
                        ResourceType::HitDiceD12 => 12.0,
                        ResourceType::HitDiceD10 => 10.0,
                        ResourceType::HitDiceD8 => 8.0,
                        ResourceType::HitDiceD6 => 6.0,
                        _ => 8.0,
                    };
                    
                    let avg_roll = (sides + 1.0) / 2.0;
                    let total_avg = avg_roll + con_mod;

                    // Stop if using another hit die would overflow max HP
                    if current_hp as f64 + total_avg > max_hp as f64 {
                        break;
                    }

                    // Consume die and apply healing
                    if resources.consume(hd_type.clone(), None, 1.0).is_ok() {
                        current_hp = (current_hp as f64 + total_avg).round() as u32;
                        current_hp = current_hp.min(max_hp);
                        
                        let weight = crate::intensity_calculation::get_hit_die_average(&hd_type.to_key(None), con_mod);
                        cumulative_spent += weight;
                        
                        used_die = true;
                        break;
                    }
                }
            }

            if !used_die {
                break;
            }
        }

        // Update state
        let next_state = crate::model::CreatureState {
            current_hp,
            temp_hp: None, // Temp HP lost on rest
            resources: resources.into(),
            cumulative_spent,
            ..player.final_state.clone()
        };

        updated_player.initial_state = next_state.clone();
        updated_player.final_state = next_state;

        updated_players.push(updated_player);
    }

    updated_players
}

/// Apply short rest with event collection
fn apply_short_rest_standalone(
    players: &[Combattant],
    events: &mut Vec<crate::events::Event>,
) -> Vec<Combattant> {
    let mut updated_players = Vec::new();

    // Create a temporary context to handle unified healing and event emission
    let mut context = TurnContext::new(players.to_vec(), vec![], None, "Rest".to_string(), true);

    for player in players {
        // 1. Reset Short Rest resources
        if let Some(c) = context.get_combatant_mut(&player.id) {
            c.resources.reset_by_type(&ResetType::ShortRest);
        }

        // 2. Iterative Hit Dice healing
        let mut current_hp = player.final_state.current_hp;
        let max_hp = player.creature.hp;
        let con_mod = player.creature.con_modifier.unwrap_or(0.0);

        // Wake up if at 0
        if current_hp == 0 {
            if let Some(c) = context.get_combatant_mut(&player.id) {
                c.current_hp = 1;
                current_hp = 1;
            }
        }

        // Find available hit dice
        let hd_types = [
            ResourceType::HitDiceD12,
            ResourceType::HitDiceD10,
            ResourceType::HitDiceD8,
            ResourceType::HitDiceD6,
        ];

        while current_hp < max_hp {
            let mut used_die = false;
            
            for hd_type in &hd_types {
                let mut heal_to_apply = None;
                
                if let Some(c) = context.get_combatant_mut(&player.id) {
                    if c.resources.has(hd_type.clone(), None, 1.0) {
                        let sides = match hd_type {
                            ResourceType::HitDiceD12 => 12.0,
                            ResourceType::HitDiceD10 => 10.0,
                            ResourceType::HitDiceD8 => 8.0,
                            ResourceType::HitDiceD6 => 6.0,
                            _ => 8.0,
                        };
                        
                        let avg_roll = (sides + 1.0) / 2.0;
                        let total_avg = avg_roll + con_mod;

                        // Stop if using another hit die would overflow max HP
                        if current_hp as f64 + total_avg > max_hp as f64 {
                            // Break inner loop, but we need to check other die types? 
                            // Usually all hit dice are same or similar, but let's be safe.
                            continue; 
                        }

                        // Consume die
                        if c.resources.consume(hd_type.clone(), None, 1.0).is_ok() {
                            heal_to_apply = Some(total_avg);
                        }
                    }
                }

                if let Some(amount) = heal_to_apply {
                    context.apply_healing(&player.id, amount, false, &player.id);
                    
                    // Add resource consumption event
                    context.record_event(crate::events::Event::ResourceConsumed {
                        unit_id: player.id.clone(),
                        resource_type: hd_type.to_key(None),
                        amount: 1.0,
                    });

                    // Track cumulative expenditure for strategic attrition model
                    if let Some(c) = context.get_combatant_mut(&player.id) {
                        let weight = crate::intensity_calculation::get_hit_die_average(&hd_type.to_key(None), con_mod);
                        c.cumulative_spent += weight;
                    }
                    
                    // Update local current_hp from context
                    if let Some(c) = context.get_combatant(&player.id) {
                        current_hp = c.current_hp;
                    }
                    
                    used_die = true;
                    break;
                }
            }

            if !used_die {
                break;
            }
        }

        // Get updated state back from context
        if let Some(c_state) = context.get_combatant(&player.id) {
            let mut updated_player = player.clone();
            let next_state = crate::model::CreatureState {
                current_hp: c_state.current_hp,
                temp_hp: None, // Temp HP lost on rest
                resources: c_state.resources.clone().into(),
                cumulative_spent: c_state.cumulative_spent,
                ..player.final_state.clone()
            };
            updated_player.initial_state = next_state.clone();
            updated_player.final_state = next_state;
            updated_players.push(updated_player);
        }
    }

    // Collect all events from temporary context
    events.extend(context.event_bus.get_all_events().to_vec());

    updated_players
}

/// Update player states for the next encounter
///
/// Extracts final states from the encounter result and applies them to players
/// for the next encounter in the timeline.
fn update_player_states_for_next_encounter(
    players: &[Combattant],
    encounter_result: &crate::execution::EncounterResult,
    short_rest: bool,
) -> Vec<Combattant> {
    // Update players with their final state from the encounter
    let mut updated_players = Vec::new();

    for player in players {
        // Find corresponding final state
        if let Some(final_state) = encounter_result
            .final_combatant_states
            .iter()
            .find(|s| s.id == player.id)
        {
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
            let next_state = crate::model::CreatureState {
                current_hp,
                temp_hp: if temp_hp > 0 {
                    Some(temp_hp)
                } else {
                    None
                },
                buffs: HashMap::new(),
                resources: resources.into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: final_state.concentration.clone(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: final_state.known_ac.clone(),
                arcane_ward_hp: final_state.arcane_ward_hp,
                cumulative_spent: final_state.cumulative_spent,
            };

            updated_player.initial_state = next_state.clone();
            updated_player.final_state = next_state;

            updated_players.push(updated_player);
        } else {
            // Should not happen, but keep original if not found
            updated_players.push(player.clone());
        }
    }

    updated_players
}

/// Reconstruct actions from event history
///
/// Parses the event history to rebuild which actions each combatant took
/// in each round, including target information.
fn reconstruct_actions(
    event_history: &[crate::events::Event],
) -> HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> {
    let mut actions_by_round_actor: HashMap<(u32, String), Vec<(String, HashMap<String, i32>)>> =
        HashMap::new();
    let mut current_round = 0;
    let mut current_actor_actions: HashMap<String, (String, HashMap<String, i32>)> = HashMap::new();

    for event in event_history {
        match event {
            crate::events::Event::RoundStarted { round_number } => {
                current_round = *round_number;
            }
            crate::events::Event::ActionStarted {
                actor_id,
                action_id,
                ..
            } => {
                if let Some((prev_action_id, prev_targets)) =
                    current_actor_actions.remove(actor_id)
                {
                    actions_by_round_actor
                        .entry((current_round, actor_id.clone()))
                        .or_default()
                        .push((prev_action_id, prev_targets));
                }
                current_actor_actions.insert(actor_id.clone(), (action_id.clone(), HashMap::new()));
            }
            crate::events::Event::TurnEnded { unit_id, .. } => {
                if let Some((prev_action_id, prev_targets)) = current_actor_actions.remove(unit_id) {
                    actions_by_round_actor
                        .entry((current_round, unit_id.clone()))
                        .or_default()
                        .push((prev_action_id, prev_targets));
                }
            }
            crate::events::Event::AttackHit {
                attacker_id,
                target_id,
                ..
            }
            | crate::events::Event::AttackMissed {
                attacker_id,
                target_id,
                ..
            } => {
                if let Some((_, targets)) = current_actor_actions.get_mut(attacker_id) {
                    *targets.entry(target_id.clone()).or_insert(0) += 1;
                }
            }
            crate::events::Event::HealingApplied {
                source_id,
                target_id,
                ..
            }
            | crate::events::Event::BuffApplied {
                source_id,
                target_id,
                ..
            }
            | crate::events::Event::ConditionAdded {
                source_id,
                target_id,
                ..
            } => {
                if let Some((_, targets)) = current_actor_actions.get_mut(source_id) {
                    *targets.entry(target_id.clone()).or_insert(0) += 1;
                }
            }
            _ => {}
        }
    }

    for (actor_id, (action_id, targets)) in current_actor_actions {
        actions_by_round_actor
            .entry((current_round, actor_id))
            .or_default()
            .push((action_id, targets));
    }

    actions_by_round_actor
}

/// Convert execution result to legacy simulation result format
///
/// This function converts the new ActionExecutionEngine output format
/// to the legacy EncounterResult format for backward compatibility.
fn convert_to_legacy_simulation_result(
    encounter_result: &crate::execution::EncounterResult,
    _encounter_idx: usize,
    target_role: crate::model::TargetRole,
) -> crate::model::EncounterResult {
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
                cumulative_spent: state.cumulative_spent,
            };

            let mut combatant = state.base_combatant.clone();
            // combatant.creature.hp = state.current_hp; // Removed: creature.hp should remain max HP
            combatant.final_state = final_creature_state;

            // Populate actions for this round
            if let Some(raw_actions) =
                actions_by_round_actor.get(&(current_round_num, combatant.id.clone()))
            {
                for (action_id, targets) in raw_actions {
                    if let Some(action) = combatant
                        .creature
                        .actions
                        .iter()
                        .find(|a| a.base().id == *action_id)
                    {
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

        rounds.push(crate::model::Round { team1, team2 });
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
                cumulative_spent: state.cumulative_spent,
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

        rounds.push(crate::model::Round { team1, team2 });
    }

    crate::model::EncounterResult {
        stats: HashMap::new(), // Would convert from encounter_result.statistics
        rounds,
        target_role,
    }
}

/// Phase 1: Survey pass - runs all iterations with lightweight simulation (no event collection)
///
/// Returns all `LightweightRun` results (~323 KB for 10,100 iterations with 1% granularity).
/// This is the main entry point for Phase 1 of the Two-Pass system.
///
/// # Arguments
/// * `players` - Player creatures participating in the simulation
/// * `timeline` - Timeline of encounters and rest steps
/// * `iterations` - Number of iterations to run
/// * `base_seed` - Optional base seed for deterministic results
///
/// # Returns
/// Vector of `LightweightRun` results with scores and death tracking
pub fn run_survey_pass(
    players: Vec<Creature>,
    timeline: Vec<crate::model::TimelineStep>,
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
