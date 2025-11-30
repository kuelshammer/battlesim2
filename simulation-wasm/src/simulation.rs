use std::collections::{HashMap, HashSet};
use rand::Rng;
use crate::model::*;
use crate::enums::*;
use crate::dice;
use crate::targeting::*;
use crate::actions::*;
use crate::aggregation::*;
use crate::cleanup::*;

#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;

pub fn run_monte_carlo(players: &[Creature], encounters: &[Encounter], iterations: usize) -> Vec<SimulationResult> {
    let mut results: Vec<(f64, SimulationResult)> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let result = run_single_simulation(players, encounters);
        let score = calculate_score(&result);
        results.push((score, result));
    }

    // Sort by score
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    #[cfg(not(target_arch = "wasm32"))]
    {
        if !results.is_empty() {
            let median_idx = results.len() / 2;
            let (_, median_result) = &results[median_idx];
            let log = generate_combat_log(median_result);
            
            // Try to write to GEMINI_REPORTS if it exists
            let path = std::path::Path::new("./GEMINI_REPORTS");
            if path.exists() {
                 let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                 let filename = path.join(format!("median_run_log_{}.txt", timestamp));
                 if let Ok(mut file) = std::fs::File::create(filename) {
                     let _ = file.write_all(log.as_bytes());
                 }
            } else {
                // Try ../GEMINI_REPORTS
                let path = std::path::Path::new("../GEMINI_REPORTS");
                if path.exists() {
                     let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                     let filename = path.join(format!("median_run_log_{}.txt", timestamp));
                     if let Ok(mut file) = std::fs::File::create(filename) {
                         let _ = file.write_all(log.as_bytes());
                     }
                }
            }
        }
    }

    // Return all results sorted by score
    results.into_iter().map(|(_, r)| r).collect()
}

fn run_single_simulation(players: &[Creature], encounters: &[Encounter]) -> SimulationResult {
    let mut results = Vec::new();
    
    // Initialize players with state
    // We iterate to capture group index for deterministic IDs
    let mut players_with_state = Vec::new();
    for (group_idx, player) in players.iter().enumerate() {
        for i in 0..player.count as i32 {
             let name = if player.count > 1.0 { format!("{} {}", player.name, i + 1) } else { player.name.clone() };
             let mut p = player.clone();
             p.name = name;
             // ID format: {template_id}-{group_idx}-{index} to be absolutely sure of uniqueness
             let id = format!("{}-{}-{}", player.id, group_idx, i);
             players_with_state.push(create_combattant(p, id));
        }
    }

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

fn create_combattant(creature: Creature, id: String) -> Combattant {
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
        id,
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
    
    roll + c.initiative_bonus
}

fn run_encounter(players: &[Combattant], encounter: &Encounter) -> EncounterResult {
    let mut team2 = Vec::new();
    for (group_idx, monster) in encounter.monsters.iter().enumerate() {
        for i in 0..monster.count as i32 {
            let name = if monster.count > 1.0 { format!("{} {}", monster.name, i + 1) } else { monster.name.clone() };
            let mut m = monster.clone();
            m.name = name;
            // ID format: {template_id}-{group_idx}-{index}
            let id = format!("{}-{}-{}", monster.id, group_idx, i);
            team2.push(create_combattant(m, id));
        }
    }
    
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
    for (team_id, idx, _initiative_value) in turn_order {
        let _combatant_name = match team_id {
            TeamId::Team1 => t1[idx].creature.name.clone(),
            TeamId::Team2 => t2[idx].creature.name.clone(),
        };
        #[cfg(debug_assertions)]
        eprintln!("  Turn for: {} (Init: {:.1})", _combatant_name, _initiative_value);

        // Check if creature is still alive (might have died in previous turn)
        let is_alive = match team_id {
            TeamId::Team1 => t1[idx].final_state.current_hp > 0.0,
            TeamId::Team2 => t2[idx].final_state.current_hp > 0.0,
        };
        if !is_alive {
            #[cfg(debug_assertions)]
            eprintln!("    {} is dead, skipping turn.", _combatant_name);
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
        eprintln!("  {} turn END. Current State: P1 HP: {:.1}, P2 HP: {:.1}", _combatant_name, t1[0].final_state.current_hp, t2[0].final_state.current_hp);
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
            let _current_target_hp = if is_enemy { enemies[target_idx].final_state.current_hp } else { allies[target_idx].final_state.current_hp };
            #[cfg(debug_assertions)]
            eprintln!("          Executing action {} by {} on {}. Target HP: {:.1}", action.base().name, attacker.creature.name, target_name, _current_target_hp);

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

#[cfg(test)]
#[path = "./simulation_test.rs"]
mod simulation_test;
