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

    for i in 0..iterations {
        let log_enabled = i == 0;
        let (result, run_log) = run_single_simulation(players, encounters, log_enabled);
        let score = calculate_score(&result);
        results.push((score, result));

        if log_enabled {
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Write detailed log to file
                let path = std::path::Path::new("./GEMINI_REPORTS");
                if path.exists() {
                     let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                     let filename = path.join(format!("detailed_run_log_{}.txt", timestamp));
                     if let Ok(mut file) = std::fs::File::create(filename) {
                         for line in run_log {
                             let _ = writeln!(file, "{}", line);
                         }
                     }
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                // For WASM, we can't write to file easily, but we can log to console
                if !run_log.is_empty() {
                    web_sys::console::log_1(&"--- DETAILED SIMULATION LOG (First Run) ---".into());
                    // Log in chunks to avoid browser limits if needed, or just summary
                    // For now, let's log the first 100 lines
                    for line in run_log.iter().take(100) {
                        web_sys::console::log_1(&line.into());
                    }
                    if run_log.len() > 100 {
                        web_sys::console::log_1(&format!("... and {} more lines", run_log.len() - 100).into());
                    }
                }
            }
        }
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
                 let filename = path.join(format!("median_run_summary_{}.txt", timestamp));
                 if let Ok(mut file) = std::fs::File::create(filename) {
                     let _ = file.write_all(log.as_bytes());
                 }
            }
        }
    }

    // Return all results sorted by score
    results.into_iter().map(|(_, r)| r).collect()
}

