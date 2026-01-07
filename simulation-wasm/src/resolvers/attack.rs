use crate::context::{TurnContext, EffectType};
use crate::events::Event;
use crate::model::{Action, AtkAction};
use crate::{dice, targeting, action_resolver::ActionResolver, rng};
use crate::action_resolver::AttackRollResult;
use crate::events::RollResult;

pub fn resolve(
    resolver: &ActionResolver,
    attack: &AtkAction,
    context: &mut TurnContext,
    actor_id: &str,
) -> Vec<Event> {
    let mut events = Vec::new();

    // 1. Get actor
    let actor = match context.get_combatant(actor_id) {
        Some(c) => c.base_combatant.clone(),
        None => return events,
    };

    let actor_side = context.get_combatant(actor_id).unwrap().side;
    
    // 2. Resolve hits
    let count = attack.targets.max(1) as usize;
    
    for _ in 0..count {
        // Check if action was interrupted by a reaction
        if context.action_interrupted {
            break;
        }

        // Refresh alive enemies for every hit to support dynamic strategies (Most HP, Highest Survivability)
        let all_alive = context.get_alive_combatants();
        let enemies: Vec<_> = all_alive.into_iter()
            .filter(|c| c.side != actor_side)
            .map(|c| c.base_combatant.clone())
            .collect();

        let strategy = attack.target.clone();
        
        if let Some(idx) = targeting::select_enemy_target(&actor, strategy, &enemies, &[], None) {
            let target_id = enemies[idx].id.clone();
            resolve_single_attack_hit(resolver, attack, context, actor_id, &target_id, &mut events);
        } else {
            break; // No targets available
        }
    }

    events
}

