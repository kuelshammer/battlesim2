use crate::model::*;
use std::collections::{HashMap, HashSet};

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
    #[cfg(debug_assertions)]
    eprintln!("AGGREGATION: Starting with {} results", results.len());

    if results.is_empty() { return Vec::new(); }
    
    let max_rounds = results.iter().map(|r| r.first().map(|e| e.rounds.len()).unwrap_or(0)).max().unwrap_or(0);
    
    #[cfg(debug_assertions)]
    eprintln!("AGGREGATION: max_rounds = {}", max_rounds);

    let mut aggregated_rounds: Vec<Round> = Vec::with_capacity(max_rounds);
    
    let template_encounter = results.first().and_then(|r| r.first());
    if template_encounter.is_none() { 
        #[cfg(debug_assertions)]
        eprintln!("AGGREGATION: No template encounter found!");
        return Vec::new(); 
    }
    let template_encounter = template_encounter.unwrap();
    
    for round_idx in 0..max_rounds {
        let mut team1_map: HashMap<String, AggregationData> = HashMap::new();
        let mut team2_map: HashMap<String, AggregationData> = HashMap::new();
        let mut count = 0;
        
        for res in results {
            if let Some(encounter) = res.first() {
                // Determine which round to use (actual or last padding)
                let round_opt = if round_idx < encounter.rounds.len() {
                    encounter.rounds.get(round_idx)
                } else {
                    encounter.rounds.last()
                };

                if let Some(round) = round_opt {
                    count += 1;
                    
                    for c in &round.team1 {
                        // With deterministic IDs, c.id is stable across runs.
                        let entry = team1_map.entry(c.id.clone()).or_insert_with(AggregationData::new);
                        entry.total_hp += c.final_state.current_hp;
                        
                        let action_key = serde_json::to_string(&c.actions).unwrap_or_default();
                        *entry.action_counts.entry(action_key).or_insert(0) += 1;

                        // Aggregate Buffs
                        for (buff_id, buff) in &c.final_state.buffs {
                            *entry.buff_counts.entry(buff_id.clone()).or_insert(0) += 1;
                            entry.buff_definitions.entry(buff_id.clone()).or_insert_with(|| buff.clone());
                        }

                        // Aggregate Concentration
                        if let Some(conc_id) = &c.final_state.concentrating_on {
                            *entry.concentration_counts.entry(conc_id.clone()).or_insert(0) += 1;
                        }
                    }
                    
                    // Team 2
                    for c in &round.team2 {
                        let entry = team2_map.entry(c.id.clone()).or_insert_with(AggregationData::new);
                        entry.total_hp += c.final_state.current_hp;
                        
                        let action_key = serde_json::to_string(&c.actions).unwrap_or_default();
                        *entry.action_counts.entry(action_key).or_insert(0) += 1;

                        for (buff_id, buff) in &c.final_state.buffs {
                            *entry.buff_counts.entry(buff_id.clone()).or_insert(0) += 1;
                            entry.buff_definitions.entry(buff_id.clone()).or_insert_with(|| buff.clone());
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

        let template_round = if round_idx < template_encounter.rounds.len() {
            &template_encounter.rounds[round_idx]
        } else {
            template_encounter.rounds.last().unwrap()
        };
        
        // Reconstruct Team 1
        let mut t1 = Vec::new();
        for c_template in &template_round.team1 {
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
                 if let Some((conc_id, conc_count)) = data.concentration_counts.iter().max_by_key(|e| e.1) {
                     if *conc_count > threshold {
                         c.final_state.concentrating_on = Some(conc_id.clone());
                     }
                 }
                 
                 // Fix initial_state
                 if round_idx > 0 {
                     if let Some(prev_round) = aggregated_rounds.get(round_idx - 1) {
                         if let Some(prev_c) = prev_round.team1.iter().find(|pc| pc.id == c.id) {
                             c.initial_state = prev_c.final_state.clone();
                         }
                     }
                 }
                 
                 t1.push(c);
             }
        }
        
        // Reconstruct Team 2
        let mut t2 = Vec::new();
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
                         if let Some(prev_c) = prev_round.team2.iter().find(|pc| pc.id == c.id) {
                             c.initial_state = prev_c.final_state.clone();
                         }
                     }
                 }
                 
                 t2.push(c);
             }
        }
        
        // Consistency Cleanup: Enforce "Dead = No Concentration" on the aggregated result
        let mut dead_source_ids = HashSet::new();
        
        // 1. Identify effectively dead combatants and clear their concentration
        for c in t1.iter_mut().chain(t2.iter_mut()) {
            if c.final_state.current_hp < 0.5 {
                #[cfg(debug_assertions)]
                eprintln!("AGGREGATION: {} is dead (HP: {:.1}). Clearing concentration.", c.creature.name, c.final_state.current_hp);
                
                if c.final_state.concentrating_on.is_some() {
                    c.final_state.concentrating_on = None;
                }
                dead_source_ids.insert(c.id.clone());
            }
        }
        
        // 2. Build a map of who is concentrating on what
        let mut concentration_map: HashMap<String, Option<String>> = HashMap::new(); // caster_id -> buff_id
        for c in t1.iter().chain(t2.iter()) {
            concentration_map.insert(c.id.clone(), c.final_state.concentrating_on.clone());
        }
        
        // 3. Multi-pass buff cleanup for comprehensive dead source handling
        if !dead_source_ids.is_empty() || !concentration_map.is_empty() {
            #[cfg(debug_assertions)]
            eprintln!("AGGREGATION: Starting comprehensive cleanup. Dead sources: {}, concentration_map: {:?}",
                dead_source_ids.len(), concentration_map.len());

            // First pass: Remove buffs from clearly dead sources (HP <= 0.0)
            for c in t1.iter_mut().chain(t2.iter_mut()) {
                let _before_count = c.final_state.buffs.len();
                c.final_state.buffs.retain(|_buff_id, buff| {
                    if let Some(source) = &buff.source {
                        if dead_source_ids.contains(source) {
                            #[cfg(debug_assertions)]
                            eprintln!("AGGREGATION: PASS1: Removing buff {} from {} (source {} is dead, HP: {:.1})",
                                _buff_id, c.creature.name, source, c.final_state.current_hp);
                            return false;
                        }
                        true
                    } else {
                        // Buff with no source is always kept (might be innate effects)
                        true
                    }
                });
                let _after_count = c.final_state.buffs.len();

                #[cfg(debug_assertions)]
                if _before_count != _after_count {
                    eprintln!("AGGREGATION: PASS1: {} had {} buffs, now has {}", c.creature.name, _before_count, _after_count);
                }
            }

            // Second pass: Handle concentration-specific cleanup for remaining alive casters
            if !concentration_map.is_empty() {
                #[cfg(debug_assertions)]
                eprintln!("AGGREGATION: PASS2: Checking concentration mechanics for sources: {:?}", concentration_map);

                for c in t1.iter_mut().chain(t2.iter_mut()) {
                    let _before_count = c.final_state.buffs.len();
                    c.final_state.buffs.retain(|buff_id, buff| {
                        if let Some(source) = &buff.source {
                            // Skip if already handled in first pass
                            if dead_source_ids.contains(source) {
                                return false;
                            }
                        
                        // Handle concentration buffs specifically
                            if buff.concentration {
                                if let Some(source_concentrating) = concentration_map.get(source) {
                                    let is_concentrating_on_this = source_concentrating.as_ref() == Some(buff_id);
                                    if !is_concentrating_on_this {
                                    #[cfg(debug_assertions)]
                                        eprintln!("AGGREGATION: Removing buff {} from {} (source {} not concentrating on it, concentrating on: {:?})",
                                            buff_id, c.creature.name, source, source_concentrating);
                                        return false;
                                    }
                                    // Concentration buff is valid - keep it
                                    true
                                } else {
                                    #[cfg(debug_assertions)]
                                    eprintln!("AGGREGATION: Removing concentration buff {} from {} (source {} not in concentration map)",
                                        buff_id, c.creature.name, source);
                                    false
                                }
                            } else {
                                // Non-concentration buff from alive source - keep it
                                true
                            }
                        } else {
                            // Buff with no source is always kept (innate effects)
                            true
                        }
                });
                let _after_count = c.final_state.buffs.len();

                    #[cfg(debug_assertions)]
                    if _before_count != _after_count {
                        eprintln!("AGGREGATION: PASS2: {} had {} concentration-related buffs, now has {}", c.creature.name, _before_count, _after_count);
                    }
                }
            }
        }

        aggregated_rounds.push(Round { team1: t1, team2: t2 });
    }
    
    aggregated_rounds
}

