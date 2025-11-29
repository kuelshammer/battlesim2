use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use rand::Rng;
use crate::model::*;
use crate::enums::*;
use crate::dice;

pub fn run_monte_carlo(players: &[Creature], encounters: &[Encounter], iterations: usize) -> Vec<SimulationResult> {
    let mut results: Vec<(f64, SimulationResult)> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let result = run_single_simulation(players, encounters);
        let score = calculate_score(&result);
        results.push((score, result));
    }

    // Sort by score
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Return all results sorted by score
    results.into_iter().map(|(_, r)| r).collect()
}


struct AggregationData {
    total_hp: f64,
    action_counts: HashMap<String, usize>,
    buff_counts: HashMap<String, usize>,
    buff_definitions: HashMap<String, Buff>, // ID -> Buff
    concentration_counts: HashMap<String, usize>, // ID -> Count
}

impl AggregationData {
    fn new() -> Self {
        Self {
            total_hp: 0.0,
            action_counts: HashMap::new(),
            buff_counts: HashMap::new(),
            buff_definitions: HashMap::new(),
            concentration_counts: HashMap::new(),
        }
    }
}

pub fn aggregate_results(results: &[SimulationResult]) -> Vec<Round> {
    if results.is_empty() { return Vec::new(); }
    
    // Assuming all results have the same number of encounters (usually 1 for now)
    // We aggregate the first encounter's rounds.
    
    let max_rounds = results.iter().map(|r| r.first().map(|e| e.rounds.len()).unwrap_or(0)).max().unwrap_or(0);
    let mut aggregated_rounds: Vec<Round> = Vec::with_capacity(max_rounds);
    
    let template_encounter = results.first().and_then(|r| r.first());
    if template_encounter.is_none() { return Vec::new(); }
    let template_encounter = template_encounter.unwrap();
    
    // Get template IDs for mapping
    let mut template_ids_t1: Vec<String> = Vec::new();
    let mut template_ids_t2: Vec<String> = Vec::new();
    if let Some(first_round) = template_encounter.rounds.first() {
        for c in &first_round.team1 { template_ids_t1.push(c.id.clone()); }
        for c in &first_round.team2 { template_ids_t2.push(c.id.clone()); }
    }
    
    for round_idx in 0..max_rounds {
        let mut team1_map: HashMap<String, AggregationData> = HashMap::new();
        let mut team2_map: HashMap<String, AggregationData> = HashMap::new();
        let mut count = 0;
        
        for res in results {
            if let Some(encounter) = res.first() {
                // Build UUID map for this run
                let mut uuid_map: HashMap<String, String> = HashMap::new();
                if let Some(first_round) = encounter.rounds.first() {
                    for (i, c) in first_round.team1.iter().enumerate() {
                        if i < template_ids_t1.len() {
                            uuid_map.insert(c.id.clone(), template_ids_t1[i].clone());
                        }
                    }
                    for (i, c) in first_round.team2.iter().enumerate() {
                        if i < template_ids_t2.len() {
                            uuid_map.insert(c.id.clone(), template_ids_t2[i].clone());
                        }
                    }
                }

                // Determine which round to use (actual or last padding)
                let round_opt = if round_idx < encounter.rounds.len() {
                    encounter.rounds.get(round_idx)
                } else {
                    encounter.rounds.last()
                };

                if let Some(round) = round_opt {
                    count += 1;
                    
                    for c in &round.team1 {
                        let mapped_id = uuid_map.get(&c.id).unwrap_or(&c.creature.id).clone();
                        // Fallback to creature ID if not in map, but ideally we use the template UUID map
                        // Actually team1_map keys should be the TEMPLATE IDs (from template_ids_t1)
                        // But here `c.creature.id` might be same as `c.id`? No.
                        // `c.creature.id` is the definition ID. `c.id` is the instance ID.
                        // The aggregation map is keyed by `c.creature.id` in the previous code, which is wrong if multiple same creatures exist.
                        // It should be keyed by the Template Instance ID.
                        // In previous code: `team1_map.entry(c.creature.id.clone())` -> This merges all Goblins into one bucket!
                        // Wait, `c.creature.id` comes from `uuid::Uuid::new_v4()` in `create_combattant`? No.
                        // `create_combattant` takes `Creature` which has an ID.
                        // In `run_single_simulation`: `p.name = name; create_combattant(p)`.
                        // `create_combattant` generates a NEW `id` for the `Combattant`.
                        // `Combattant.creature.id` preserves the original definition ID.
                        
                        // The `uuid_map` maps `current_run_combattant_id` -> `template_run_combattant_id`.
                        // So we should use `mapped_id` as key for `team1_map`.
                        // Previous code used `c.creature.id`. This is BUGGY for multiple monsters of same type!
                        // It aggregates all "Goblin" stats together if `creature.id` is shared.
                        // But `c.creature` is cloned from input `players`. If players input has unique IDs for each line, it's fine.
                        // But if `count: 5`, we generate 5 combattants. They share `creature.id`?
                        // In `run_single_simulation`: `players.iter().flat_map(...)`.
                        // `p` is a clone of `player`. `player` comes from `players`.
                        // The `id` in `Creature` struct is usually the template ID (e.g. from library).
                        // So yes, `c.creature.id` is likely shared.
                        // WE MUST USE THE MAPPED INSTANCE ID.
                        
                        // Let's fix the key to use `mapped_id` (which maps to template instance ID).
                        
                        // Find the template ID corresponding to this creature
                        // We rely on the order. `uuid_map` is built by index.
                        // So `mapped_id` is correct.
                        
                        let entry = team1_map.entry(mapped_id.clone()).or_insert_with(AggregationData::new);
                        entry.total_hp += c.final_state.current_hp;
                        
                        // Remap targets in actions
                        let mut actions = c.actions.clone();
                        for action in &mut actions {
                            let mut new_targets = HashMap::new();
                            for (target_id, count) in &action.targets {
                                if let Some(m_id) = uuid_map.get(target_id) {
                                    new_targets.insert(m_id.clone(), *count);
                                } else {
                                    new_targets.insert(target_id.clone(), *count);
                                }
                            }
                            action.targets = new_targets;
                        }
                        
                        let action_key = serde_json::to_string(&actions).unwrap_or_default();
                        *entry.action_counts.entry(action_key).or_insert(0) += 1;

                        // Aggregate Buffs
                        for (buff_id, buff) in &c.final_state.buffs {
                            *entry.buff_counts.entry(buff_id.clone()).or_insert(0) += 1;
                            entry.buff_definitions.entry(buff_id.clone()).or_insert_with(|| {
                                let mut b = buff.clone();
                                if let Some(sid) = &b.source {
                                    if let Some(mapped_sid) = uuid_map.get(sid) {
                                        b.source = Some(mapped_sid.clone());
                                    }
                                }
                                b
                            });
                        }

                        // Aggregate Concentration
                        if let Some(conc_id) = &c.final_state.concentrating_on {
                            *entry.concentration_counts.entry(conc_id.clone()).or_insert(0) += 1;
                        }
                    }
                    
                    // Team 2
                    for c in &round.team2 {
                        let mapped_id = uuid_map.get(&c.id).unwrap_or(&c.creature.id).clone();
                        let entry = team2_map.entry(mapped_id.clone()).or_insert_with(AggregationData::new);
                        entry.total_hp += c.final_state.current_hp;
                        
                        let mut actions = c.actions.clone();
                        for action in &mut actions {
                            let mut new_targets = HashMap::new();
                            for (target_id, count) in &action.targets {
                                if let Some(m_id) = uuid_map.get(target_id) {
                                    new_targets.insert(m_id.clone(), *count);
                                } else {
                                    new_targets.insert(target_id.clone(), *count);
                                }
                            }
                            action.targets = new_targets;
                        }

                        let action_key = serde_json::to_string(&actions).unwrap_or_default();
                        *entry.action_counts.entry(action_key).or_insert(0) += 1;

                        for (buff_id, buff) in &c.final_state.buffs {
                            *entry.buff_counts.entry(buff_id.clone()).or_insert(0) += 1;
                            entry.buff_definitions.entry(buff_id.clone()).or_insert_with(|| {
                                let mut b = buff.clone();
                                if let Some(sid) = &b.source {
                                    if let Some(mapped_sid) = uuid_map.get(sid) {
                                        b.source = Some(mapped_sid.clone());
                                    }
                                }
                                b
                            });
                        }

                        if let Some(conc_id) = &c.final_state.concentrating_on {
                            *entry.concentration_counts.entry(conc_id.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        
        if count == 0 { continue; }
        let threshold = count / 2;
        
        // Reconstruct Team 1
        let mut t1 = Vec::new();
        if let Some(template_round) = template_encounter.rounds.get(round_idx) {
             for c_template in &template_round.team1 {
                 // We use `c_template.id` as the key because we mapped everything to it.
                 if let Some(data) = team1_map.get(&c_template.id) {
                     let avg_hp = data.total_hp / count as f64;
                     let best_action_json = data.action_counts.iter().max_by_key(|entry| entry.1).map(|(k, _)| k).unwrap();
                     let actions: Vec<CombattantAction> = serde_json::from_str(best_action_json).unwrap_or_default();
                     
                     let mut c = c_template.clone();
                     c.final_state.current_hp = avg_hp;
                     c.actions = actions;
                     
                     // Reconstruct Buffs
                     c.final_state.buffs.clear();
                     for (buff_id, buff_count) in &data.buff_counts {
                         if *buff_count > threshold {
                             if let Some(buff_def) = data.buff_definitions.get(buff_id) {
                                 c.final_state.buffs.insert(buff_id.clone(), buff_def.clone());
                             }
                         }
                     }

                     // Reconstruct Concentration
                     c.final_state.concentrating_on = None;
                     // Find the concentration ID with max count
                     if let Some((conc_id, conc_count)) = data.concentration_counts.iter().max_by_key(|e| e.1) {
                         if *conc_count > threshold {
                             c.final_state.concentrating_on = Some(conc_id.clone());
                         }
                     }
                     
                     // Fix initial_state: It should be the final_state of the previous round.
                     if round_idx > 0 {
                         if let Some(prev_round) = aggregated_rounds.get(round_idx - 1) {
                             if let Some(prev_c) = prev_round.team1.iter().find(|pc| pc.creature.id == c.creature.id) {
                                 c.initial_state = prev_c.final_state.clone();
                             }
                         }
                     }
                     
                     t1.push(c);
                 }
             }
        }
        
        // Reconstruct Team 2
        let mut t2 = Vec::new();
        if let Some(template_round) = template_encounter.rounds.get(round_idx) {
             for c_template in &template_round.team2 {
                 if let Some(data) = team2_map.get(&c_template.id) {
                     let avg_hp = data.total_hp / count as f64;
                     let best_action_json = data.action_counts.iter().max_by_key(|entry| entry.1).map(|(k, _)| k).unwrap();
                     let actions: Vec<CombattantAction> = serde_json::from_str(best_action_json).unwrap_or_default();
                     
                     let mut c = c_template.clone();
                     c.final_state.current_hp = avg_hp;
                     c.actions = actions;

                     // Reconstruct Buffs
                     c.final_state.buffs.clear();
                     for (buff_id, buff_count) in &data.buff_counts {
                         if *buff_count > threshold {
                             if let Some(buff_def) = data.buff_definitions.get(buff_id) {
                                 c.final_state.buffs.insert(buff_id.clone(), buff_def.clone());
                             }
                         }
                     }

                     // Reconstruct Concentration
                     c.final_state.concentrating_on = None;
                     if let Some((conc_id, conc_count)) = data.concentration_counts.iter().max_by_key(|e| e.1) {
                         if *conc_count > threshold {
                             c.final_state.concentrating_on = Some(conc_id.clone());
                         }
                     }
                     
                     if round_idx > 0 {
                         if let Some(prev_round) = aggregated_rounds.get(round_idx - 1) {
                             if let Some(prev_c) = prev_round.team2.iter().find(|pc| pc.creature.id == c.creature.id) {
                                 c.initial_state = prev_c.final_state.clone();
                             }
                         }
                     }
                     
                     t2.push(c);
                 }
             }
        }
        
        // Consistency Cleanup: Enforce "Dead = No Concentration" on the aggregated result
        // This handles edge cases where statistical aggregation might result in inconsistent states (e.g. HP < 0.5 but Concentration > 50% due to some anomaly, or just to align visual state).
        let mut dead_source_ids = HashSet::new();
        
        // 1. Identify effectively dead combatants and clear their concentration
        for c in t1.iter_mut().chain(t2.iter_mut()) {
            if c.final_state.current_hp < 0.5 {
                // Consider effectively dead for visualization
                if c.final_state.concentrating_on.is_some() {
                    c.final_state.concentrating_on = None;
                }
                dead_source_ids.insert(c.id.clone());
            }
        }
        
        // 2. Remove buffs sourced by these dead combatants
        if !dead_source_ids.is_empty() {
            for c in t1.iter_mut().chain(t2.iter_mut()) {
                c.final_state.buffs.retain(|_, buff| {
                    if let Some(source) = &buff.source {
                        !dead_source_ids.contains(source)
                    } else {
                        true
                    }
                });
            }
        }

        aggregated_rounds.push(Round { team1: t1, team2: t2 });
    }
    
    aggregated_rounds
}

fn calculate_score(result: &SimulationResult) -> f64 {
    if result.is_empty() { return 0.0; }
    
    let last_encounter = result.last().unwrap();
    let last_round = last_encounter.rounds.last();
    
    if let Some(round) = last_round {
        let player_hp: f64 = round.team1.iter().map(|c| c.final_state.current_hp).sum();
        let monster_hp: f64 = round.team2.iter().map(|c| c.final_state.current_hp).sum();
        
        return 3.0 * player_hp - monster_hp;
    }
    
    0.0
}

fn run_single_simulation(players: &[Creature], encounters: &[Encounter]) -> SimulationResult {
    let mut results = Vec::new();
    
    // Initialize players with state
    let mut players_with_state: Vec<Combattant> = players.iter().flat_map(|player| {
        (0..player.count as i32).map(|i| {
            let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
            let mut p = player.clone();
            p.name = name;
            
            create_combattant(p)
        }).collect::<Vec<_>>()
    }).collect();

    for (index, encounter) in encounters.iter().enumerate() {
        let encounter_result = run_encounter(&players_with_state, encounter);
        results.push(encounter_result.clone());
        
        // Prepare for next encounter
        if let Some(last_round) = encounter_result.rounds.last() {
            let next_encounter = encounters.get(index + 1);
            let is_short_rest = next_encounter.map(|e| e.short_rest.unwrap_or(false)).unwrap_or(false);
            
            players_with_state = last_round.team1.iter().map(|c| {
                let mut state = c.final_state.clone();
                if is_short_rest {
                    state.current_hp = c.creature.hp; 
                    state.remaining_uses = get_remaining_uses(&c.creature, "short rest", Some(&state.remaining_uses));
                } else {
                    state.remaining_uses = get_remaining_uses(&c.creature, "none", Some(&state.remaining_uses));
                }
                
                state.buffs.clear();
                state.upcoming_buffs.clear();
                state.used_actions.clear();
                
                Combattant {
                    id: c.id.clone(),
                    initiative: roll_initiative(&c.creature),
                    creature: c.creature.clone(),
                    initial_state: state.clone(),
                    final_state: state,
                    actions: Vec::new(),
                }
            }).collect();
        }
    }

    results
}

fn create_combattant(creature: Creature) -> Combattant {
    let state = CreatureState {
        current_hp: creature.hp,
        temp_hp: None,
        buffs: HashMap::new(),
        remaining_uses: get_remaining_uses(&creature, "long rest", None),
        upcoming_buffs: HashMap::new(),
        used_actions: HashSet::new(),
        concentrating_on: None,
    };
    Combattant {
        id: Uuid::new_v4().to_string(),
        initiative: roll_initiative(&creature),
        creature: creature.clone(),
        initial_state: state.clone(),
        final_state: state,
        actions: Vec::new(),
    }
}

fn roll_initiative(c: &Creature) -> f64 {
    let roll = if c.initiative_advantage {
        let r1 = rand::thread_rng().gen_range(1..=20);
        let r2 = rand::thread_rng().gen_range(1..=20);
        r1.max(r2)
    } else {
        rand::thread_rng().gen_range(1..=20)
    } as f64;
    
    // eprintln!("Rolling init for {}: Roll {} + Bonus {} = {}", c.name, roll, c.initiative_bonus, roll + c.initiative_bonus);
    roll + c.initiative_bonus
}

fn get_remaining_uses(creature: &Creature, rest: &str, old_value: Option<&HashMap<String, f64>>) -> HashMap<String, f64> {
    let mut result = HashMap::new();
    
    for action in &creature.actions {
        let val = match &action.base().freq {
            Frequency::Static(s) if s == "at will" => continue,
            Frequency::Static(s) if s == "1/fight" => {
                if rest == "long rest" || rest == "short rest" { 1.0 } else { *old_value.and_then(|m| m.get(&action.base().id)).unwrap_or(&0.0) }
            },
            Frequency::Static(s) if s == "1/day" => {
                if rest == "long rest" { 1.0 } else { *old_value.and_then(|m| m.get(&action.base().id)).unwrap_or(&0.0) }
            },
            Frequency::Recharge { .. } => 1.0,
            Frequency::Limited { reset, uses } => {
                if reset == "lr" {
                    if rest == "long rest" { *uses as f64 } else { *old_value.and_then(|m| m.get(&action.base().id)).unwrap_or(&0.0) }
                } else { // sr
                    if rest == "long rest" || rest == "short rest" { *uses as f64 } else { *old_value.and_then(|m| m.get(&action.base().id)).unwrap_or(&0.0) }
                }
            },
            _ => 0.0,
        };
        result.insert(action.base().id.clone(), val);
    }
    
    result
}

fn run_encounter(players: &[Combattant], encounter: &Encounter) -> EncounterResult {
    let mut team2: Vec<Combattant> = encounter.monsters.iter().flat_map(|monster| {
        (0..monster.count as i32).map(|i| {
            let name = if monster.count > 1.0 { format!("{} {}", monster.name, i + 1) } else { monster.name.clone() };
            let mut m = monster.clone();
            m.name = name;
            create_combattant(m)
        }).collect::<Vec<_>>()
    }).collect();
    
    let mut team1 = players.to_vec();
    
    let mut rounds = Vec::new();
    let mut stats = HashMap::new();
    
    let max_rounds = 100;
    for _ in 0..max_rounds {
        if !team1.iter().any(|c| c.final_state.current_hp > 0.0) || !team2.iter().any(|c| c.final_state.current_hp > 0.0) {
            break;
        }
        
        let round = run_round(&team1, &team2, &mut stats);
        rounds.push(round.clone());
        
        team1 = round.team1;
        team2 = round.team2;
    }
    
    EncounterResult {
        stats,
        rounds,
    }
}

fn run_round(team1: &[Combattant], team2: &[Combattant], stats: &mut HashMap<String, EncounterStats>) -> Round {
    #[cfg(debug_assertions)]
    eprintln!("\n--- Round START ---");
    // 1. Create mutable copies of teams
    let mut t1 = team1.to_vec();
    let mut t2 = team2.to_vec();
    
    // 2. Create turn order
    #[derive(Clone, Copy, Debug)]
    enum TeamId { Team1, Team2 }
    
    let mut turn_order: Vec<(TeamId, usize, f64)> = Vec::new();
    for (i, c) in t1.iter().enumerate() { turn_order.push((TeamId::Team1, i, c.initiative)); }
    for (i, c) in t2.iter().enumerate() { turn_order.push((TeamId::Team2, i, c.initiative)); }
    
    // Sort by initiative descending
    turn_order.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    
    #[cfg(debug_assertions)]
    eprintln!("  Turn Order: {:?}", turn_order.iter().map(|(id,_,init)| format!("{:?} {:.1}", id, init)).collect::<Vec<_>>());

    
    // 3. Iterate through turns
    for (team_id, idx, initiative_value) in turn_order {
        let combatant_name = match team_id {
            TeamId::Team1 => t1[idx].creature.name.clone(),
            TeamId::Team2 => t2[idx].creature.name.clone(),
        };
        #[cfg(debug_assertions)]
        eprintln!("  Turn for: {} (Init: {:.1})", combatant_name, initiative_value);

        // Check if creature is still alive (might have died in previous turn)
        let is_alive = match team_id {
            TeamId::Team1 => t1[idx].final_state.current_hp > 0.0,
            TeamId::Team2 => t2[idx].final_state.current_hp > 0.0,
        };
        if !is_alive {
            #[cfg(debug_assertions)]
            eprintln!("    {} is dead, skipping turn.", combatant_name);
            continue;
        }
        
        // Start of turn updates (iterate_combattant logic)
        match team_id {
            TeamId::Team1 => t1[idx] = iterate_combattant(&t1[idx]),
            TeamId::Team2 => t2[idx] = iterate_combattant(&t2[idx]),
        }
        
        // Execute turn
        match team_id {
            TeamId::Team1 => execute_turn(idx, &mut t1, &mut t2, stats),
            TeamId::Team2 => execute_turn(idx, &mut t2, &mut t1, stats),
        }
        #[cfg(debug_assertions)]
        eprintln!("  {} turn END. Current State: P1 HP: {:.1}, P2 HP: {:.1}", combatant_name, t1[0].final_state.current_hp, t2[0].final_state.current_hp);
    }
    
    // Remove buffs from dead sources (at end of round)
    let mut dead_ids = HashSet::new();
    for c in t1.iter().chain(t2.iter()) {
        if c.final_state.current_hp <= 0.0 {
            dead_ids.insert(c.id.clone());
        }
    }
    
    remove_dead_buffs(&mut t1, &dead_ids);
    remove_dead_buffs(&mut t2, &dead_ids);

    Round {
        team1: t1,
        team2: t2,
    }
}

fn remove_dead_buffs(targets: &mut [Combattant], dead_source_ids: &HashSet<String>) {
    if dead_source_ids.is_empty() { return; }

    for target in targets.iter_mut() {
        target.final_state.buffs.retain(|_, buff| {
            if let Some(source) = &buff.source {
                !dead_source_ids.contains(source)
            } else {
                true
            }
        });
    }
}

fn iterate_combattant(c: &Combattant) -> Combattant {
    let mut new_initial_state = c.final_state.clone();
    new_initial_state.buffs.clear();
    new_initial_state.upcoming_buffs.clear();
    
    for (name, buff) in &c.final_state.buffs {
        match buff.duration {
            BuffDuration::EntireEncounter => {
                new_initial_state.buffs.insert(name.clone(), buff.clone());
            },
            BuffDuration::RepeatTheSaveEachRound => {
                let dc = dice::evaluate(buff.dc.as_ref().unwrap_or(&DiceFormula::Value(10.0)), 1);
                let save_bonus = c.creature.save_bonus; 
                let roll = rand::thread_rng().gen_range(1..=20) as f64;
                if roll + save_bonus < dc {
                     new_initial_state.buffs.insert(name.clone(), buff.clone());
                }
            },
            _ => {}
        }
    }
    
    for (name, buff) in &c.final_state.upcoming_buffs {
        new_initial_state.buffs.insert(name.clone(), buff.clone());
    }
    
    for action in &c.creature.actions {
        if let Frequency::Recharge { cooldown_rounds, .. } = &action.base().freq {
            let increment = 1.0 / *cooldown_rounds as f64;
            let current = *new_initial_state.remaining_uses.get(&action.base().id).unwrap_or(&0.0);
            new_initial_state.remaining_uses.insert(action.base().id.clone(), (current + increment).min(1.0));
        }
    }
    
    Combattant {
        id: c.id.clone(),
        initiative: c.initiative,
        creature: c.creature.clone(),
        initial_state: new_initial_state.clone(),
        final_state: new_initial_state,
        actions: Vec::new(),
    }
}

fn generate_actions_for_creature(c: &mut Combattant, allies: &[Combattant], enemies: &[Combattant]) {
    #[cfg(debug_assertions)]
    eprintln!("    Generate actions for {}. Current HP: {:.1}", c.creature.name, c.initial_state.current_hp);
    if c.initial_state.current_hp <= 0.0 { return; }
    
    let actions = get_actions(c, allies, enemies);
    #[cfg(debug_assertions)]
    eprintln!("      Generated {} actions for {}: {:?}", actions.len(), c.creature.name, actions.iter().map(|a| a.base().name.clone()).collect::<Vec<_>>());
    
    for action in actions {
        if let Frequency::Static(s) = &action.base().freq {
            if s != "at will" {
                 let uses = *c.initial_state.remaining_uses.get(&action.base().id).unwrap_or(&0.0);
                 c.final_state.remaining_uses.insert(action.base().id.clone(), (uses - 1.0).max(0.0));
                 c.final_state.used_actions.insert(action.base().id.clone());
            }
        }
        
        c.actions.push(CombattantAction {
            action,
            targets: HashMap::new(),
        });
    }
}

fn get_actions(c: &Combattant, allies: &[Combattant], enemies: &[Combattant]) -> Vec<Action> {
    #[cfg(debug_assertions)]
    eprintln!("      Getting actions for {}. Creature actions: {}", c.creature.name, c.creature.actions.len());
    let mut result = Vec::new();
    let mut used_slots = HashSet::new();
    
    for action in &c.creature.actions {
        #[cfg(debug_assertions)]
        eprintln!("        Considering action: {} (Slot: {}, Freq: {:?})", action.base().name, action.base().action_slot, action.base().freq);
        if used_slots.contains(&action.base().action_slot) {
            #[cfg(debug_assertions)]
            eprintln!("          Slot {} already used.", action.base().action_slot);
            continue;
        }
        if !is_usable(c, action) {
            #[cfg(debug_assertions)]
            eprintln!("          Action {} not usable.", action.base().name);
            continue;
        }
        
        // Match condition
        // Simplified: Always true for now or implement match_condition
        #[cfg(debug_assertions)]
        eprintln!("          Action {} usable. Adding to result.", action.base().name);
        result.push(action.clone());
        used_slots.insert(action.base().action_slot);
    }
    
    result
}

fn is_usable(c: &Combattant, action: &Action) -> bool {
    #[cfg(debug_assertions)]
    eprintln!("        Checking usability for {}: {}. Remaining uses: {:?}", c.creature.name, action.base().name, c.final_state.remaining_uses.get(&action.base().id));
    match &action.base().freq {
        Frequency::Static(s) if s == "at will" => true,
        _ => {
            let uses = *c.final_state.remaining_uses.get(&action.base().id).unwrap_or(&0.0);
            uses >= 1.0
        }
    }
}

// Helper to determine if a combatant has a specific condition
fn has_condition(c: &Combattant, condition: CreatureCondition) -> bool {
    c.final_state.buffs.iter()
        .any(|(_, buff)| buff.condition == Some(condition))
}

// Helper to get effective attack roll considering advantage/disadvantage
fn get_attack_roll_result(attacker: &Combattant) -> (f64, bool, bool) {
    let mut rng = rand::thread_rng();
    let roll1 = rng.gen_range(1..=20) as f64;
    let roll2 = rng.gen_range(1..=20) as f64;

    let has_advantage = has_condition(attacker, CreatureCondition::AttacksWithAdvantage) || has_condition(attacker, CreatureCondition::AttacksAndIsAttackedWithAdvantage);
    let has_disadvantage = has_condition(attacker, CreatureCondition::AttacksWithDisadvantage) || has_condition(attacker, CreatureCondition::AttacksAndSavesWithDisadvantage); // Assuming this also applies to attacks.

    let final_roll: f64;
    let is_crit_hit: bool;
    let is_crit_miss: bool;

    if has_advantage && !has_disadvantage { // Pure Advantage
        final_roll = roll1.max(roll2);
        is_crit_hit = roll1 == 20.0 || roll2 == 20.0;
        is_crit_miss = roll1 == 1.0 && roll2 == 1.0;
    } else if has_disadvantage && !has_advantage { // Pure Disadvantage
        final_roll = roll1.min(roll2);
        is_crit_hit = roll1 == 20.0 && roll2 == 20.0;
        is_crit_miss = roll1 == 1.0 || roll2 == 1.0;
    } else { // Normal roll, or advantage/disadvantage cancel out
        final_roll = roll1;
        is_crit_hit = roll1 == 20.0;
        is_crit_miss = roll1 == 1.0;
    }

    (final_roll, is_crit_hit, is_crit_miss)
}

fn execute_turn(attacker_idx: usize, allies: &mut [Combattant], enemies: &mut [Combattant], stats: &mut HashMap<String, EncounterStats>) {
    let attacker = allies[attacker_idx].clone(); // Clone attacker for logging convenience
    #[cfg(debug_assertions)]
    eprintln!("    Executing turn for {}. Current HP: {:.1}", attacker.creature.name, attacker.final_state.current_hp);

    // 1. Generate actions
    // We need a read-only view of allies for decision making
    let allies_clone = allies.to_vec();
    generate_actions_for_creature(&mut allies[attacker_idx], &allies_clone, enemies);

    // 2. Execute actions
    // Collect actions to avoid borrow checker
    let mut pending_actions = Vec::new();
    for (action_idx, action_entry) in allies[attacker_idx].actions.iter().enumerate() {
        pending_actions.push((action_idx, action_entry.action.clone()));
    }
    #[cfg(debug_assertions)]
    eprintln!("      {} has {} pending actions: {:?}", attacker.creature.name, pending_actions.len(), pending_actions.iter().map(|(_,a)| a.base().name.clone()).collect::<Vec<_>>());
    
    for (action_idx, action) in pending_actions {
        let targets = get_targets(&attacker, &action, allies, enemies);
        #[cfg(debug_assertions)]
        eprintln!("        Action {}: {} - Found {} targets.", action.base().name, action.base().id, targets.len());
        
        // Update targets
        for (is_enemy, target_idx) in &targets {
            let target_id = if *is_enemy { enemies[*target_idx].id.clone() } else { allies[*target_idx].id.clone() };
            let entry = allies[attacker_idx].actions[action_idx].targets.entry(target_id).or_insert(0);
            *entry += 1;
        }
        
        // Execute
        for (is_enemy, target_idx) in targets {
            let target_name = if is_enemy { enemies[target_idx].creature.name.clone() } else { allies[target_idx].creature.name.clone() };
            let current_target_hp = if is_enemy { enemies[target_idx].final_state.current_hp } else { allies[target_idx].final_state.current_hp };
            #[cfg(debug_assertions)]
            eprintln!("          Executing action {} by {} on {}. Target HP: {:.1}", action.base().name, attacker.creature.name, target_name, current_target_hp);

            match &action {
                Action::Atk(a) => {
                    let target_ac = if is_enemy { enemies[target_idx].creature.ac } else { allies[target_idx].creature.ac };
                    let (d20_roll, is_crit_hit_roll, is_crit_miss_roll) = get_attack_roll_result(&attacker);
                    let to_hit = dice::evaluate(&a.to_hit, 1);

                    #[cfg(debug_assertions)]
                    eprintln!("            Attack: d20_roll: {:.1}, to_hit: {:.1}, target_ac: {:.1}. Crit: {}, Miss: {}", d20_roll, to_hit, target_ac, is_crit_hit_roll, is_crit_miss_roll);

                    // Natural 1 is always a miss (even if modifiers would hit)
                    if is_crit_miss_roll {
                        #[cfg(debug_assertions)]
                        eprintln!("              CRIT MISS! {} misses {}.", attacker.creature.name, target_name);
                        continue;
                    }

                    let dice_multiplier = if is_crit_hit_roll { 2 } else { 1 };
                    
                    // Natural 20 is always a hit (even if modifiers would miss)
                    if d20_roll + to_hit >= target_ac || is_crit_hit_roll {
                        let dmg = dice::evaluate(&a.dpr, dice_multiplier);
                        #[cfg(debug_assertions)]
                        eprintln!("              HIT! {} hits {} for {:.1} damage (Multiplier: {}).", attacker.creature.name, target_name, dmg, dice_multiplier);
                            if is_enemy {
                                enemies[target_idx].final_state.current_hp = (enemies[target_idx].final_state.current_hp - dmg).max(0.0);
                                #[cfg(debug_assertions)]
                                eprintln!("                {} new HP: {:.1}", target_name, enemies[target_idx].final_state.current_hp);
                                update_stats(stats, &attacker.id, &enemies[target_idx].id, dmg, 0.0);

                                // Concentration Check
                                if let Some(buff_id) = enemies[target_idx].final_state.concentrating_on.clone() {
                                    if enemies[target_idx].final_state.current_hp <= 0.0 {
                                        break_concentration(&enemies[target_idx].id.clone(), &buff_id, allies, enemies);
                                    } else {
                                        let con_save_bonus = enemies[target_idx].creature.con_save_bonus.unwrap_or(enemies[target_idx].creature.save_bonus);
                                        let dc = (dmg / 2.0).max(10.0);
                                        let roll = rand::thread_rng().gen_range(1..=20) as f64;
                                        if roll + con_save_bonus < dc {
                                        let caster_id = enemies[target_idx].id.clone();
                                        break_concentration(&caster_id, &buff_id, allies, enemies);
                                    }    }
                                }
                            } else {
                                allies[target_idx].final_state.current_hp = (allies[target_idx].final_state.current_hp - dmg).max(0.0);
                                #[cfg(debug_assertions)]
                                eprintln!("                {} new HP: {:.1}", target_name, allies[target_idx].final_state.current_hp);
                                update_stats(stats, &attacker.id, &allies[target_idx].id, dmg, 0.0);

                                // Concentration Check
                                if let Some(buff_id) = allies[target_idx].final_state.concentrating_on.clone() {
                                    if allies[target_idx].final_state.current_hp <= 0.0 {
                                        break_concentration(&allies[target_idx].id.clone(), &buff_id, allies, enemies);
                                    } else {
                                        let con_save_bonus = allies[target_idx].creature.con_save_bonus.unwrap_or(allies[target_idx].creature.save_bonus);
                                        let dc = (dmg / 2.0).max(10.0);
                                        let roll = rand::thread_rng().gen_range(1..=20) as f64;
                                        if roll + con_save_bonus < dc {
                                        let caster_id = allies[target_idx].id.clone();
                                        break_concentration(&caster_id, &buff_id, allies, enemies);
                                    }    }
                                }
                            }
                    } else {
                        #[cfg(debug_assertions)]
                        eprintln!("              MISS! {} misses {}.", attacker.creature.name, target_name);
                    }
                },
                Action::Heal(a) => {
                    let amount = dice::evaluate(&a.amount, 1);
                    if is_enemy {
                         let max_hp = enemies[target_idx].creature.hp;
                         enemies[target_idx].final_state.current_hp = (enemies[target_idx].final_state.current_hp + amount).min(max_hp);
                    } else {
                         let max_hp = allies[target_idx].creature.hp;
                         allies[target_idx].final_state.current_hp = (allies[target_idx].final_state.current_hp + amount).min(max_hp);
                         update_stats(stats, &attacker.id, &allies[target_idx].id, 0.0, amount);
                    }
                },
                Action::Buff(a) => {
                    if a.buff.concentration {
                        let current_concentration = allies[attacker_idx].final_state.concentrating_on.clone();
                        if let Some(old_buff) = current_concentration {
                             let caster_id = allies[attacker_idx].id.clone();
                             break_concentration(&caster_id, &old_buff, allies, enemies);
                        }
                        allies[attacker_idx].final_state.concentrating_on = Some(a.base().id.clone());
                    }
                    let mut buff = a.buff.clone();
                    buff.source = Some(attacker.id.clone());
                    if is_enemy {
                        enemies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &enemies[target_idx].id, true);
                    } else {
                        allies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &allies[target_idx].id, true);
                    }
                },
                Action::Debuff(a) => {
                    if a.buff.concentration {
                        let current_concentration = allies[attacker_idx].final_state.concentrating_on.clone();
                        if let Some(old_buff) = current_concentration {
                             let caster_id = allies[attacker_idx].id.clone();
                             break_concentration(&caster_id, &old_buff, allies, enemies);
                        }
                        allies[attacker_idx].final_state.concentrating_on = Some(a.base().id.clone());
                    }
                    let mut buff = a.buff.clone();
                    buff.source = Some(attacker.id.clone());
                    
                    let dc_val = a.save_dc; // Extract f64 value
                    let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1); // Pass as DiceFormula::Value
                    let save_bonus = if is_enemy { enemies[target_idx].creature.save_bonus } else { allies[target_idx].creature.save_bonus };
                    let roll = rand::thread_rng().gen_range(1..=20) as f64;
                    
                    if roll + save_bonus < dc {
                        if is_enemy {
                            enemies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                            update_stats_buff(stats, &attacker.id, &enemies[target_idx].id, false);
                        } else {
                            allies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                            update_stats_buff(stats, &attacker.id, &allies[target_idx].id, false);
                        }
                    }
                },

            }
        }
    }
}


pub(crate) fn get_targets(c: &Combattant, action: &Action, allies: &[Combattant], enemies: &[Combattant]) -> Vec<(bool, usize)> {
    #[cfg(debug_assertions)]
    eprintln!("        Getting targets for {}'s action: {}. Allies: {}, Enemies: {}", c.creature.name, action.base().name, allies.len(), enemies.len());
    let mut targets = Vec::new();
    let count = action.base().targets.max(1) as usize;
    
    match action {
        Action::Atk(a) => {
            for i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!("          Attack {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                // For attacks, we allow targeting the same enemy multiple times (e.g. Multiattack, Scorching Ray)
                // So we pass an empty excluded list.
                if let Some(idx) = select_enemy_target(a.target.clone(), enemies, &[], None) {
                    #[cfg(debug_assertions)]
                    eprintln!("            Target selected for {}: Enemy {}", c.creature.name, enemies[idx].creature.name);
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("            No target found for {}'s attack {}.", c.creature.name, i + 1);
                }
            }
        },
        Action::Heal(a) => {
             for i in 0..count {
                 #[cfg(debug_assertions)]
                 eprintln!("          Heal {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                 let self_idx = allies.iter().position(|a| a.id == c.id).unwrap_or(0);
                 if let Some(idx) = select_ally_target(AllyTarget::AllyWithLeastHP, allies, self_idx, &targets, None) {
                     #[cfg(debug_assertions)]
                     eprintln!("            Target selected for {}: Ally {}", c.creature.name, allies[idx].creature.name);
                     targets.push((false, idx));
                 } else {
                     #[cfg(debug_assertions)]
                     eprintln!("            No target found for {}'s heal {}.", c.creature.name, i + 1);
                 }
             }
        },
        Action::Buff(a) => {
            for i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!("          Buff {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                let self_idx = allies.iter().position(|a| a.id == c.id).unwrap_or(0);
                if let Some(idx) = select_ally_target(a.target.clone(), allies, self_idx, &targets, Some(&a.base().id)) {
                    #[cfg(debug_assertions)]
                    eprintln!("            Target selected for {}: Ally {}", c.creature.name, allies[idx].creature.name);
                    targets.push((false, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("            No target found for {}'s buff {}.", c.creature.name, i + 1);
                }
            }
        },
        Action::Debuff(a) => {
            for i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!("          Debuff {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                if let Some(idx) = select_enemy_target(a.target.clone(), enemies, &targets, Some(&a.base().id)) {
                    #[cfg(debug_assertions)]
                    eprintln!("            Target selected for {}: Enemy {}", c.creature.name, enemies[idx].creature.name);
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("            No target found for {}'s debuff {}.", c.creature.name, i + 1);
                }
            }
        },

    }
    #[cfg(debug_assertions)]
    eprintln!("        {} found {} total targets for action {}.", c.creature.name, targets.len(), action.base().name);
    
    targets
}

fn select_enemy_target(strategy: EnemyTarget, enemies: &[Combattant], excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting enemy target (Strategy: {:?}). Enemies available: {}. Excluded: {:?}", strategy, enemies.len(), excluded);
    let mut best_target = None;
    let mut best_val = f64::MAX; 
    
    for (i, e) in enemies.iter().enumerate() {
        // Check exclusion (true = enemy)
        if excluded.contains(&(true, i)) {
            continue;
        }

        // Check buff
        if let Some(bid) = buff_check {
            if e.final_state.buffs.contains_key(bid) {
                continue;
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("              Considering enemy {}. HP: {:.1}", e.creature.name, e.final_state.current_hp);
        if e.final_state.current_hp <= 0.0 {
            #[cfg(debug_assertions)]
            eprintln!("                Enemy {} is dead, skipping.", e.creature.name);
            continue;
        }
        
        let val = match strategy {
            EnemyTarget::EnemyWithLeastHP => e.final_state.current_hp,
            EnemyTarget::EnemyWithMostHP => -e.final_state.current_hp,
            EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e),
            EnemyTarget::EnemyWithLowestAC => e.creature.ac,
            EnemyTarget::EnemyWithHighestAC => -e.creature.ac,
        };
        
        if val < best_val {
            best_val = val;
            best_target = Some(i);
        }
    }
    #[cfg(debug_assertions)]
    eprintln!("            Selected target: {:?}", best_target.map(|idx| enemies[idx].creature.name.clone()));
    
    best_target
}

fn select_ally_target(strategy: AllyTarget, allies: &[Combattant], self_idx: usize, excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting ally target (Strategy: {:?}). Allies available: {}. Excluded: {:?}", strategy, allies.len(), excluded);
    let mut best_target = None;
    let mut best_val = f64::MAX;
    
    // For single-target heals/buffs in multi-target actions, we allow re-targeting the same ally.
    // The previous implementation was designed more for abilities that must hit distinct targets.
    // This removes the `excluded_indices` check.

    if strategy == AllyTarget::Self_ {
        // Only exclude if the self target is explicitly dead (which shouldn't happen for self-buffs)
        if allies[self_idx].final_state.current_hp <= 0.0 {
            #[cfg(debug_assertions)]
            eprintln!("              Self target is dead, skipping.");
            return None;
        } else {
            #[cfg(debug_assertions)]
            eprintln!("              Self target selected.");
            return Some(self_idx);
        }
    }

    for (i, a) in allies.iter().enumerate() {
        // Check exclusion (false = ally)
        if excluded.contains(&(false, i)) {
            continue;
        }

        // Check buff
        if let Some(bid) = buff_check {
            if a.final_state.buffs.contains_key(bid) {
                continue;
            }
        }

        #[cfg(debug_assertions)]
        eprintln!("              Considering ally {}. HP: {:.1}", a.creature.name, a.final_state.current_hp);
        if a.final_state.current_hp <= 0.0 {
            #[cfg(debug_assertions)]
            eprintln!("                Ally {} is dead, skipping.", a.creature.name);
            continue;
        }
        
        let val = match strategy {
            AllyTarget::AllyWithLeastHP => a.final_state.current_hp,
            AllyTarget::AllyWithMostHP => -a.final_state.current_hp,
            AllyTarget::AllyWithHighestDPR => -estimate_dpr(a),
            AllyTarget::AllyWithLowestAC => a.creature.ac,
            AllyTarget::AllyWithHighestAC => -a.creature.ac,
            AllyTarget::Self_ => f64::MAX, // Should be handled above
        };
        
        if val < best_val {
            best_val = val;
            best_target = Some(i);
        }
    }
    #[cfg(debug_assertions)]
    eprintln!("            Selected target: {:?}", best_target.map(|idx| allies[idx].creature.name.clone()));
    
    best_target
}

fn estimate_dpr(c: &Combattant) -> f64 {
    let mut max_dpr = 0.0;
    for action in &c.creature.actions {
        if let Action::Atk(a) = action {
            // Simple estimation: (to_hit - 10) * 0.05 * dpr? 
            // Or just raw DPR.
            // Let's use raw DPR for simplicity as "Highest DPR" usually refers to potential damage.
            // But to be more accurate we could consider to_hit.
            // For now, raw DPR.
            let dpr = match &a.dpr {
                DiceFormula::Value(v) => *v,
                DiceFormula::Expr(e) => dice::parse_average(e),
            };
            if dpr > max_dpr {
                max_dpr = dpr;
            }
        }
    }
    max_dpr
}

fn update_stats(stats: &mut HashMap<String, EncounterStats>, attacker_id: &str, target_id: &str, damage: f64, heal: f64) {
    let attacker_stats = stats.entry(attacker_id.to_string()).or_insert(EncounterStats {
        damage_dealt: 0.0, damage_taken: 0.0, heal_given: 0.0, heal_received: 0.0,
        characters_buffed: 0.0, buffs_received: 0.0, characters_debuffed: 0.0, debuffs_received: 0.0, times_unconscious: 0.0
    });
    attacker_stats.damage_dealt += damage;
    attacker_stats.heal_given += heal;
    
    let target_stats = stats.entry(target_id.to_string()).or_insert(EncounterStats {
        damage_dealt: 0.0, damage_taken: 0.0, heal_given: 0.0, heal_received: 0.0,
        characters_buffed: 0.0, buffs_received: 0.0, characters_debuffed: 0.0, debuffs_received: 0.0, times_unconscious: 0.0
    });
    target_stats.damage_taken += damage;
    target_stats.heal_received += heal;
}

fn update_stats_buff(stats: &mut HashMap<String, EncounterStats>, attacker_id: &str, target_id: &str, is_buff: bool) {
    let attacker_stats = stats.entry(attacker_id.to_string()).or_insert(EncounterStats {
        damage_dealt: 0.0, damage_taken: 0.0, heal_given: 0.0, heal_received: 0.0,
        characters_buffed: 0.0, buffs_received: 0.0, characters_debuffed: 0.0, debuffs_received: 0.0, times_unconscious: 0.0
    });
    if is_buff { attacker_stats.characters_buffed += 1.0; } else { attacker_stats.characters_debuffed += 1.0; }
    
    let target_stats = stats.entry(target_id.to_string()).or_insert(EncounterStats {
        damage_dealt: 0.0, damage_taken: 0.0, heal_given: 0.0, heal_received: 0.0,
        characters_buffed: 0.0, buffs_received: 0.0, characters_debuffed: 0.0, debuffs_received: 0.0, times_unconscious: 0.0
    });
    if is_buff { target_stats.buffs_received += 1.0; } else { target_stats.debuffs_received += 1.0; }
}

fn break_concentration(caster_id: &str, buff_id: &str, allies: &mut [Combattant], enemies: &mut [Combattant]) {
    #[cfg(debug_assertions)]
    eprintln!("        Break Concentration! {} loses concentration on {}.", caster_id, buff_id);

    // Clear concentration on caster
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        if c.id == caster_id {
            c.final_state.concentrating_on = None;
        }
    }

    // Remove buffs from all combatants
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        // We need to check if the buff exists and if it's from this source
        // Since we can't easily iterate and remove, we'll check if it exists first
        let should_remove = if let Some(buff) = c.final_state.buffs.get(buff_id) {
            buff.source.as_ref() == Some(&caster_id.to_string())
        } else {
            false
        };

        if should_remove {
            c.final_state.buffs.remove(buff_id);
            #[cfg(debug_assertions)]
            eprintln!("          Removed {} from {}.", buff_id, c.creature.name);
        }
    }
}

#[cfg(test)]
#[path = "./simulation_test.rs"]
mod simulation_test;