pub fn resolve_single_attack_hit(
    resolver: &ActionResolver,
    attack: &AtkAction,
    context: &mut TurnContext,
    actor_id: &str,
    target_id: &str,
    events: &mut Vec<Event>,
) {
    let mut current_target_id = target_id.to_string();

    // 1. Trigger "OnBeingAttacked" reactions
    let pre_roll_triggers = resolver.trigger_reactions_internal(
        &current_target_id,
        crate::enums::TriggerCondition::OnBeingAttacked,
        context,
        Some(actor_id),
        None,
    );
    events.extend(pre_roll_triggers);

    // Check for RedirectAttack among the events
    for event in events.iter() {
        if let Event::Custom { event_type, data, .. } = event {
            if event_type == "RedirectAttack" {
                if let Some(new_target) = data.get("new_target_id") {
                    current_target_id = new_target.clone();
                }
            }
        }
    }

    // 2. Perform attack roll
    let mut attack_result = roll_attack(attack, context, actor_id, &current_target_id);
    let mut target_ac = get_target_ac(&current_target_id, context);

    // 3. Check for hit
    let mut is_hit = !attack_result.is_miss
        && (attack_result.is_critical || attack_result.total >= target_ac);

    // 4. Defensive Reactions (Shield)
    if is_hit && !attack_result.is_critical {
        let (new_ac, reaction_events) = resolve_defensive_reactions(resolver, &current_target_id, context, attack_result.total, target_ac);
        if !reaction_events.is_empty() {
            events.extend(reaction_events);
            target_ac = new_ac;
            if attack_result.total < target_ac {
                is_hit = false;
            }
        }
    }

    // 5. Accuracy Reactions
    if !is_hit && !attack_result.is_miss && !attack_result.is_critical {
        let accuracy_triggers = resolver.trigger_reactions_internal(
            actor_id,
            crate::enums::TriggerCondition::OnMiss,
            context,
            Some(&current_target_id),
            None,
        );
        
        if !accuracy_triggers.is_empty() {
            events.extend(accuracy_triggers);
            
            let mods = context.roll_modifications.take_all(actor_id);
            for modif in mods {
                match modif {
                    crate::context::RollModification::AddBonus { amount } => {
                        let formula = crate::model::DiceFormula::Expr(amount);
                        let bonus = dice::evaluate(&formula, 1);
                        attack_result.total += bonus;
                        
                        if let Some(detail) = &mut attack_result.roll_detail {
                            detail.modifiers.push(("Post-Roll Bonus".to_string(), bonus));
                            detail.total += bonus;
                        }
                    }
                    _ => {}
                }
            }
            
            is_hit = attack_result.total >= target_ac;
        }
    }

    if is_hit {
        let (damage, damage_roll) = calculate_damage(attack, attack_result.is_critical, context, actor_id);

        use crate::resources::ActionTag;
        let range = if attack.tags.contains(&ActionTag::Melee) {
            Some(crate::enums::AttackRange::Melee)
        } else if attack.tags.contains(&ActionTag::Ranged) {
            Some(crate::enums::AttackRange::Ranged)
        } else {
            None
        };

        let hit_event = Event::AttackHit {
            attacker_id: actor_id.to_string(),
            target_id: current_target_id.clone(),
            damage,
            attack_roll: attack_result.roll_detail,
            damage_roll,
            target_ac,
            range,
        };
        context.record_event(hit_event.clone());
        events.push(hit_event);

        let trigger_events = resolver.trigger_reactions_internal(
            &current_target_id,
            crate::enums::TriggerCondition::OnBeingHit,
            context,
            Some(actor_id),
            None,
        );
        events.extend(trigger_events);

        // Check for SplitDamage
        let mut actual_damage = damage;
        let mut split_target_id = None;
        let mut split_percent = 0.0;

        for event in events.iter() {
            if let Event::Custom { event_type, data, .. } = event {
                if event_type == "SplitDamage" {
                    if let (Some(target), Some(percent_str)) = (data.get("target_id"), data.get("percent")) {
                        if let Ok(p) = percent_str.parse::<f64>() {
                            split_target_id = Some(target.clone());
                            split_percent = p / 100.0;
                        }
                    }
                }
            }
        }

        if let Some(tid) = split_target_id {
            let split_amount = actual_damage * split_percent;
            actual_damage -= split_amount;
            
            let split_events = context.apply_damage(&tid, split_amount, "Shared", actor_id);
            events.extend(split_events);
        }

        let damage_events = context.apply_damage(&current_target_id, actual_damage, "Physical", actor_id);
        events.extend(damage_events);
    } else {
        let miss_event = Event::AttackMissed {
            attacker_id: actor_id.to_string(),
            target_id: current_target_id,
            attack_roll: attack_result.roll_detail,
            target_ac,
        };
        context.record_event(miss_event.clone());
        events.push(miss_event);
    }
}

