use crate::model::*;
use crate::enums::*;
use crate::dice;
use std::cmp::Ordering;

pub fn get_targets(c: &Combattant, action: &Action, allies: &[Combattant], enemies: &[Combattant]) -> Vec<(bool, usize)> {
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
             // First check if any allies actually need healing
             let injured_ally_idx = allies.iter().position(|ally| {
                 ally.id != c.id && ally.final_state.current_hp < ally.creature.hp
             });

             if injured_ally_idx.is_none() {
                 #[cfg(debug_assertions)]
                 eprintln!("          No allies need healing, skipping heal action for {}", c.creature.name);
                 return targets; // Return empty targets if no healing needed
             }

             for i in 0..count {
                 #[cfg(debug_assertions)]
                 eprintln!("          Heal {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                 let self_idx = allies.iter().position(|a| a.id == c.id).unwrap_or(0);
                 if let Some(idx) = select_injured_ally_target(a.target.clone(), allies, self_idx, &targets, None) {
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
        Action::Template(a) => {
            // For templates, we need to determine the target based on the template type
            // Since templates should be resolved to their final form, for now we'll treat them as buff actions
            // targeting allies (like Hunter's Mark which targets enemies)
            // TODO: Implement proper template resolution based on templateOptions.templateName

            // For now, assume template targets enemies (like Hunter's Mark)
            for i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!("          Template {}/{} of {}. Attempting to select target.", i + 1, count, c.creature.name);
                if let Some(idx) = select_enemy_target(crate::enums::EnemyTarget::EnemyWithLeastHP, enemies, &targets, Some(&a.base().id)) {
                    #[cfg(debug_assertions)]
                    eprintln!("            Target selected for {}: Enemy {}", c.creature.name, enemies[idx].creature.name);
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("            No target found for {}'s template {}.", c.creature.name, i + 1);
                }
            }
        },

    }
    #[cfg(debug_assertions)]
    eprintln!("        {} found {} total targets for action {}.", c.creature.name, targets.len(), action.base().name);
    
    targets
}

pub fn select_enemy_target(strategy: EnemyTarget, enemies: &[Combattant], excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting enemy target (Strategy: {:?}). Enemies available: {}. Excluded: {:?}", strategy, enemies.len(), excluded);
    
    // Collect valid candidates
    let mut candidates: Vec<usize> = Vec::new();
    
    for (i, e) in enemies.iter().enumerate() {
        // Check exclusion (true = enemy)
        if excluded.contains(&(true, i)) { continue; }

        // Check buff
        if let Some(bid) = buff_check {
            if e.final_state.buffs.contains_key(bid) { continue; }
        }

        if e.final_state.current_hp <= 0.0 { continue; }
        
        candidates.push(i);
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort candidates based on strategy and tie-breakers
    candidates.sort_by(|&idx1, &idx2| {
        let e1 = &enemies[idx1];
        let e2 = &enemies[idx2];
        
        // 1. Primary Strategy Comparison
        let v1 = match strategy {
            EnemyTarget::EnemyWithLeastHP => e1.final_state.current_hp,
            EnemyTarget::EnemyWithMostHP => -e1.final_state.current_hp,
            EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e1),
            EnemyTarget::EnemyWithLowestAC => e1.creature.ac,
            EnemyTarget::EnemyWithHighestAC => -e1.creature.ac,
        };
        
        let v2 = match strategy {
            EnemyTarget::EnemyWithLeastHP => e2.final_state.current_hp,
            EnemyTarget::EnemyWithMostHP => -e2.final_state.current_hp,
            EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e2),
            EnemyTarget::EnemyWithLowestAC => e2.creature.ac,
            EnemyTarget::EnemyWithHighestAC => -e2.creature.ac,
        };

        // Using partial_cmp for floats. We want strict ordering.
        // For "Least" strategies, smaller is better. 
        // Our mapping above handles "Most" by negating, so we always want "smaller" value to be first.
        // e.g. Most HP: -100 < -50. -100 comes first (higher HP).
        // e.g. Least HP: 10 < 20. 10 comes first (lower HP).
        match v1.partial_cmp(&v2).unwrap_or(Ordering::Equal) {
            Ordering::Equal => {}, // Proceed to tie-breakers
            ord => return ord,
        }

        // 2. Tie-Breaker: Concentration (Target Concentrating > Not Concentrating)
        let c1 = e1.final_state.concentrating_on.is_some();
        let c2 = e2.final_state.concentrating_on.is_some();
        if c1 != c2 {
            // We want c1=true to be "smaller" (come first)
            return if c1 { Ordering::Less } else { Ordering::Greater };
        }

        // 3. Tie-Breaker: Initiative (Higher > Lower)
        // Higher initiative comes first.
        if e1.initiative != e2.initiative {
            return e2.initiative.partial_cmp(&e1.initiative).unwrap_or(Ordering::Equal);
        }

        // 4. Tie-Breaker: AC (Lower > Higher) "Easier to hit"
        // Even if Primary Strategy was AC, this still applies for other strategies.
        if e1.creature.ac != e2.creature.ac {
             return e1.creature.ac.partial_cmp(&e2.creature.ac).unwrap_or(Ordering::Equal);
        }

        // 5. Tie-Breaker: Name (Alphabetical)
        // Ensure deterministic sorting
        e1.creature.name.cmp(&e2.creature.name)
    });

    let best_target = candidates.first().copied();

    #[cfg(debug_assertions)]
    eprintln!("            Selected target: {:?}", best_target.map(|idx| enemies[idx].creature.name.clone()));
    
    best_target
}

pub fn select_ally_target(strategy: AllyTarget, allies: &[Combattant], self_idx: usize, excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
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

fn select_injured_ally_target(strategy: AllyTarget, allies: &[Combattant], _self_idx: usize, excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting injured ally target (Strategy: {:?}). Allies available: {}. Excluded: {:?}", strategy, allies.len(), excluded);
    let mut best_target = None;
    let mut best_val = f64::MAX;

    // Only consider injured allies (current HP < max HP)
    let injured_allies: Vec<(usize, &Combattant)> = allies.iter().enumerate().filter(|(i, a)| {
        // Check exclusion (false = ally)
        if excluded.contains(&(false, *i)) {
            return false;
        }

        // Check buff
        if let Some(bid) = buff_check {
            if a.final_state.buffs.contains_key(bid) {
                return false;
            }
        }

        // Only include injured allies
        a.final_state.current_hp < a.creature.hp
    }).collect();

    if injured_allies.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("            No injured allies found for healing.");
        return None;
    }

    for (i, a) in injured_allies {
        #[cfg(debug_assertions)]
        eprintln!("              Considering injured ally {}. HP: {:.1}/{:.1}", a.creature.name, a.final_state.current_hp, a.creature.hp);

        let val = match strategy {
            AllyTarget::AllyWithLeastHP => a.final_state.current_hp,
            AllyTarget::AllyWithMostHP => -a.final_state.current_hp,
            AllyTarget::AllyWithHighestDPR => -estimate_dpr(a),
            AllyTarget::AllyWithLowestAC => a.creature.ac,
            AllyTarget::AllyWithHighestAC => -a.creature.ac,
            AllyTarget::Self_ => f64::MAX, // Should not happen for healing
        };

        if val < best_val {
            best_val = val;
            best_target = Some(i);
        }
    }
    #[cfg(debug_assertions)]
    eprintln!("            Selected injured ally target: {:?}", best_target.map(|idx| allies[idx].creature.name.clone()));

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
