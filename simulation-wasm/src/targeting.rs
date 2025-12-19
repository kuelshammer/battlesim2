use crate::dice;
use crate::enums::*;
use crate::model::*;
use crate::combat_stats::CombatantStats;
use std::cmp::Ordering;

pub fn get_targets(
    c: &Combattant,
    action: &Action,
    allies: &[Combattant],
    enemies: &[Combattant],
) -> Vec<(bool, usize)> {
    #[cfg(debug_assertions)]
    eprintln!(
        "        Getting targets for {}'s action: {}. Allies: {}, Enemies: {}",
        c.creature.name,
        action.base().name,
        allies.len(),
        enemies.len()
    );
    let mut targets = Vec::new();
    let count = action.base().targets.max(1) as usize;

    match action {
        Action::Atk(a) => {
            for _i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          Attack {}/{} of {}. Attempting to select target.",
                    _i + 1,
                    count,
                    c.creature.name
                );
                // For attacks, we allow targeting the same enemy multiple times (e.g. Multiattack, Scorching Ray)
                // So we pass an empty excluded list.
                if let Some(idx) = select_enemy_target(c, a.target.clone(), enemies, &[], None) {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            Target selected for {}: Enemy {}",
                        c.creature.name, enemies[idx].creature.name
                    );
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            No target found for {}'s attack {}.",
                        c.creature.name,
                        _i + 1
                    );
                }
            }
        }
        Action::Heal(a) => {
            // First check if any allies actually need healing
            let injured_ally_idx = allies
                .iter()
                .position(|ally| ally.id != c.id && ally.final_state.current_hp < ally.creature.hp);

            if injured_ally_idx.is_none() {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          No allies need healing, skipping heal action for {}",
                    c.creature.name
                );
                return targets; // Return empty targets if no healing needed
            }

            for _i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          Heal {}/{} of {}. Attempting to select target.",
                    _i + 1,
                    count,
                    c.creature.name
                );
                let self_idx = allies.iter().position(|a| a.id == c.id).unwrap_or(0);
                if let Some(idx) =
                    select_injured_ally_target(a.target.clone(), allies, self_idx, &targets, None)
                {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            Target selected for {}: Ally {}",
                        c.creature.name, allies[idx].creature.name
                    );
                    targets.push((false, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            No target found for {}'s heal {}.",
                        c.creature.name,
                        _i + 1
                    );
                }
            }
        }
        Action::Buff(a) => {
            for _i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          Buff {}/{} of {}. Attempting to select target.",
                    _i + 1,
                    count,
                    c.creature.name
                );
                let self_idx = allies.iter().position(|a| a.id == c.id).unwrap_or(0);
                if let Some(idx) = select_ally_target(
                    a.target.clone(),
                    allies,
                    self_idx,
                    &targets,
                    Some(&a.base().id),
                ) {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            Target selected for {}: Ally {}",
                        c.creature.name, allies[idx].creature.name
                    );
                    targets.push((false, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            No target found for {}'s buff {}.",
                        c.creature.name,
                        _i + 1
                    );
                }
            }
        }
        Action::Debuff(a) => {
            for _i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          Debuff {}/{} of {}. Attempting to select target.",
                    _i + 1,
                    count,
                    c.creature.name
                );
                if let Some(idx) =
                    select_enemy_target(c, a.target.clone(), enemies, &targets, Some(&a.base().id))
                {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            Target selected for {}: Enemy {}",
                        c.creature.name, enemies[idx].creature.name
                    );
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            No target found for {}'s debuff {}.",
                        c.creature.name,
                        _i + 1
                    );
                }
            }
        }
        Action::Template(a) => {
            for _i in 0..count {
                #[cfg(debug_assertions)]
                eprintln!(
                    "          Template {}/{} of {}. Attempting to select target.",
                    _i + 1,
                    count,
                    c.creature.name
                );
                if let Some(idx) = select_enemy_target(
                    c,
                    crate::enums::EnemyTarget::EnemyWithLeastHP,
                    enemies,
                    &targets,
                    Some(&a.base().id),
                ) {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            Target selected for {}: Enemy {}",
                        c.creature.name, enemies[idx].creature.name
                    );
                    targets.push((true, idx));
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "            No target found for {}'s template {}.",
                        c.creature.name,
                        _i + 1
                    );
                }
            }
        }
    }
    #[cfg(debug_assertions)]
    eprintln!(
        "        {} found {} total targets for action {}.",
        c.creature.name,
        targets.len(),
        action.base().name
    );

    targets
}