pub fn roll_attack(attack: &AtkAction, context: &mut TurnContext, actor_id: &str, target_id: &str) -> AttackRollResult {
    // Take any pending roll modifications
    let mods = context.roll_modifications.take_all(actor_id);

    // 1. Determine Advantage/Disadvantage
    let attacker_has_adv = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithAdvantage)
        || context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksAndIsAttackedWithAdvantage);
    let attacker_has_triple_adv = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithTripleAdvantage);
    let target_grants_adv = context.has_condition(target_id, crate::enums::CreatureCondition::IsAttackedWithAdvantage);
    
    let attacker_has_dis = context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksWithDisadvantage)
        || context.has_condition(actor_id, crate::enums::CreatureCondition::AttacksAndSavesWithDisadvantage);
    let target_grants_dis = context.has_condition(target_id, crate::enums::CreatureCondition::IsAttackedWithDisadvantage);

    // Check modifications for advantage/disadvantage
    let mut mod_adv = false;
    let mut mod_dis = false;
    for modif in &mods {
        if let crate::context::RollModification::SetAdvantage { roll_type, advantage } = modif {
            if roll_type == "attack" {
                if *advantage {
                    mod_adv = true;
                } else {
                    mod_dis = true;
                }
            }
        }
    }

    let final_triple_adv = attacker_has_triple_adv && !(attacker_has_dis || target_grants_dis || mod_dis);
    let final_adv = (attacker_has_adv || target_grants_adv || mod_adv) && !(attacker_has_dis || target_grants_dis || mod_dis) && !final_triple_adv;
    let final_dis = (attacker_has_dis || target_grants_dis || mod_dis) && !(attacker_has_adv || target_grants_adv || attacker_has_triple_adv || mod_adv);

    // 2. Perform Roll
    let mut roll1 = rng::roll_d20();
    
    // Apply rerolls before calculating natural_roll
    for modif in &mods {
        if let crate::context::RollModification::Reroll { roll_type, must_use_second } = modif {
            if roll_type == "attack" {
                let roll2 = rng::roll_d20();
                if *must_use_second {
                    roll1 = roll2;
                } else {
                    roll1 = roll1.max(roll2);
                }
            }
        }
    }

    let natural_roll: u32;
    
    if final_triple_adv {
        let roll2 = rng::roll_d20();
        let roll3 = rng::roll_d20();
        natural_roll = roll1.max(roll2).max(roll3);
    } else if final_adv {
        let roll2 = rng::roll_d20();
        natural_roll = roll1.max(roll2);
    } else if final_dis {
        let roll2 = rng::roll_d20();
        natural_roll = roll1.min(roll2);
    } else {
        natural_roll = roll1;
    }

    let (mut total, roll_detail) = if context.log_enabled {
        let mut detail = dice::evaluate_detailed(&attack.to_hit, 1);
        detail.modifiers.insert(0, ("Natural Roll".to_string(), natural_roll as f64));
        detail.total += natural_roll as f64;
        (detail.total, Some(detail))
    } else {
        (natural_roll as f64 + dice::evaluate(&attack.to_hit, 1), None)
    };

    // Apply bonus modifications
    for modif in &mods {
        if let crate::context::RollModification::AddBonus { amount } = modif {
            let formula = crate::model::DiceFormula::Expr(amount.clone());
            total += dice::evaluate(&formula, 1);
        }
    }

    // Check for accuracy-altering buffs in active effects
    let mut final_roll_detail = roll_detail;
    
    let mut attacker_buffs: Vec<_> = context.active_effects.values()
        .filter(|e| e.target_id == actor_id)
        .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
        .collect();
    attacker_buffs.sort_by(|a, b| a.0.cmp(b.0));

    for (_, buff) in attacker_buffs {
        if let Some(to_hit_formula) = &buff.to_hit {
            if context.log_enabled {
                let buff_roll = dice::evaluate_detailed(to_hit_formula, 1);
                total += buff_roll.total;
                if let Some(detail) = &mut final_roll_detail {
                    detail.modifiers.push((buff.display_name.clone().unwrap_or_else(|| "Buff".to_string()), buff_roll.total));
                    detail.total += buff_roll.total;
                }
            } else {
                total += dice::evaluate(to_hit_formula, 1);
            }
        }
    }

    AttackRollResult {
        total,
        is_critical: natural_roll == 20,
        is_miss: natural_roll == 1,
        roll_detail: final_roll_detail,
    }
}

pub fn get_target_ac(target_id: &str, context: &TurnContext) -> f64 {
    let Some(target) = context.get_combatant(target_id) else {
        return 10.0;
    };

    let base_ac = target.base_combatant.creature.ac as f64;
    
    let mut buff_ac = 0.0;
    for effect in context.active_effects.values() {
        if effect.target_id == target_id {
            if let EffectType::Buff(buff) = &effect.effect_type {
                if let Some(ac_formula) = &buff.ac {
                    buff_ac += dice::average(ac_formula);
                }
            }
        }
    }

    base_ac + buff_ac
}

