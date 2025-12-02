use std::collections::{HashMap, HashSet};
use rand::Rng;
use crate::model::*;
use crate::enums::*;
use crate::dice;
// use crate::targeting::*; // Unused if execute_turn doesn't do targeting directly? No, it does get_targets.
use crate::targeting::get_targets; 
use crate::actions::*;
use crate::aggregation::*;
use crate::cleanup::*;
use crate::resolution; // New module

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

/// Execute all pre-combat actions (actionSlot: -3) before combat begins
/// This allows spells like Mage Armour, Armor of Agathys, etc. to be cast before initiative
fn execute_precombat_actions(
    team1: &mut [Combattant],
    team2: &mut [Combattant],
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
) {
    let mut has_precombat = false;
    
    // Check if any actions need to be executed
    for combattant in team1.iter().chain(team2.iter()) {
        if combattant.creature.actions.iter().any(|a| a.base().action_slot == -3) {
            has_precombat = true;
            break;
        }
    }
    
    if !has_precombat {
        return;
    }
    
    if log_enabled {
        log.push("".to_string());
        log.push("=== Pre-Combat Setup ===".to_string());
    }
    
    // Execute pre-combat actions for team1 (players)
    for attacker_index in 0..team1.len() {
        let precombat_actions: Vec<_> = team1[attacker_index]
            .creature
            .actions
            .iter()
            .filter(|a| a.base().action_slot == -3)
            .cloned()
            .collect();
        
        for action in precombat_actions {
            if log_enabled {
                log.push(format!("  > {} uses {}", team1[attacker_index].creature.name, action.base().name));
            }
            
            // Get targets for the action (correct signature: combattant, action, allies, enemies)
            let targets = get_targets(&team1[attacker_index], &action, &team1, &team2);
            
            // Create action record
            let mut action_record = CombattantAction {
                action: action.clone(),
                targets: HashMap::new(),
            };
           
            for (is_target_enemy, target_idx) in &targets {
                let target_id = if *is_target_enemy { &team2[*target_idx].id } else { &team1[*target_idx].id };
                *action_record.targets.entry(target_id.clone()).or_insert(0) += 1;
            }
            
            // Execute action on all targets
            let cleanup = resolution::resolve_action_execution(
                attacker_index,
                team1,
                team2,
                &action,
                &targets,
                &action_record,
                stats,
                log,
                log_enabled,
            );
            
            // Process cleanup instructions
            for instruction in cleanup {
                match instruction {
                    CleanupInstruction::RemoveAllBuffsFromSource(source_id) => {
                        remove_all_buffs_from_source(&source_id, team1, team2);
                    },
                    CleanupInstruction::BreakConcentration(combatant_id, buff_id) => {
                        break_concentration(&combatant_id, &buff_id, team1, team2);
                    },
                }
            }
        }
    }
    
    // Execute pre-combat actions for team2 (monsters)
    for attacker_index in 0..team2.len() {
        let precombat_actions: Vec<_> = team2[attacker_index]
            .creature
            .actions
            .iter()
            .filter(|a| a.base().action_slot == -3)
            .cloned()
            .collect();
        
        for action in precombat_actions {
            if log_enabled {
                log.push(format!("  > {} uses {}", team2[attacker_index].creature.name, action.base().name));
            }
            
            // Get targets for the action (correct signature: combattant, action, allies, enemies)
            // Note: from team2's perspective, team2 is allies and team1 is enemies
            let targets = get_targets(&team2[attacker_index], &action, &team2, &team1);
            
            // Create action record
            let mut action_record = CombattantAction {
                action: action.clone(),
                targets: HashMap::new(),
            };
            
            for (is_target_enemy, target_idx) in &targets {
                let target_id = if *is_target_enemy { &team1[*target_idx].id } else { &team2[*target_idx].id };
                *action_record.targets.entry(target_id.clone()).or_insert(0) += 1;
            }
            
            // Execute action on all targets
            let cleanup = resolution::resolve_action_execution(
                attacker_index,
                team2,
                team1,  // Note: reversed for team2's perspective
                &action,
                &targets,
                &action_record,
                stats,
                log,
                log_enabled,
            );
            
            // Process cleanup instructions
            for instruction in cleanup {
                match instruction {
                    CleanupInstruction::RemoveAllBuffsFromSource(source_id) => {
                        remove_all_buffs_from_source(&source_id, team2, team1);
                    },
                    CleanupInstruction::BreakConcentration(combatant_id, buff_id) => {
                        break_concentration(&combatant_id, &buff_id, team2, team1);
                    },
                }
            }
        }
    }
    
    if log_enabled {
        log.push("".to_string());
    }
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

    // NEW: Execute pre-combat actions (actionSlot: -3) before combat begins
    execute_precombat_actions(&mut team1, &mut team2, &mut stats, log, log_enabled);

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
    enum TeamId { Team1, Team2 } // This enum is defined inside run_round
    
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
            _ => {} // Other durations are handled elsewhere or not relevant for start of round
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

// Simplified execute_turn delegating to resolution logic
fn execute_turn(index: usize, allies: &mut [Combattant], enemies: &mut [Combattant], stats: &mut HashMap<String, EncounterStats>, _is_enemy: bool, log: &mut Vec<String>, log_enabled: bool) {
    // Log the turn
        let attacker_name_for_log = allies[index].creature.name.clone();
        log.push(format!("  > Turn: {} (HP: {:.0} of {:.0})", attacker_name_for_log, allies[index].final_state.current_hp, allies[index].creature.hp));

    // Get actions
    let actions = get_actions(&allies[index], allies, enemies);
    
    if actions.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("      No actions available.");
        if log_enabled {
            log.push(format!("    - No actions available."));
        }
        return;
    }
    
    // Choose actions according to D&D 5e action economy
    let _rng = rand::thread_rng();

    // Sort actions: 
    // 1. Priority based on Frequency (Limited > Recharge > At Will)
    // 2. Buffs first, then Attacks
    let mut sorted_actions = actions.clone();
    sorted_actions.sort_by(|a, b| {
        let score_a = get_action_priority(&a.base().freq);
        let score_b = get_action_priority(&b.base().freq);
        
        if score_a != score_b {
             // Higher score comes first (Descending priority)
             return score_b.cmp(&score_a);
        }

        match (a, b) {
            (Action::Buff(_), Action::Atk(_)) => std::cmp::Ordering::Less,
            (Action::Atk(_), Action::Buff(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Execute up to 2 actions (1 Action + 1 Bonus Action)
    let mut used_slots = std::collections::HashSet::new();
    let mut actions_to_execute = Vec::new();

    // Find first two available actions with different slots
    for action in &sorted_actions {
        let action_slot = action.base().action_slot;
        if !used_slots.contains(&action_slot) {
            used_slots.insert(action_slot);
            actions_to_execute.push(action);
        }
    }

    // Execute all selected actions
    for action in &actions_to_execute {
        #[cfg(debug_assertions)]
        eprintln!("      Chose action: {}", action.base().name);

        if log_enabled {
            log.push(format!("    - Uses Action: {}", action.base().name));
        }

        // Resolve targets (this takes an immutable attacker and returns indices, so it's fine)
        let raw_targets = get_targets(&allies[index], action, allies, enemies);

        #[cfg(debug_assertions)]
        eprintln!("      Selected {} targets.", raw_targets.len());

        // Record action in history (Aggregation) - this requires a clone of the action
        let mut action_record = CombattantAction {
            action: (*action).clone(),
            targets: HashMap::new(),
        };

        for (is_target_enemy, target_idx) in &raw_targets {
            let target_id = if *is_target_enemy { &enemies[*target_idx].id } else { &allies[*target_idx].id };
            *action_record.targets.entry(target_id.clone()).or_insert(0) += 1;
        }

        // Delegate execution mechanics to the resolution module
        // This handles slice splitting, mutable borrowing, and effect application including triggers
        let instructions = resolution::resolve_action_execution(
            index,
            allies,
            enemies,
            action,
            &raw_targets,
            &action_record,
            stats,
            log,
            log_enabled
        );



        // Process returned cleanup instructions
        for instruction in instructions {
            match instruction {
                CleanupInstruction::RemoveAllBuffsFromSource(source_id) => {
                    remove_all_buffs_from_source(&source_id, allies, enemies);
                },
                CleanupInstruction::BreakConcentration(combatant_id, buff_id) => {
                    break_concentration(&combatant_id, &buff_id, allies, enemies);
                },
            }
        }
    }
}

fn get_action_priority(freq: &Frequency) -> i32 {
    match freq {
        Frequency::Limited { .. } => 3,
        Frequency::Recharge { .. } => 2,
        Frequency::Static(s) => if s == "at will" { 1 } else { 3 },
    }
}

#[cfg(test)]
#[path = "./simulation_test.rs"]
mod simulation_test;