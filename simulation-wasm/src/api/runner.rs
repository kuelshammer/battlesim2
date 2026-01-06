use crate::model::{Creature, SimulationResult, Combattant, CreatureState};
use crate::execution::ActionExecutionEngine;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Phase 1: Survey pass - runs all iterations with lightweight simulation (no event collection)
pub fn run_survey_pass(
    players: Vec<Creature>,
    timeline: Vec<crate::model::TimelineStep>,
    iterations: usize,
    base_seed: Option<u64>,
) -> Vec<crate::model::LightweightRun> {
    let mut all_runs = Vec::with_capacity(iterations);
    let scenario_hash = crate::cache::get_scenario_hash(&players, &timeline);

    for i in 0..iterations {
        let seed = base_seed.unwrap_or(i as u64).wrapping_add(i as u64);

        if let Some(cached_run) = crate::cache::get_cached_run(scenario_hash, seed) {
            all_runs.push(cached_run);
            continue;
        }

        let lightweight_run = run_single_lightweight_simulation(&players, &timeline, seed);
        crate::cache::insert_cached_run(scenario_hash, seed, lightweight_run.clone());
        all_runs.push(lightweight_run);
    }

    all_runs
}

/// Run a single event-driven simulation with full event collection
pub fn run_single_event_driven_simulation(
    players: &[Creature],
    timeline: &[crate::model::TimelineStep],
    _log_enabled: bool,
) -> (SimulationResult, Vec<crate::events::Event>) {
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
                let enemies = initialize_enemies(step_idx, &encounter.monsters);
                let mut all_combatants = players_with_state.clone();
                all_combatants.extend(enemies);

                let mut engine = ActionExecutionEngine::new(all_combatants.clone(), true);
                let encounter_result = engine.execute_encounter();

                all_events.extend(encounter_result.event_history.clone());

                let legacy_result = crate::simulation::convert_to_legacy_simulation_result(
                    &encounter_result,
                    step_idx,
                    encounter.target_role.clone(),
                );
                encounter_results.push(legacy_result);

                players_with_state =
                    crate::simulation::update_player_states_for_next_encounter(&players_with_state, &encounter_result, false);
            }
            crate::model::TimelineStep::ShortRest(_) => {
                players_with_state = crate::simulation::apply_short_rest_standalone(&players_with_state, &mut all_events);

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

    let mut result = SimulationResult {
        encounters: encounter_results,
        score: None,
        num_combat_encounters,
        seed,
    };

    let score = crate::aggregation::calculate_efficiency_score(&result, &all_events);
    result.score = Some(score);

    (result, all_events)
}

/// Lightweight simulation that tracks only scores and deaths, no event collection
pub fn run_single_lightweight_simulation(
    players: &[Creature],
    timeline: &[crate::model::TimelineStep],
    seed: u64,
) -> crate::model::LightweightRun {
    crate::rng::seed_rng(seed);

    let mut encounter_scores = Vec::new();
    let mut has_death = false;
    let mut first_death_encounter: Option<usize> = None;

    let mut players_with_state = initialize_players(players);

    for (step_idx, step) in timeline.iter().enumerate() {
        match step {
            crate::model::TimelineStep::Combat(encounter) => {
                let enemies = initialize_enemies(step_idx, &encounter.monsters);
                let mut all_combatants = players_with_state.clone();
                all_combatants.extend(enemies);

                let mut engine = ActionExecutionEngine::new(all_combatants.clone(), false);
                let encounter_result = engine.execute_encounter();

                let score = crate::safe_aggregation::calculate_lightweight_score(
                    &encounter_result.final_combatant_states,
                );
                encounter_scores.push(score);

                if !has_death {
                    for combatant in &encounter_result.final_combatant_states {
                        if combatant.current_hp == 0 && combatant.base_combatant.team == 0 {
                            has_death = true;
                            first_death_encounter = Some(encounter_scores.len() - 1);
                            break;
                        }
                    }
                }

                players_with_state = crate::simulation::update_player_states_for_next_encounter(
                    &players_with_state,
                    &encounter_result,
                    false,
                );
            }
            crate::model::TimelineStep::ShortRest(_) => {
                players_with_state = crate::simulation::apply_short_rest_standalone_no_events(&players_with_state);
            }
        }
    }

    let total_survivors = players_with_state
        .iter()
        .filter(|p| p.final_state.current_hp > 0)
        .count();

    let final_score = encounter_scores.last().copied().unwrap_or(0.0);

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

pub(crate) fn initialize_players(players: &[Creature]) -> Vec<Combattant> {
    let mut players_with_state = Vec::new();
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
            let combattant = create_player_combatant(group_idx, i, player);
            players_with_state.push(combattant);
        }
    }
    players_with_state
}

pub(crate) fn initialize_enemies(step_idx: usize, monsters: &[Creature]) -> Vec<Combattant> {
    let mut enemies = Vec::new();
    for (group_idx, monster) in monsters.iter().enumerate() {
        for i in 0..monster.count as i32 {
            let enemy_combattant = create_enemy_combatant(step_idx, group_idx, i, monster);
            enemies.push(enemy_combattant);
        }
    }
    enemies
}

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

fn create_creature_state(creature: &Creature) -> CreatureState {
    let mut state = CreatureState {
        current_hp: creature.hp,
        temp_hp: None,
        buffs: HashMap::new(),
        resources: {
            let mut r = crate::model::SerializableResourceLedger::from(creature.initialize_ledger());
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
        arcane_ward_hp: creature.max_arcane_ward_hp,
        cumulative_spent: 0.0,
    };

    for (index, buff) in creature.initial_buffs.iter().enumerate() {
        let buff_id = buff.display_name.clone()
            .unwrap_or_else(|| format!("initial-buff-{}", index));
        state.buffs.insert(buff_id, buff.clone());
    }

    state
}