pub fn select_enemy_target(
    attacker: &Combattant,
    strategy: EnemyTarget,
    enemies: &[Combattant],
    excluded: &[(bool, usize)],
    buff_check: Option<&str>,
) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting enemy target (Strategy: {:?}). Enemies available: {}. Excluded: {:?}", strategy, enemies.len(), excluded);

    // Collect valid candidates
    let mut candidates: Vec<usize> = Vec::new();

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

        if e.final_state.current_hp == 0 {
            continue;
        }

        candidates.push(i);
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort candidates based on strategy and tie-breakers
    candidates.sort_by(|&idx1, &idx2| {
        let e1 = &enemies[idx1];
        let e2 = &enemies[idx2];

        let get_estimated_ac = |target: &Combattant| -> f64 {
            if let Some(k) = attacker.final_state.known_ac.get(&target.id) {
                (k.min + k.max) as f64 / 2.0
            } else {
                15.0 // Default assumption
            }
        };

        let est_ac1 = get_estimated_ac(e1);
        let est_ac2 = get_estimated_ac(e2);

        // 1. Primary Strategy Comparison
        let v1 = match strategy {
            EnemyTarget::EnemyWithLeastHP => e1.final_state.current_hp,
            EnemyTarget::EnemyWithMostHP => -e1.final_state.current_hp,
            EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e1),
            EnemyTarget::EnemyWithLowestAC => est_ac1,
            EnemyTarget::EnemyWithHighestAC => -est_ac1,
        };

        let v2 = match strategy {
            EnemyTarget::EnemyWithLeastHP => e2.final_state.current_hp,
            EnemyTarget::EnemyWithMostHP => -e2.final_state.current_hp,
            EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e2),
            EnemyTarget::EnemyWithLowestAC => est_ac2,
            EnemyTarget::EnemyWithHighestAC => -est_ac2,
        };

        // Using partial_cmp for floats. We want strict ordering.
        match v1.partial_cmp(&v2).unwrap_or(Ordering::Equal) {
            Ordering::Equal => {} // Proceed to tie-breakers
            ord => return ord,
        }

        // 2. Tie-Breaker: Concentration (Target Concentrating > Not Concentrating)
        // Prefer breaking concentration!
        let c1 = e1.final_state.concentrating_on.is_some();
        let c2 = e2.final_state.concentrating_on.is_some();
        if c1 && !c2 {
            return Ordering::Less;
        } // c1 comes first
        if !c1 && c2 {
            return Ordering::Greater;
        }

        // 3. Tie-Breaker: AC (Lower Estimated AC > Higher) "Easier to hit"
        // Use KNOWLEDGE here!
        if est_ac1 != est_ac2 {
            return est_ac1.partial_cmp(&est_ac2).unwrap_or(Ordering::Equal);
        }

        // 4. Tie-Breaker: HP (Lower HP > Higher) "Execute weak targets"
        if e1.final_state.current_hp != e2.final_state.current_hp {
            return e1
                .final_state
                .current_hp
                .partial_cmp(&e2.final_state.current_hp)
                .unwrap_or(Ordering::Equal);
        }

        // 5. Tie-Breaker: Name (Alphabetical) - Deterministic fallback
        e1.creature.name.cmp(&e2.creature.name)
    });

    if let Some(first_idx) = candidates.first() {
        let best = &enemies[*first_idx];
        println!(
            "DEBUG: Strategy '{:?}' selected {} (Index {})",
            strategy, best.creature.name, first_idx
        );
        for idx in &candidates {
            let e = &enemies[*idx];
            let val = match strategy {
                EnemyTarget::EnemyWithLeastHP => e.final_state.current_hp,
                EnemyTarget::EnemyWithMostHP => -e.final_state.current_hp,
                EnemyTarget::EnemyWithHighestDPR => -estimate_dpr(e),
                EnemyTarget::EnemyWithLowestAC => e.creature.ac,
                EnemyTarget::EnemyWithHighestAC => -e.creature.ac,
            };
            println!("  - Candidate {}: Score {:.1}", e.creature.name, val);
        }
    }

    let best_target = candidates.first().copied();

    #[cfg(debug_assertions)]
    eprintln!(
        "            Selected target: {:?}",
        best_target.map(|idx| enemies[idx].creature.name.clone())
    );

    best_target
}