pub fn calculate_damage(attack: &AtkAction, is_critical: bool, context: &TurnContext, actor_id: &str) -> (f64, Option<RollResult>) {
    let (mut damage, mut damage_roll) = if context.log_enabled {
        let detail = dice::evaluate_detailed(&attack.dpr, if is_critical { 2 } else { 1 });
        (detail.total, Some(detail))
    } else {
        (dice::evaluate(&attack.dpr, if is_critical { 2 } else { 1 }), None)
    };

    let mut attacker_buffs: Vec<_> = context.active_effects.values()
        .filter(|e| e.target_id == actor_id)
        .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
        .collect();
    attacker_buffs.sort_by(|a, b| a.0.cmp(b.0));

    for (_, buff) in attacker_buffs {
        if let Some(damage_formula) = &buff.damage {
            if context.log_enabled {
                let buff_dmg_roll = dice::evaluate_detailed(damage_formula, 1);
                damage += buff_dmg_roll.total;
                if let Some(detail) = &mut damage_roll {
                    detail.modifiers.push((buff.display_name.clone().unwrap_or_else(|| "Damage Buff".to_string()), buff_dmg_roll.total));
                    detail.total += buff_dmg_roll.total;
                }
            } else {
                damage += dice::evaluate(damage_formula, 1);
            }
        }
    }

    if is_critical {
        if let Some(detail) = &mut damage_roll {
            detail.modifiers.push(("Critical".to_string(), 0.0));
        }
    }

    (damage, damage_roll)
}

pub fn resolve_defensive_reactions(
    resolver: &ActionResolver,
    reactor_id: &str,
    context: &mut TurnContext,
    hit_roll: f64,
    current_ac: f64,
) -> (f64, Vec<Event>) {
    let mut events = Vec::new();
    let mut final_ac = current_ac;

    // Get reactor's triggers
    let triggers = match context.get_combatant(reactor_id) {
        Some(c) => c.base_combatant.creature.triggers.clone(),
        None => return (final_ac, events),
    };

    // Check if reactor has a reaction available
    let reaction_slot_id = crate::enums::ActionSlot::Reaction;
    if let Some(reactor) = context.get_combatant(reactor_id) {
        if reactor.base_combatant.final_state.used_actions.contains(&(reaction_slot_id as i32).to_string()) {
            return (final_ac, events);
        }
    }

    for trigger in triggers {
        if trigger.condition == crate::enums::TriggerCondition::OnBeingAttacked
            && trigger.cost == Some(crate::enums::ActionSlot::Reaction as i32)
            && hit_roll >= final_ac
        {
            // Check if it's a Shield-like template
            if let Action::Template(template_action) = &trigger.action {
                let template_name = template_action.template_options.template_name.to_lowercase();
                
                if template_name == "shield" {
                    // Shield adds +5 AC
                    let shield_ac_bonus = 5.0;
                    if hit_roll < final_ac + shield_ac_bonus {
                        // Trigger it!
                        // 1. Consume reaction
                        if let Some(reactor_mut) = context.get_combatant_mut(reactor_id) {
                            reactor_mut.base_combatant.final_state.used_actions.insert((reaction_slot_id as i32).to_string());
                        }

                        // 2. Resolve the action (will apply the buff)
                        // Record action start for the reaction
                        context.record_event(Event::ActionStarted {
                            actor_id: reactor_id.to_string(),
                            action_id: template_action.id.clone(),
                            decision_trace: std::collections::HashMap::new(),
                        });

                        let reaction_events = resolver.resolve_action(&trigger.action, context, reactor_id);
                        events.extend(reaction_events);

                        final_ac += shield_ac_bonus;
                        break; // Only one defensive reaction per attack
                    }
                }
            }
        }
    }

    (final_ac, events)
}

pub fn get_save_bonus(target_id: &str, context: &TurnContext) -> f64 {
    let Some(target) = context.get_combatant(target_id) else {
        return 0.0;
    };

    let mut bonus = target.base_combatant.creature.save_bonus;

    // Add bonuses from active effects (Bless, Bane, etc.)
    // Filter and sort for determinism
    let mut target_buffs: Vec<_> = context.active_effects.values()
        .filter(|e| e.target_id == target_id)
        .filter_map(|e| if let EffectType::Buff(b) = &e.effect_type { Some((&e.id, b)) } else { None })
        .collect();
    target_buffs.sort_by(|a, b| a.0.cmp(b.0));

    for (_, buff) in target_buffs {
        if let Some(save_formula) = &buff.save {
            bonus += dice::average(save_formula);
        }
    }

    bonus
}