pub fn calculate_score(result: &SimulationResult) -> f64 {
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

#[cfg(not(target_arch = "wasm32"))]
pub fn generate_combat_log(result: &SimulationResult) -> String {
    use std::fmt::Write;
    let mut log = String::new();
    
    if let Some(encounter) = result.first() {
        // Build ID -> Name map
        let mut id_to_name = HashMap::new();
        if let Some(first_round) = encounter.rounds.first() {
            for c in first_round.team1.iter().chain(first_round.team2.iter()) {
                id_to_name.insert(c.id.clone(), c.creature.name.clone());
            }
        }

        for (i, round) in encounter.rounds.iter().enumerate() {
            writeln!(&mut log, "--- Round {} ---", i + 1).unwrap();
            
            let mut all_combatants: Vec<&Combattant> = round.team1.iter().chain(round.team2.iter()).collect();
            all_combatants.sort_by(|a, b| b.initiative.partial_cmp(&a.initiative).unwrap_or(std::cmp::Ordering::Equal));
            
            for c in all_combatants {
                if c.final_state.current_hp <= 0.0 && c.initial_state.current_hp <= 0.0 { continue; } // Skip dead
                
                writeln!(&mut log, "{}: (HP: {:.1} -> {:.1})", c.creature.name, c.initial_state.current_hp, c.final_state.current_hp).unwrap();
                
                if c.actions.is_empty() && c.final_state.current_hp > 0.0 {
                    writeln!(&mut log, "  - No actions taken.").unwrap();
                }

                for action in &c.actions {
                    let target_names: Vec<String> = action.targets.iter().map(|(id, count)| {
                         let name = id_to_name.get(id).cloned().unwrap_or_else(|| id.clone());
                         if *count > 1 { format!("{} (x{})", name, count) } else { name }
                    }).collect();
                    writeln!(&mut log, "  - Uses {}: Targets {:?}", action.action.base().name, target_names).unwrap();
                }
            }
            writeln!(&mut log).unwrap();
        }
    }
    log
}