/// Optimized enemy target selection using cached combat statistics
/// Provides O(n) complexity instead of O(n²) for large battles
pub fn select_enemy_target_cached(
    attacker: &Combattant,
    strategy: EnemyTarget,
    enemies: &[Combattant],
    excluded: &[(bool, usize)],
    buff_check: Option<&str>,
    combat_stats_cache: &mut crate::combat_stats::CombatStatsCache,
) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting enemy target (CACHED - Strategy: {:?}). Enemies available: {}. Excluded: {:?}", strategy, enemies.len(), excluded);

    // Collect valid candidates first, then get cached stats in separate step
    let valid_indices: Vec<usize> = enemies.iter().enumerate()
        .filter_map(|(i, e)| {
            // Check exclusion (true = enemy)
            if excluded.contains(&(true, i)) {
                return None;
            }

            // Check buff
            if let Some(bid) = buff_check {
                if e.final_state.buffs.contains_key(bid) {
                    return None;
                }
            }

            if e.final_state.current_hp == 0 {
                return None;
            }

            Some(i)
        })
        .collect();

    if valid_indices.is_empty() {
        return None;
    }

    // Now get cached stats for valid candidates
    let mut candidates: Vec<(usize, CombatantStats)> = Vec::new();
    for &i in &valid_indices {
        let e = &enemies[i];
        let stats = combat_stats_cache.get_stats(e).clone();
        candidates.push((i, stats));
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort candidates based on strategy and tie-breakers using cached stats
    candidates.sort_by(|(idx1, stats1), (idx2, stats2)| {
        let e1 = &enemies[*idx1];
        let e2 = &enemies[*idx2];

        let get_estimated_ac = |target: &Combattant| -> f64 {
            if let Some(k) = attacker.final_state.known_ac.get(&target.id) {
                (k.min + k.max) as f64 / 2.0
            } else {
                15.0 // Default assumption
            }
        };

        let est_ac1 = get_estimated_ac(e1);
        let est_ac2 = get_estimated_ac(e2);

        // 1. Primary Strategy Comparison using cached stats
        let v1 = calculate_target_score_cached(&strategy, stats1, e1.final_state.current_hp, e1.final_state.concentrating_on.is_some(), est_ac1);
        let v2 = calculate_target_score_cached(&strategy, stats2, e2.final_state.current_hp, e2.final_state.concentrating_on.is_some(), est_ac2);

        // Using partial_cmp for floats. We want strict ordering.
        match v1.partial_cmp(&v2).unwrap_or(Ordering::Equal) {
            Ordering::Equal => {} // Proceed to tie-breakers
            ord => return ord,
        }

        // 2. Tie-Breaker: Concentration (Target Concentrating > Not Concentrating)
        // Prefer breaking concentration!
        let c1 = e1.final_state.concentrating_on.is_some();
        let c2 = e2.final_state.concentrating_on.is_some();
        if c1 && !c2 {
            return Ordering::Less;
        } // c1 comes first
        if !c1 && c2 {
            return Ordering::Greater;
        }

        // 3. Tie-Breaker: AC (Lower Estimated AC > Higher) "Easier to hit"
        // Use KNOWLEDGE here!
        if est_ac1 != est_ac2 {
            return est_ac1.partial_cmp(&est_ac2).unwrap_or(Ordering::Equal);
        }

        // 4. Tie-Breaker: HP (Lower HP > Higher) "Execute weak targets"
        if e1.final_state.current_hp != e2.final_state.current_hp {
            return e1
                .final_state
                .current_hp
                .partial_cmp(&e2.final_state.current_hp)
                .unwrap_or(Ordering::Equal);
        }

        // 5. Tie-Breaker: DPR (Higher DPR > Lower) using cached stats
        if stats1.total_dpr != stats2.total_dpr {
            return stats2.total_dpr.partial_cmp(&stats1.total_dpr).unwrap_or(Ordering::Equal);
        }

        // 6. Tie-Breaker: Name (Alphabetical) - Deterministic fallback
        e1.creature.name.cmp(&e2.creature.name)
    });

    if let Some((first_idx, _)) = candidates.first() {
        let best = &enemies[*first_idx];
        println!(
            "DEBUG CACHED: Strategy '{:?}' selected {} (Index {})",
            strategy, best.creature.name, first_idx
        );
        for (idx, stats) in &candidates {
            let e = &enemies[*idx];
            let val = match strategy {
                EnemyTarget::EnemyWithLeastHP => e.final_state.current_hp,
                EnemyTarget::EnemyWithMostHP => -e.final_state.current_hp,
                EnemyTarget::EnemyWithHighestDPR => -stats.total_dpr,
                EnemyTarget::EnemyWithLowestAC => e.creature.ac,
                EnemyTarget::EnemyWithHighestAC => -e.creature.ac,
            };
            println!("  - Candidate {}: Score {:.1} (DPR: {:.1})", e.creature.name, val, stats.total_dpr);
        }
    }

    let best_target = candidates.first().map(|(idx, _)| *idx);

    #[cfg(debug_assertions)]
    eprintln!(
        "            Selected cached target: {:?}",
        best_target.map(|idx| enemies[idx].creature.name.clone())
    );

    best_target
}

