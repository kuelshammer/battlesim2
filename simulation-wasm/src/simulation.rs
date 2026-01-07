//! Internal simulation state and conversion helpers

use crate::model::{Combattant, CreatureState};
use crate::context::TurnContext;
use crate::resources::{ResetType, ResourceType};
use std::collections::{HashMap, HashSet};

/// Short rest without event collection - used by lightweight simulation
pub(crate) fn apply_short_rest_standalone_no_events(players: &[Combattant]) -> Vec<Combattant> {
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
        let next_state = CreatureState {
            current_hp,
            temp_hp: None, // Temp HP lost on rest
            resources: resources.into(),
            position: player.final_state.position,
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
pub(crate) fn apply_short_rest_standalone(
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
            let next_state = CreatureState {
                current_hp: c_state.current_hp,
                temp_hp: None, // Temp HP lost on rest
                resources: c_state.resources.clone().into(),
                position: c_state.position,
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
pub(crate) fn update_player_states_for_next_encounter(
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
            let next_state = CreatureState {
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
                position: crate::model::Position::default(),
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
pub(crate) fn reconstruct_actions(
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
pub(crate) fn convert_to_legacy_simulation_result(
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
            let final_creature_state = CreatureState {
                current_hp: state.current_hp,
                temp_hp: Some(state.temp_hp),
                buffs: HashMap::new(), // TODO: Convert active effects to buffs if needed
                resources: state.resources.clone().into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: state.concentration.clone(),
                position: crate::model::Position::default(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
                cumulative_spent: state.cumulative_spent,
            };

            let mut combatant = state.base_combatant.clone();
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
            let final_creature_state = CreatureState {
                current_hp: state.current_hp,
                temp_hp: Some(state.temp_hp),
                buffs: HashMap::new(),
                resources: state.resources.clone().into(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: state.concentration.clone(),
                position: crate::model::Position::default(),
                actions_used_this_encounter: HashSet::new(),
                bonus_action_used: false,
                known_ac: HashMap::new(),
                arcane_ward_hp: None,
                cumulative_spent: state.cumulative_spent,
            };

            let mut combatant = state.base_combatant.clone();
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