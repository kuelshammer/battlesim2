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

pub fn aggregate_results(results: &[SimulationResult]) -> Vec<EncounterResult> {
    #[cfg(debug_assertions)]
    eprintln!("AGGREGATION: Starting with {} results", results.len());

    if results.is_empty() {
        return Vec::new();
    }

    // Determine number of encounters from the first result
    let num_encounters = results[0].len();
    let mut aggregated_encounters = Vec::with_capacity(num_encounters);

    for enc_idx in 0..num_encounters {
        // Use first available encounter as template for metadata
        let template_encounter = match results.iter().find_map(|r| r.get(enc_idx)) {
            Some(e) => e,
            None => continue, // Skip if this encounter index is missing in all results
        };

        let max_rounds = results
            .iter()
            .map(|r| r.get(enc_idx).map(|e| e.rounds.len()).unwrap_or(0))
            .max()
            .unwrap_or(0);

        if max_rounds == 0 {
            // Push empty encounter result if no rounds found
            aggregated_encounters.push(EncounterResult {
                stats: HashMap::new(),
                rounds: Vec::new(),
                target_role: template_encounter.target_role.clone(),
            });
            continue;
        }

        let mut aggregated_rounds: Vec<Round> = Vec::with_capacity(max_rounds);
        
        for round_idx in 0..max_rounds {
            let mut team1_map: HashMap<String, AggregationData> = HashMap::new();
            let mut team2_map: HashMap<String, AggregationData> = HashMap::new();
            let mut count = 0;

            for res in results {
                if let Some(encounter) = res.get(enc_idx) {
                    let round_opt = if round_idx < encounter.rounds.len() {
                        encounter.rounds.get(round_idx)
                    } else {
                        encounter.rounds.last()
                    };

                    if let Some(round) = round_opt {
                        count += 1;
                        for c in &round.team1 {
                            let entry = team1_map.entry(c.id.clone()).or_insert_with(AggregationData::new);
                            entry.total_hp += c.final_state.current_hp as f64;
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
                        for c in &round.team2 {
                            let entry = team2_map.entry(c.id.clone()).or_insert_with(AggregationData::new);
                            entry.total_hp += c.final_state.current_hp as f64;
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
                Some(&template_encounter.rounds[round_idx])
            } else {
                template_encounter.rounds.last()
            };

            let template_round = match template_round {
                Some(r) => r,
                None => continue,
            };

            let mut t1 = Vec::new();
            for c_template in &template_round.team1 {
                if let Some(data) = team1_map.get(&c_template.id) {
                    let mut c = c_template.clone();
                    c.final_state.current_hp = (data.total_hp / count as f64).round() as u32;
                    let best_action_json = data.action_counts.iter().max_by_key(|e| e.1).map(|(k,_)| k.as_str()).unwrap_or("[]");
                    c.actions = serde_json::from_str(best_action_json).unwrap_or_default();
                    c.final_state.buffs.clear();
                    for (bid, bcount) in &data.buff_counts {
                        if *bcount > threshold { if let Some(bdef) = data.buff_definitions.get(bid) { c.final_state.buffs.insert(bid.clone(), bdef.clone()); } }
                    }
                    c.final_state.concentrating_on = data.concentration_counts.iter().max_by_key(|e| e.1).and_then(|(cid, ccount)| if *ccount > threshold { Some(cid.clone()) } else { None });
                    if round_idx > 0 { if let Some(prev) = aggregated_rounds.get(round_idx - 1) { if let Some(pc) = prev.team1.iter().find(|x| x.id == c.id) { c.initial_state = pc.final_state.clone(); } } }
                    t1.push(c);
                }
            }

            let mut t2 = Vec::new();
            for c_template in &template_round.team2 {
                if let Some(data) = team2_map.get(&c_template.id) {
                    let mut c = c_template.clone();
                    c.final_state.current_hp = (data.total_hp / count as f64).round() as u32;
                    let best_action_json = data.action_counts.iter().max_by_key(|e| e.1).map(|(k,_)| k.as_str()).unwrap_or("[]");
                    c.actions = serde_json::from_str(best_action_json).unwrap_or_default();
                    c.final_state.buffs.clear();
                    for (bid, bcount) in &data.buff_counts {
                        if *bcount > threshold { if let Some(bdef) = data.buff_definitions.get(bid) { c.final_state.buffs.insert(bid.clone(), bdef.clone()); } }
                    }
                    c.final_state.concentrating_on = data.concentration_counts.iter().max_by_key(|e| e.1).and_then(|(cid, ccount)| if *ccount > threshold { Some(cid.clone()) } else { None });
                    if round_idx > 0 { if let Some(prev) = aggregated_rounds.get(round_idx - 1) { if let Some(pc) = prev.team2.iter().find(|x| x.id == c.id) { c.initial_state = pc.final_state.clone(); } } }
                    t2.push(c);
                }
            }

            // Cleanup
            let mut dead_ids = HashSet::new();
            for c in t1.iter_mut().chain(t2.iter_mut()) {
                if c.final_state.current_hp == 0 {
                    c.final_state.concentrating_on = None;
                    dead_ids.insert(c.id.clone());
                }
            }
            
            // Simplified cleanup for brevity
            for c in t1.iter_mut().chain(t2.iter_mut()) {
                c.final_state.buffs.retain(|_, b| b.source.as_ref().is_none_or(|s| !dead_ids.contains(s)));
            }

            aggregated_rounds.push(Round { team1: t1, team2: t2 });
        }

        aggregated_encounters.push(EncounterResult {
            stats: HashMap::new(), // TODO: Aggregate stats if needed
            rounds: aggregated_rounds,
            target_role: template_encounter.target_role.clone(),
        });
    }

    aggregated_encounters
}

pub fn calculate_score(result: &SimulationResult) -> f64 {
    // If we already have a pre-computed score, use it
    if let Some(s) = result.score {
        return s;
    }

    // Use safe calculation with fallback to -1000000 for compatibility
    crate::safe_aggregation::calculate_score_safe(result).unwrap_or(-1_000_000.0)
}

pub fn calculate_cumulative_score(result: &SimulationResult, encounter_idx: usize) -> f64 {
    let encounter = match result.encounters.get(encounter_idx) {
        Some(e) => e,
        None => return -1_000_000.0,
    };
    
    calculate_encounter_score(encounter)
}

pub fn calculate_encounter_score(encounter: &EncounterResult) -> f64 {
    let last_round = match encounter.rounds.last() {
        Some(r) => r,
        None => return -1_000_000.0,
    };

    let player_hp: f64 = last_round.team1.iter().map(|c| c.final_state.current_hp as f64).sum();
    let monster_hp: f64 = last_round.team2.iter().map(|c| c.final_state.current_hp as f64).sum();
    
    let survivors = last_round.team1.iter().filter(|c| c.final_state.current_hp > 0).count() as f64;
    
    // Calculate total max HP of the party to weight the survivors
    let party_max_hp: f64 = last_round.team1.iter().map(|c| c.creature.hp as f64).sum();
    
    // NEW SCORING FORMULA:
    // 1. Survivors are still paramount. Weight them by the Party's total HP pool. 
    //    This means 1 survivor is worth "1 party of HP".
    // 2. Add actual remaining HP.
    // 3. Subtract Monster HP * 2 to prioritize finishing the fight.
    
    let survival_weight = if party_max_hp > 0.0 { party_max_hp } else { 100.0 };
    
    (survivors * survival_weight * 1000.0) + player_hp - (monster_hp * 2.0)
}

/// Calculate efficiency-aware score based on survival and resource consumption
pub fn calculate_efficiency_score(result: &SimulationResult, events: &[crate::events::Event]) -> f64 {
    // 1. Base Survival Score
    let last_encounter = match result.encounters.last() {
        Some(e) => e,
        None => return -1_000_000.0,
    };
    
    let base_score = calculate_encounter_score(last_encounter);

    // 2. Resource Penalty Calculation
    let mut resource_penalty = 0.0;

    for event in events {
        match event {
            crate::events::Event::SpellCast { spell_level, .. } => {
                // Formula: 15 * (Level ^ 1.6)
                // Lvl 1 = 15, Lvl 3 = 87, Lvl 9 = 500
                let cost = 15.0 * (*spell_level as f64).powf(1.6);
                resource_penalty += cost;
            }
            crate::events::Event::ResourceConsumed { resource_type, amount, .. } => {
                // Identify resources by name/type
                let r_type = resource_type.to_lowercase();
                if r_type.contains("potion") {
                    resource_penalty += *amount * 20.0;
                } else if r_type.contains("action surge") || r_type.contains("ki") || r_type.contains("channel divinity") {
                    resource_penalty += *amount * 20.0; // Short rest
                } else if r_type.contains("rage") || r_type.contains("indomitable") {
                    resource_penalty += *amount * 40.0; // Long rest
                } else if r_type.contains("hitdice") || r_type.contains("hit die") {
                    resource_penalty += *amount * 15.0; // Short rest healing
                }
                // Generic ClassResource penalty if not matched above
                else if r_type.contains("classresource") {
                    resource_penalty += *amount * 20.0;
                }
            }
            _ => {}
        }
    }

    base_score - resource_penalty
}

#[cfg(not(target_arch = "wasm32"))]
pub fn generate_combat_log(result: &SimulationResult) -> String {
    use std::fmt::Write;
    let mut log = String::new();

    for (enc_idx, encounter) in result.encounters.iter().enumerate() {
        writeln!(&mut log, "=== Encounter {} ===\n", enc_idx + 1).unwrap();
        
        // Build ID -> Name map
        let mut id_to_name = HashMap::new();
        if let Some(first_round) = encounter.rounds.first() {
            for c in first_round.team1.iter().chain(first_round.team2.iter()) {
                id_to_name.insert(c.id.clone(), c.creature.name.clone());
            }
        }

        for (i, round) in encounter.rounds.iter().enumerate() {
            writeln!(&mut log, "--- Round {} ---", i + 1).unwrap();

            let mut all_combatants: Vec<&Combattant> =
                round.team1.iter().chain(round.team2.iter()).collect();
            all_combatants.sort_by(|a, b| {
                b.initiative
                    .partial_cmp(&a.initiative)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for c in all_combatants {
                if c.final_state.current_hp == 0 && c.initial_state.current_hp == 0 {
                    continue;
                } // Skip dead

                writeln!(
                    &mut log,
                    "{}: (HP: {} -> {})",
                    c.creature.name, c.initial_state.current_hp, c.final_state.current_hp
                )
                .unwrap();

                if c.actions.is_empty() && c.final_state.current_hp > 0 {
                    writeln!(&mut log, "  - No actions taken.").unwrap();
                }

                for action in &c.actions {
                    let target_names: Vec<String> = action
                        .targets
                        .iter()
                        .map(|(id, count)| {
                            let name = id_to_name.get(id).cloned().unwrap_or_else(|| id.clone());
                            if *count > 1 {
                                format!("{} (x{})", name, count)
                            } else {
                                name
                            }
                        })
                        .collect();
                    writeln!(
                        &mut log,
                        "  - Uses {}: Targets {:?}",
                        action.action.base().name,
                        target_names
                    )
                    .unwrap();
                }
            }
            writeln!(&mut log).unwrap();
        }
        writeln!(&mut log).unwrap();
    }
    log
}
