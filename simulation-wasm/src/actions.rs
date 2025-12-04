use crate::model::*;
use crate::enums::*;
// use crate::dice;
use rand::Rng;
use std::collections::{HashMap, HashSet};

pub fn check_action_condition(
    action: &Action,
    actor: &Combattant,
    allies: &[Combattant],
    enemies: &[Combattant]
) -> bool {
    match &action.base().condition {
        ActionCondition::Default => true,
        ActionCondition::AllyAt0HP => {
            allies.iter().any(|c| c.final_state.current_hp <= 0.0)
        },
        ActionCondition::AllyUnderHalfHP => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp / 2.0)
        },
        ActionCondition::AllyBelow25PercentHP => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp * 0.25)
        },
        ActionCondition::AllyBelow50PercentHP => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp * 0.5)
        },
        ActionCondition::AllyBelow75PercentHP => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp * 0.75)
        },
        ActionCondition::AnyAllyInjured => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp)
        },
        ActionCondition::AnyAllyNeedsHealing => {
            // More specific - check if any ally is significantly injured (more than 50% HP lost)
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp * 0.5)
        },
        ActionCondition::AnyAllyBelow50PercentHP => {
            allies.iter().any(|c| c.final_state.current_hp < c.creature.hp * 0.5)
        },
        ActionCondition::IsAvailable => true, // Always available
        ActionCondition::IsUnderHalfHP => {
            actor.final_state.current_hp < actor.creature.hp / 2.0
        },
        ActionCondition::HasNoTHP => {
            actor.final_state.temp_hp.is_none() || actor.final_state.temp_hp == Some(0.0)
        },
        ActionCondition::NotUsedYet => {
            // Check if this specific action has been used in the current encounter
            !actor.final_state.actions_used_this_encounter.contains(&action.base().id)
        },
        ActionCondition::EnemyCountOne => {
            enemies.iter().filter(|c| c.final_state.current_hp > 0.0).count() == 1
        },
        ActionCondition::EnemyCountMultiple => {
            enemies.iter().filter(|c| c.final_state.current_hp > 0.0).count() > 1
        },
        ActionCondition::BuffNotActive => {
            // Check if the buff provided by this action is already active on the actor
            // This is primarily for Self-targeted buffs like Rage
            if let Action::Buff(b) = action {
                !actor.final_state.buffs.contains_key(&b.base().id)
            } else {
                true
            }
        },
        ActionCondition::NotConcentrating => {
            actor.final_state.concentrating_on.is_none()
        },
    }
}

pub fn get_actions(c: &Combattant, allies: &[Combattant], enemies: &[Combattant]) -> Vec<Action> {
    #[cfg(debug_assertions)]
    eprintln!("      Getting actions for {}. Creature actions: {}", c.creature.name, c.creature.actions.len());
    let mut result = Vec::new();
    let mut used_slots = HashSet::new();

    for action in &c.creature.actions {
        #[cfg(debug_assertions)]
        eprintln!("        Considering action: {} (Slot: {:?}, Freq: {:?})", action.base().name, action.base().action_slot, action.base().freq);

        // For D&D 5e action economy: Only check slot conflicts for exact same slots
        // This allows Action (0) and Bonus Action (1) together, as well as Other actions (5,6) with Action/Bonus
        if let Some(slot) = action.base().action_slot {
            if used_slots.contains(&slot) {
                #[cfg(debug_assertions)]
                eprintln!("          Slot {} already used this turn.", slot);
                continue;
            }
            used_slots.insert(slot);
        }
        if !is_usable(c, action) {
            #[cfg(debug_assertions)]
            eprintln!("          Action {} not usable.", action.base().name);
            continue;
        }
        
        // Check action condition
        if !check_action_condition(action, c, allies, enemies) {
            #[cfg(debug_assertions)]
            eprintln!("          Action {} condition not met.", action.base().name);
            continue;
        }
        
        #[cfg(debug_assertions)]
        eprintln!("          Action {} usable. Adding to result.", action.base().name);
        result.push(action.clone());
    }
    
    result
}

pub fn is_usable(c: &Combattant, action: &Action) -> bool {
    #[cfg(debug_assertions)]
    eprintln!("        Checking usability for {}: {}. Remaining uses: {:?}", c.creature.name, action.base().name, c.final_state.resources.current.get(&action.base().id));
    match &action.base().freq {
        Frequency::Static(s) if s == "at will" => true,
        _ => {
            let uses = *c.final_state.resources.current.get(&action.base().id).unwrap_or(&0.0);
            uses >= 1.0
        }
    }
}

// Helper to determine if a combatant has a specific condition
pub fn has_condition(c: &Combattant, condition: CreatureCondition) -> bool {
    c.final_state.buffs.iter()
        .any(|(_, buff)| buff.condition == Some(condition))
}

// Helper to get effective attack roll considering advantage/disadvantage
pub fn get_attack_roll_result(attacker: &Combattant) -> (f64, bool, bool) {
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

pub fn break_concentration(caster_id: &str, buff_id: &str, allies: &mut [Combattant], enemies: &mut [Combattant]) {
    #[cfg(debug_assertions)]
    eprintln!("        [DEBUG] break_concentration called: Caster ID: {}, Buff ID: {}", caster_id, buff_id);

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

/// Removes ALL buffs from a dead source, not just the one being concentrated on.
/// Use this when a caster dies (HP <= 0) to ensure all their spells end.
pub fn remove_all_buffs_from_source(source_id: &str, allies: &mut [Combattant], enemies: &mut [Combattant]) {
    #[cfg(debug_assertions)]
    eprintln!("        [DEBUG] remove_all_buffs_from_source called: Source ID: {}", source_id);

    // Clear concentration on the dead caster AND clear their action targets
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        if c.id == source_id {
            c.final_state.concentrating_on = None;
            
            // Clear targets from all actions this caster has taken
            // This ensures the frontend doesn't show "Bless on X" for a dead caster
            for action in c.actions.iter_mut() {
                action.targets.clear();
            }
            
            #[cfg(debug_assertions)]
            eprintln!("          Cleared concentration and action targets on dead caster: {}", c.creature.name);
        }
    }

    // Remove ALL buffs from this source across all combatants
    for c in allies.iter_mut().chain(enemies.iter_mut()) {
        let _before_count = c.final_state.buffs.len();
        c.final_state.buffs.retain(|_buff_id, buff| {
            let should_keep = buff.source.as_ref() != Some(&source_id.to_string());
            if !should_keep {
                #[cfg(debug_assertions)]
                eprintln!("          Removed buff '{}' from {} (source {} is dead)", _buff_id, c.creature.name, source_id);
            }
            should_keep
        });
        let _after_count = c.final_state.buffs.len();
        
        #[cfg(debug_assertions)]
        if _before_count != _after_count {
            eprintln!("          {} had {} buffs from dead source, now has {} buffs total", c.creature.name, _before_count - _after_count, _after_count);
        }
    }
}

pub fn get_remaining_uses(creature: &Creature, rest: &str, old_value: Option<&HashMap<String, f64>>) -> HashMap<String, f64> {
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