pub fn select_ally_target(
    strategy: AllyTarget,
    allies: &[Combattant],
    self_idx: usize,
    excluded: &[(bool, usize)],
    buff_check: Option<&str>,
) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!(
        "            Selecting ally target (Strategy: {:?}). Allies available: {}. Excluded: {:?}",
        strategy,
        allies.len(),
        excluded
    );
    let mut best_target = None;
    let mut best_val = f64::MAX;

    // For single-target heals/buffs in multi-target actions, we allow re-targeting the same ally.
    // The previous implementation was designed more for abilities that must hit distinct targets.
    // This removes the `excluded_indices` check.

    if strategy == AllyTarget::Self_ {
        // Only exclude if the self target is explicitly dead (which shouldn't happen for self-buffs)
        if allies[self_idx].final_state.current_hp == 0 {
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
        eprintln!(
            "              Considering ally {}. HP: {:.1}",
            a.creature.name, a.final_state.current_hp
        );
        if a.final_state.current_hp == 0 {
            #[cfg(debug_assertions)]
            eprintln!(
                "                Ally {} is dead, skipping.",
                a.creature.name
            );
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
    eprintln!(
        "            Selected target: {:?}",
        best_target.map(|idx| allies[idx].creature.name.clone())
    );

    best_target
}

fn select_injured_ally_target(
    strategy: AllyTarget,
    allies: &[Combattant],
    _self_idx: usize,
    excluded: &[(bool, usize)],
    buff_check: Option<&str>,
) -> Option<usize> {
    #[cfg(debug_assertions)]
    eprintln!("            Selecting injured ally target (Strategy: {:?}). Allies available: {}. Excluded: {:?}", strategy, allies.len(), excluded);
    let mut best_target = None;
    let mut best_val = f64::MAX;

    // Only consider injured allies (current HP < max HP)
    let injured_allies: Vec<(usize, &Combattant)> = allies
        .iter()
        .enumerate()
        .filter(|(i, a)| {
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
        })
        .collect();

    if injured_allies.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("            No injured allies found for healing.");
        return None;
    }

    for (i, a) in injured_allies {
        #[cfg(debug_assertions)]
        eprintln!(
            "              Considering injured ally {}. HP: {:.1}/{:.1}",
            a.creature.name, a.final_state.current_hp, a.creature.hp
        );

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
    eprintln!(
        "            Selected injured ally target: {:?}",
        best_target.map(|idx| allies[idx].creature.name.clone())
    );

    best_target
}

/// Calculate target score using cached combat statistics for O(1) performance
fn calculate_target_score_cached(
    strategy: &EnemyTarget,
    target_stats: &CombatantStats,
    target_current_hp: f64,
    _target_concentrating: bool,
    attacker_estimated_ac: f64,
) -> f64 {
    match strategy {
        EnemyTarget::EnemyWithLeastHP => target_current_hp,
        EnemyTarget::EnemyWithMostHP => -target_current_hp,
        EnemyTarget::EnemyWithHighestDPR => -target_stats.total_dpr,
        EnemyTarget::EnemyWithLowestAC => attacker_estimated_ac,
        EnemyTarget::EnemyWithHighestAC => -attacker_estimated_ac,
    }
}

/// Legacy DPR estimation function - kept for compatibility
fn estimate_dpr(c: &Combattant) -> f64 {
    const BASELINE_AC: f64 = 15.0;

    // Separate actions by action type for proper action economy
    let mut action_dpr: f64 = 0.0;
    let mut bonus_action_dpr: f64 = 0.0;

    for action in &c.creature.actions {
        if let Action::Atk(a) = action {
            // Calculate base damage per hit
            let damage_per_hit = match &a.dpr {
                DiceFormula::Value(v) => *v,
                DiceFormula::Expr(e) => dice::parse_average(e),
            };

            // Calculate to_hit bonus
            let to_hit_bonus = match &a.to_hit {
                DiceFormula::Value(v) => *v,
                DiceFormula::Expr(e) => dice::parse_average(e),
            };

            // Calculate hit probability (d20 + bonus >= AC)
            // Need: d20 roll >= (AC - bonus)
            // Hit on: 21 - (AC - bonus) or higher on d20
            let needed_roll = BASELINE_AC - to_hit_bonus;
            let hit_chance = if needed_roll <= 1.0 {
                0.95 // Auto-hit except on nat 1
            } else if needed_roll >= 20.0 {
                0.05 // Only nat 20
            } else {
                (21.0 - needed_roll) / 20.0
            };

            // Account for number of targets
            let num_targets = a.targets.max(1) as f64;

            // Calculate expected DPR for this action
            let expected_dpr = damage_per_hit * hit_chance * num_targets;

            eprintln!("DEBUG: Action {} - Damage: {:.1}, ToHit: +{:.0}, HitChance: {:.0}%, Targets: {}, DPR: {:.1}", 
                a.name, damage_per_hit, to_hit_bonus, hit_chance * 100.0, num_targets, expected_dpr);

            // Categorize by action cost (simplified - assumes legacy action_slot)
            // 0 = Action, 1 = Bonus Action, 2+ = Other
            let is_bonus_action = a.action_slot == Some(1);

            if is_bonus_action {
                bonus_action_dpr = bonus_action_dpr.max(expected_dpr);
            } else {
                action_dpr = action_dpr.max(expected_dpr);
            }
        }
    }

    // Total DPR = best Action + best Bonus Action
    let total_dpr = action_dpr + bonus_action_dpr;
    eprintln!(
        "DEBUG: Creature {} - Action DPR: {:.1}, Bonus DPR: {:.1}, Total: {:.1}",
        c.creature.name, action_dpr, bonus_action_dpr, total_dpr
    );

    total_dpr
}

#[cfg(test)]
#[path = "./targeting_test.rs"]
mod targeting_test;
