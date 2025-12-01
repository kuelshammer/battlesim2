use crate::model::*;
use crate::enums::*;
use crate::dice;

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

    }
    #[cfg(debug_assertions)]
    eprintln!("        {} found {} total targets for action {}.", c.creature.name, targets.len(), action.base().name);
    
    targets
}

pub fn select_enemy_target(strategy: EnemyTarget, enemies: &[Combattant], excluded: &[(bool, usize)], buff_check: Option<&str>) -> Option<usize> {
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