fn run_single_simulation(players: &[Creature], encounters: &[Encounter], log_enabled: bool) -> (SimulationResult, Vec<String>) {
    let mut results = Vec::new();
    let mut log = Vec::new();
    
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
        let encounter_result = run_encounter(&players_with_state, encounter, &mut log, log_enabled);
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

    (results, log)
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

fn run_encounter(players: &[Combattant], encounter: &Encounter, log: &mut Vec<String>, log_enabled: bool) -> EncounterResult {
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
    
    if log_enabled {
        log.push(format!("--- Encounter Start: Players vs Monsters ---"));
    }

    let max_rounds = 100;
    for i in 0..max_rounds {
        if !team1.iter().any(|c| c.final_state.current_hp > 0.0) || !team2.iter().any(|c| c.final_state.current_hp > 0.0) {
            break;
        }
        
        let round = run_round(&team1, &team2, &mut stats, log, log_enabled, i + 1);
        rounds.push(round.clone());
        
        team1 = round.team1;
        team2 = round.team2;
    }
    
    EncounterResult {
        stats,
        rounds,
    }
}

fn run_round(team1: &[Combattant], team2: &[Combattant], stats: &mut HashMap<String, EncounterStats>, log: &mut Vec<String>, log_enabled: bool, round_num: usize) -> Round {
    if log_enabled {
        log.push(format!("\n=== Round {} ===", round_num));
    }

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
            TeamId::Team1 => execute_turn(idx, &mut t1, &mut t2, stats, false, log, log_enabled),
            TeamId::Team2 => execute_turn(idx, &mut t2, &mut t1, stats, true, log, log_enabled),
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

#[allow(dead_code)]
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


fn execute_turn(index: usize, allies: &mut [Combattant], enemies: &mut [Combattant], stats: &mut HashMap<String, EncounterStats>, _is_enemy: bool, log: &mut Vec<String>, log_enabled: bool) {
    let attacker = allies[index].clone();
    #[cfg(debug_assertions)]
    eprintln!("    Executing turn for {}", attacker.creature.name);

    if log_enabled {
        log.push(format!("  > Turn: {} (HP: {:.1})", attacker.creature.name, attacker.final_state.current_hp));
    }

    // Get actions
    let actions = get_actions(&attacker, allies, enemies);
    
    if actions.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("      No actions available.");
        if log_enabled {
            log.push(format!("    - No actions available."));
        }
        return;
    }
    
    // Choose action (simple logic: random usable)
    let mut rng = rand::thread_rng();
    let action = &actions[rng.gen_range(0..actions.len())];
    
    #[cfg(debug_assertions)]
    eprintln!("      Chose action: {}", action.base().name);
    
    if log_enabled {
        log.push(format!("    - Uses Action: {}", action.base().name));
    }

    // Mark action as used
    allies[index].final_state.used_actions.insert(action.base().id.clone());
    
    // Resolve targets
    let raw_targets = get_targets(&attacker, action, allies, enemies);
    
    #[cfg(debug_assertions)]
    eprintln!("      Selected {} targets.", raw_targets.len());
    
    // Record action in history (Aggregation)
    let mut action_record = CombattantAction {
        action: action.clone(),
        targets: HashMap::new(),
    };
    
    for (is_target_enemy, target_idx) in &raw_targets {
        let target_id = if *is_target_enemy { &enemies[*target_idx].id } else { &allies[*target_idx].id };
        *action_record.targets.entry(target_id.clone()).or_insert(0) += 1;
    }
    allies[index].actions.push(action_record);

    // Apply effects
    for (is_target_enemy, target_idx) in raw_targets {
        match action {
            Action::Atk(a) => {
                let target_name = if is_target_enemy { enemies[target_idx].creature.name.clone() } else { allies[target_idx].creature.name.clone() };
                
                // Attack Roll
                let (roll, is_crit, is_miss) = get_attack_roll_result(&attacker);
                let to_hit_bonus = dice::evaluate(&a.to_hit, 1);
                // Add buff bonuses to hit
                let buff_bonus: f64 = attacker.final_state.buffs.values()
                    .filter_map(|b| b.to_hit.as_ref().map(|f| dice::evaluate(f, 1)))
                    .sum();
                
                let total_hit = roll + to_hit_bonus + buff_bonus;
                let target_ac = if is_target_enemy { enemies[target_idx].creature.ac } else { allies[target_idx].creature.ac };
                // Add buff bonuses to AC
                let target_buff_ac: f64 = if is_target_enemy {
                    enemies[target_idx].final_state.buffs.values().filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1))).sum()
                } else {
                    allies[target_idx].final_state.buffs.values().filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1))).sum()
                };
                let total_ac = target_ac + target_buff_ac;

                let hits = !is_miss && (is_crit || total_hit >= total_ac);
                
                if log_enabled {
                    let crit_str = if is_crit { " (CRIT!)" } else if is_miss { " (MISS!)" } else { "" };
                    log.push(format!("      -> Attack vs {}: Rolled {:.0} + {:.0} (bonus) + {:.0} (buffs) = {:.0} vs AC {:.0} (Base {:.0} + {:.0} buffs){}. Result: {}", 
                        target_name, roll, to_hit_bonus, buff_bonus, total_hit, total_ac, target_ac, target_buff_ac, crit_str, if hits { "HIT" } else { "MISS" }));
                }

                if hits {
                    let mut damage = dice::evaluate(&a.dpr, if is_crit { 2 } else { 1 });
                    // Add buff damage bonuses
                    let buff_dmg: f64 = attacker.final_state.buffs.values()
                        .filter_map(|b| b.damage.as_ref().map(|f| dice::evaluate(f, 1)))
                        .sum();
                    damage += buff_dmg;
                    
                    if log_enabled {
                            log.push(format!("         Damage: {:.0} (Base) + {:.0} (Buffs) = {:.0}", damage - buff_dmg, buff_dmg, damage));
                    }

                    if is_target_enemy {
                        enemies[target_idx].final_state.current_hp -= damage;
                        if enemies[target_idx].final_state.current_hp < 0.0 {
                            enemies[target_idx].final_state.current_hp = 0.0;
                        }
                        update_stats(stats, &attacker.id, &enemies[target_idx].id, damage, 0.0);
                        
                        // Concentration check
                        if enemies[target_idx].final_state.current_hp <= 0.0 {
                            remove_all_buffs_from_source(&enemies[target_idx].id.clone(), allies, enemies);
                            if log_enabled { log.push(format!("         {} died!", enemies[target_idx].creature.name)); }
                        } else if let Some(buff_id) = enemies[target_idx].final_state.concentrating_on.clone() {
                            let dc = (damage / 2.0).max(10.0);
                            let con_save = dice::evaluate(&DiceFormula::Expr("1d20".to_string()), 1); 
                            let bonus = enemies[target_idx].creature.con_save_bonus.unwrap_or(0.0);
                            
                            if con_save + bonus < dc {
                                break_concentration(&enemies[target_idx].id.clone(), &buff_id, allies, enemies);
                                if log_enabled { log.push(format!("         (Drops concentration on {})", buff_id)); }
                            }
                        }
                    } else {
                        allies[target_idx].final_state.current_hp -= damage;
                        if allies[target_idx].final_state.current_hp < 0.0 {
                            allies[target_idx].final_state.current_hp = 0.0;
                        }
                        update_stats(stats, &attacker.id, &allies[target_idx].id, damage, 0.0);
                            if allies[target_idx].final_state.current_hp <= 0.0 {
                            remove_all_buffs_from_source(&allies[target_idx].id.clone(), allies, enemies);
                            if log_enabled { log.push(format!("         {} died!", allies[target_idx].creature.name)); }
                        } else if let Some(buff_id) = allies[target_idx].final_state.concentrating_on.clone() {
                            let dc = (damage / 2.0).max(10.0);
                            let con_save = dice::evaluate(&DiceFormula::Expr("1d20".to_string()), 1);
                            let bonus = allies[target_idx].creature.con_save_bonus.unwrap_or(0.0);
                            
                            if con_save + bonus < dc {
                                break_concentration(&allies[target_idx].id.clone(), &buff_id, allies, enemies);
                                if log_enabled { log.push(format!("         (Drops concentration on {})", buff_id)); }
                            }
                        }
                    }
                }
            },
            Action::Heal(a) => {
                    let amount = dice::evaluate(&a.amount, 1);
                    if is_target_enemy {
                        // Enemy healing enemy (ally of attacker)
                        let target_name = enemies[target_idx].creature.name.clone();
                        enemies[target_idx].final_state.current_hp += amount;
                        if enemies[target_idx].final_state.current_hp > enemies[target_idx].creature.hp {
                            enemies[target_idx].final_state.current_hp = enemies[target_idx].creature.hp;
                        }
                        update_stats(stats, &attacker.id, &enemies[target_idx].id, 0.0, amount);
                        if log_enabled {
                            log.push(format!("      -> Heals {} for {:.0} HP", target_name, amount));
                        }
                    } else {
                        // Player healing player
                        let target_name = allies[target_idx].creature.name.clone();
                        allies[target_idx].final_state.current_hp += amount;
                        if allies[target_idx].final_state.current_hp > allies[target_idx].creature.hp {
                            allies[target_idx].final_state.current_hp = allies[target_idx].creature.hp;
                        }
                        update_stats(stats, &attacker.id, &allies[target_idx].id, 0.0, amount);
                        if log_enabled {
                            log.push(format!("      -> Heals {} for {:.0} HP", target_name, amount));
                        }
                    }
            },
            Action::Buff(a) => {
                if log_enabled {
                    let target_name = if is_target_enemy { enemies[target_idx].creature.name.clone() } else { allies[target_idx].creature.name.clone() };
                    log.push(format!("      -> Applies Buff {} to {}", a.buff.display_name.as_deref().unwrap_or("Unknown"), target_name));
                }
                if a.buff.concentration {
                    let current_concentration = allies[index].final_state.concentrating_on.clone();
                    if let Some(old_buff) = current_concentration {
                            let caster_id = allies[index].id.clone();
                            break_concentration(&caster_id, &old_buff, allies, enemies);
                            if log_enabled { log.push(format!("         (Drops concentration on {})", old_buff)); }
                    }
                    allies[index].final_state.concentrating_on = Some(a.base().id.clone());
                }
                let mut buff = a.buff.clone();
                buff.source = Some(attacker.id.clone());
                if is_target_enemy {
                    enemies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &enemies[target_idx].id, true);
                } else {
                    allies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &allies[target_idx].id, true);
                }
            },
            Action::Debuff(a) => {
                    let target_name = if is_target_enemy { enemies[target_idx].creature.name.clone() } else { allies[target_idx].creature.name.clone() };
                    
                    if a.buff.concentration {
                    let current_concentration = allies[index].final_state.concentrating_on.clone();
                    if let Some(old_buff) = current_concentration {
                            let caster_id = allies[index].id.clone();
                            break_concentration(&caster_id, &old_buff, allies, enemies);
                            if log_enabled { log.push(format!("         (Drops concentration on {})", old_buff)); }
                    }
                    allies[index].final_state.concentrating_on = Some(a.base().id.clone());
                }
                
                let dc_val = a.save_dc;
                let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1);
                let save_bonus = if is_target_enemy { enemies[target_idx].creature.save_bonus } else { allies[target_idx].creature.save_bonus };
                let roll = rand::thread_rng().gen_range(1..=20) as f64;
                
                if log_enabled {
                    log.push(format!("      -> Debuff {} vs {}: DC {:.0} vs Save {:.0} (Rolled {:.0} + {:.0})", 
                        a.buff.display_name.as_deref().unwrap_or("Unknown"), target_name, dc, roll + save_bonus, roll, save_bonus));
                }

                if roll + save_bonus < dc {
                    let mut buff = a.buff.clone();
                    buff.source = Some(attacker.id.clone());
                    if is_target_enemy {
                        enemies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &enemies[target_idx].id, false);
                    } else {
                        allies[target_idx].final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &allies[target_idx].id, false);
                    }
                    if log_enabled { log.push(format!("         Failed! Debuff applied.")); }
                } else {
                    if log_enabled { log.push(format!("         Saved!")); }
                }
            },
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
