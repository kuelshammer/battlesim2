use crate::actions::get_attack_roll_result;
use crate::dice;
use crate::enums::*;
use crate::model::*;
use crate::rng;
use rand::Rng; // Import Rng trait for gen_range
use std::collections::HashMap;

// Helper to update encounter stats
pub fn update_stats(
    stats: &mut HashMap<String, EncounterStats>,
    attacker_id: &str,
    target_id: &str,
    damage: f64,
    heal: f64,
) {
    let attacker_stats = stats
        .entry(attacker_id.to_string())
        .or_insert(EncounterStats {
            damage_dealt: 0.0,
            damage_taken: 0.0,
            heal_given: 0.0,
            heal_received: 0.0,
            characters_buffed: 0.0,
            buffs_received: 0.0,
            characters_debuffed: 0.0,
            debuffs_received: 0.0,
            times_unconscious: 0.0,
        });
    attacker_stats.damage_dealt += damage;
    attacker_stats.heal_given += heal;

    let target_stats = stats
        .entry(target_id.to_string())
        .or_insert(EncounterStats {
            damage_dealt: 0.0,
            damage_taken: 0.0,
            heal_given: 0.0,
            heal_received: 0.0,
            characters_buffed: 0.0,
            buffs_received: 0.0,
            characters_debuffed: 0.0,
            debuffs_received: 0.0,
            times_unconscious: 0.0,
        });
    target_stats.damage_taken += damage;
    target_stats.heal_received += heal;
}

// Helper to update encounter stats for buffs/debuffs
pub fn update_stats_buff(
    stats: &mut HashMap<String, EncounterStats>,
    attacker_id: &str,
    target_id: &str,
    is_buff: bool,
) {
    let attacker_stats = stats
        .entry(attacker_id.to_string())
        .or_insert(EncounterStats {
            damage_dealt: 0.0,
            damage_taken: 0.0,
            heal_given: 0.0,
            heal_received: 0.0,
            characters_buffed: 0.0,
            buffs_received: 0.0,
            characters_debuffed: 0.0,
            debuffs_received: 0.0,
            times_unconscious: 0.0,
        });
    if is_buff {
        attacker_stats.characters_buffed += 1.0;
    } else {
        attacker_stats.characters_debuffed += 1.0;
    }

    let target_stats = stats
        .entry(target_id.to_string())
        .or_insert(EncounterStats {
            damage_dealt: 0.0,
            damage_taken: 0.0,
            heal_given: 0.0,
            heal_received: 0.0,
            characters_buffed: 0.0,
            buffs_received: 0.0,
            characters_debuffed: 0.0,
            debuffs_received: 0.0,
            times_unconscious: 0.0,
        });
    if is_buff {
        target_stats.buffs_received += 1.0;
    } else {
        target_stats.debuffs_received += 1.0;
    }
}

// Function to handle defensive triggers (e.g., Shield spell)
fn process_defensive_triggers(
    _attacker: &Combattant,  // Attacker (immutable)
    target: &mut Combattant, // Target (mutable, might use reaction)
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
    total_hit_roll: f64,
    current_ac: f64,
) -> (f64, bool) {
    let mut final_ac = current_ac;
    let mut reaction_used = false;

    let reaction_slot_id = ActionSlot::Reaction;
    // Convert to i32 string key
    if target
        .final_state
        .used_actions
        .contains(&(reaction_slot_id as i32).to_string())
    {
        return (final_ac, reaction_used);
    }

    let target_triggers = target.creature.triggers.clone();

    for trigger in target_triggers.iter() {
        if trigger.condition == TriggerCondition::OnBeingAttacked
            && trigger.cost == Some(ActionSlot::Reaction as i32)
            && total_hit_roll >= final_ac // Currently a hit
            && matches!(&trigger.action, Action::Buff(_))
        {
            if let Action::Buff(buff_action) = &trigger.action {
                if let Some(ac_buff_dice) = &buff_action.buff.ac {
                    let potential_ac_buff = dice::evaluate(ac_buff_dice, 1);
                    if total_hit_roll < final_ac + potential_ac_buff {
                        // Trigger activates
                        reaction_used = true;
                        target
                            .final_state
                            .used_actions
                            .insert((reaction_slot_id as i32).to_string());

                        let mut buff = buff_action.buff.clone();
                        buff.source = Some(target.id.clone());

                        target
                            .final_state
                            .buffs
                            .insert(buff_action.base().id.clone(), buff);
                        update_stats_buff(stats, &target.id, &target.id, true);

                        final_ac += potential_ac_buff;
                        if log_enabled {
                            log.push(format!(
                                "          {} uses {} to increase AC by {:.0} (New AC: {:.0})",
                                target.creature.name,
                                buff_action.base().name,
                                potential_ac_buff,
                                final_ac
                            ));
                        }
                        break;
                    }
                }
            }
        }
    }
    (final_ac, reaction_used)
}

// Function to handle offensive triggers (e.g., Divine Smite, OnCrit effects)
// Returns (additional_damage, cleanup_instructions)
fn process_offensive_triggers(
    attacker: &Combattant,
    _main_action: &Action,   // The main action that just hit
    target: &mut Combattant, // The target that was hit
    is_crit: bool,
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
) -> (f64, Vec<CleanupInstruction>) {
    let mut additional_damage = 0.0;
    let cleanup_instructions = Vec::new();

    // Iterate over a clone of triggers to avoid borrowing issues while modifying attacker
    let attacker_triggers = attacker.creature.triggers.clone();

    for trigger in attacker_triggers.iter() {
        let trigger_should_fire = trigger.condition == TriggerCondition::OnHit
            || (trigger.condition == TriggerCondition::OnCriticalHit && is_crit);

        if trigger_should_fire {
            // Check if trigger is usable (resource cost)
            // For now, assume "at will" or always usable if it has no explicit cost / resource tracking.
            // If the trigger has a cost that needs to be tracked by `remaining_uses`, it would be decremented here.

            if log_enabled {
                log.push(format!(
                    "          {} triggers {} on hit/crit!",
                    attacker.creature.name,
                    trigger.action.base().name
                ));
            }

            // Execute the trigger action
            match &trigger.action {
                Action::Atk(a) => {
                    // This is extra damage from a rider effect
                    let extra_damage = dice::evaluate(&a.dpr, if is_crit { 2 } else { 1 });
                    additional_damage += extra_damage;
                    if log_enabled {
                        log.push(format!(
                            "             -> Adds {:.0} additional damage ({})",
                            extra_damage,
                            a.base().name
                        ));
                    }
                }
                Action::Buff(a) => {
                    // Apply buff from trigger
                    let mut buff = a.buff.clone();
                    buff.source = Some(attacker.id.clone());
                    target.final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &target.id, true);
                    if log_enabled {
                        log.push(format!(
                            "             -> Applies buff {} to {}",
                            a.base().name,
                            target.creature.name
                        ));
                    }
                }
                Action::Debuff(a) => {
                    // Apply debuff from trigger
                    let dc_val = a.save_dc;
                    let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1);
                    let save_bonus = target.creature.save_bonus;
                    let roll = rng::get_rng().gen_range(1..=20) as f64;

                    if log_enabled {
                        log.push(format!("             -> Debuff {} vs {}: DC {:.0} vs Save {:.0} (Rolled {:.0} + {:.0})", 
                            a.buff.display_name.as_deref().unwrap_or("Unknown"), target.creature.name, dc, roll + save_bonus, roll, save_bonus));
                    }

                    if roll + save_bonus < dc {
                        let mut buff = a.buff.clone();
                        buff.source = Some(attacker.id.clone());
                        target.final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &target.id, false);
                        if log_enabled {
                            log.push("             Failed! Debuff applied.".to_string());
                        }
                    } else if log_enabled {
                        log.push("             Saved!".to_string());
                    }
                }
                _ => {} // Healing triggers on attack? Unlikely.
            }
        }
    }

    (additional_damage, cleanup_instructions)
}

// Core logic to apply a single action to a single target
fn apply_single_effect(
    attacker: &mut Combattant,
    mut target_opt: Option<&mut Combattant>, // Mutable binding to Option to allow re-borrowing
    action: &Action,
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
) -> Vec<CleanupInstruction> {
    let mut cleanup_instructions = Vec::new();

    // Helper to get target name (Read)
    // Re-borrow for name access
    let target_name = if let Some(t) = &target_opt {
        t.creature.name.clone()
    } else {
        attacker.creature.name.clone()
    };
    let _target_id = if let Some(t) = &target_opt {
        t.id.clone()
    } else {
        attacker.id.clone()
    };

    match action {
        Action::Atk(a) => {
            let (roll, is_crit, is_miss) = get_attack_roll_result(attacker);
            let to_hit_bonus = dice::evaluate(&a.to_hit, 1);

            // Check for advantage/disadvantage from ATTACKER's conditions
            let attacker_has_advantage =
                crate::actions::has_condition(attacker, CreatureCondition::AttacksWithAdvantage)
                    || crate::actions::has_condition(
                        attacker,
                        CreatureCondition::AttacksAndIsAttackedWithAdvantage,
                    );
            let attacker_has_disadvantage =
                crate::actions::has_condition(attacker, CreatureCondition::AttacksWithDisadvantage)
                    || crate::actions::has_condition(
                        attacker,
                        CreatureCondition::AttacksAndSavesWithDisadvantage,
                    );

            // Check for advantage/disadvantage from TARGET's conditions (e.g., Dodge, Reckless Attack)
            let (target_grants_disadvantage, target_grants_advantage) = if let Some(t) = &target_opt
            {
                let grants_dis =
                    crate::actions::has_condition(t, CreatureCondition::IsAttackedWithDisadvantage);
                // Check for IsAttackedWithAdvantage OR AttacksAndIsAttackedWithAdvantage (Reckless Attack)
                let grants_adv =
                    crate::actions::has_condition(t, CreatureCondition::IsAttackedWithAdvantage)
                        || crate::actions::has_condition(
                            t,
                            CreatureCondition::AttacksAndIsAttackedWithAdvantage,
                        );
                (grants_dis, grants_adv)
            } else {
                (false, false)
            };

            // D&D 5e Rule: If ANY source of advantage AND ANY source of disadvantage → Normal roll
            let has_any_advantage = attacker_has_advantage || target_grants_advantage;
            let has_any_disadvantage = attacker_has_disadvantage || target_grants_disadvantage;

            // Apply cancellation rule
            let final_advantage = has_any_advantage && !has_any_disadvantage;
            let final_disadvantage = has_any_disadvantage && !has_any_advantage;

            let mut buff_bonus = 0.0;
            let mut buff_details = Vec::new();

            for b in attacker.final_state.buffs.values() {
                if let Some(f) = &b.to_hit {
                    let val = dice::evaluate(f, 1);
                    buff_bonus += val;
                    if let Some(name) = &b.display_name {
                        buff_details.push(format!("{} {:.0}", name, val));
                    } else {
                        buff_details.push(format!("{:.0}", val));
                    }
                }
            }

            let total_hit = roll + to_hit_bonus + buff_bonus;

            // Check for target buffs that affect attack rolls (like Bane)
            let (target_debuffs, _bane_disadvantage) = if let Some(t) = &target_opt {
                let mut debuffs = Vec::new();
                let mut has_bane_disadvantage = false;

                for b in t.final_state.buffs.values() {
                    if let Some(to_hit_penalty) = &b.to_hit {
                        let val = dice::evaluate(to_hit_penalty, 1);
                        if val != 0.0 {
                            let name = b.display_name.as_deref().unwrap_or("Unknown");
                            debuffs.push(format!("{} {:.0}", name, val));
                        }
                    }

                    // Check if this is Bane (special case for disadvantage)
                    if let Some(name) = &b.display_name {
                        if name.contains("Bane") {
                            has_bane_disadvantage = true;
                        }
                    }
                }
                (debuffs, has_bane_disadvantage)
            } else {
                (Vec::new(), false)
            };

            // Check if attacker is affected by Bane
            let _attacker_bane_debuff: Vec<String> = attacker
                .final_state
                .buffs
                .values()
                .filter_map(|b| {
                    if let Some(name) = &b.display_name {
                        if name.contains("Bane") {
                            Some("Bane -".to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // Resolve Target Stats (Read)
            // Re-borrow target_opt immutably
            let (target_ac, target_buff_ac) = if let Some(t) = &target_opt {
                (
                    t.creature.ac,
                    t.final_state
                        .buffs
                        .values()
                        .filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1)))
                        .sum::<f64>(),
                )
            } else {
                (
                    attacker.creature.ac,
                    attacker
                        .final_state
                        .buffs
                        .values()
                        .filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1)))
                        .sum::<f64>(),
                )
            };
            let total_ac_before_trigger = target_ac as f64 + target_buff_ac;

            // Trigger processing requires mutable access to target
            // We need to be careful not to borrow attacker if target IS attacker

            // Re-borrow target_opt mutably
            let (final_ac, reaction_used) = if let Some(t) = &mut target_opt {
                // Distinct target (re-borrowed *t)
                process_defensive_triggers(
                    attacker,
                    t,
                    stats,
                    log,
                    log_enabled,
                    total_hit,
                    total_ac_before_trigger,
                )
            } else {
                // Self-target
                // Skip defensive triggers to avoid mutable/immutable borrow conflict on attacker
                (total_ac_before_trigger, false)
            };

            let hits = !is_miss && (is_crit || total_hit >= final_ac);

            // NEW: precise knowledge update
            if let Some(t) = target_opt.as_ref() {
                let t_id = t.id.clone();
                let knowledge = attacker.final_state.known_ac.entry(t_id).or_default();

                if !is_crit && hits {
                    // Start with high max, lower it as we hit with lower rolls
                    if (total_hit as i32) < knowledge.max {
                        knowledge.max = total_hit as i32;
                    }
                }

                if !is_miss && !hits {
                    // Missed (but not auto-miss), so AC must be higher
                    // min starts at 0, raise it as we miss with higher rolls
                    if (total_hit as i32 + 1) > knowledge.min {
                        knowledge.min = total_hit as i32 + 1;
                    }
                }
            }

            if log_enabled {
                let crit_str = if is_crit {
                    " (CRIT!)"
                } else if is_miss {
                    " (MISS!)"
                } else {
                    ""
                };
                let reaction_str = if reaction_used {
                    " (Reaction Used)"
                } else {
                    ""
                };

                // Advantage/Disadvantage logging with cancellation
                let adv_str = if final_advantage {
                    " (ADVANTAGE)"
                } else if final_disadvantage {
                    " (DISADVANTAGE)"
                } else if has_any_advantage && has_any_disadvantage {
                    " (ADV+DIS=Normal)"
                } else {
                    ""
                };

                // Combine attacker buffs (Bless) and target debuffs (Bane) for roll display
                let roll_modifiers: Vec<String> = buff_details
                    .iter()
                    .filter(|d| d.contains("Bless"))
                    .chain(target_debuffs.iter())
                    .cloned()
                    .collect();

                let roll_mod_str = if !roll_modifiers.is_empty() {
                    format!(" ({})", roll_modifiers.join(", "))
                } else {
                    String::new()
                };

                // Enhanced Bless/Bane logging (for status display)
                let bless_details: Vec<String> = buff_details
                    .iter()
                    .filter(|d| d.contains("Bless"))
                    .cloned()
                    .collect();
                let bane_details: Vec<String> = buff_details
                    .iter()
                    .filter(|d| d.contains("Bane"))
                    .cloned()
                    .collect();

                let effect_str = if !bless_details.is_empty() || !bane_details.is_empty() {
                    let mut effects = Vec::new();
                    if !bless_details.is_empty() {
                        effects.push(format!("Bless: {}", bless_details.join(", ")));
                    }
                    if !bane_details.is_empty() {
                        effects.push(format!("Bane: {}", bane_details.join(", ")));
                    }
                    format!(" (spell effects: {})", effects.join(", "))
                } else {
                    String::new()
                };
                let hit_str = if hits { "✅ **HIT**" } else { "❌ **MISS**" };
                log.push(format!(
                    "* ⚔️ Attack vs **{}**: **{:.0}**{} vs AC {:.0}{}{}{}{} -> {}",
                    target_name,
                    total_hit,
                    roll_mod_str,
                    final_ac,
                    adv_str,
                    reaction_str,
                    crit_str,
                    effect_str,
                    hit_str
                ));
            }

            if hits {
                // Calculate base damage and buff damage first
                let mut damage = dice::evaluate(&a.dpr, if is_crit { 2 } else { 1 });
                let buff_dmg: f64 = attacker
                    .final_state
                    .buffs
                    .values()
                    .filter_map(|b| b.damage.as_ref().map(|f| dice::evaluate(f, 1)))
                    .sum();
                damage += buff_dmg;

                // Offensive trigger hook (e.g., Divine Smite, OnCrit effects)
                // Handle self-targeting vs distinct target cases to avoid borrowing conflicts
                if let Some(target) = target_opt.as_mut() {
                    // Distinct target: can safely pass attacker as immutable reference
                    let (offensive_damage, offensive_cleanup) = process_offensive_triggers(
                        attacker,
                        action,
                        target,
                        is_crit,
                        stats,
                        log,
                        log_enabled,
                    );
                    cleanup_instructions.extend(offensive_cleanup);
                    damage += offensive_damage; // Add damage from offensive triggers
                } else {
                    // Self-target: skip offensive triggers to avoid borrowing conflicts for now
                    // TODO: Implement separate logic for self-targeting offensive triggers
                }

                // Apply Damage Reduction (Flat)
                let mut flat_reduction = 0.0;
                let mut reduction_sources = Vec::new();

                if let Some(t) = target_opt.as_ref() {
                    for b in t.final_state.buffs.values() {
                        if let Some(reduction_formula) = &b.damage_reduction {
                            let val = dice::evaluate(reduction_formula, 1);
                            flat_reduction += val;
                            if let Some(name) = &b.display_name {
                                reduction_sources.push(format!("{} (-{:.0})", name, val));
                            }
                        }
                    }
                }

                let damage_before_reduction = damage;
                damage = (damage - flat_reduction).max(0.0);

                // Apply Damage Multipliers (Resistance/Vulnerability)
                let mut total_multiplier = 1.0;
                let mut multiplier_sources = Vec::new();

                if let Some(t) = target_opt.as_ref() {
                    for b in t.final_state.buffs.values() {
                        if let Some(mult) = b.damage_taken_multiplier {
                            total_multiplier *= mult;
                            let source_name = b.display_name.as_deref().unwrap_or("Resistance");
                            if mult < 1.0 {
                                multiplier_sources.push(source_name.to_string());
                            } else if mult > 1.0 {
                                multiplier_sources.push(format!("{} (Vulnerable)", source_name));
                            }
                        }
                    }
                }

                let _damage_before_multiplier = damage;
                damage = (damage * total_multiplier).floor(); // Round down damage in 5e

                if log_enabled {
                    let mut calc_log = format!("  * 🩸 Damage: **{:.0}**", damage_before_reduction);

                    // Add base damage breakdown if there are buffs
                    if buff_dmg > 0.0 {
                        calc_log.push_str(&format!(
                            " (Base {:.0} + Buffs {:.0})",
                            damage_before_reduction - buff_dmg,
                            buff_dmg
                        ));
                    }

                    // Add damage reduction information
                    if flat_reduction > 0.0 {
                        calc_log.push_str(&format!(
                            " - {:.0} [Damage Reduction: {}]",
                            flat_reduction,
                            reduction_sources.join(", ")
                        ));
                    }

                    // Add multiplier information (resistance/vulnerability)
                    if total_multiplier != 1.0 {
                        calc_log.push_str(&format!(
                            " * {:.2} [{}]",
                            total_multiplier,
                            multiplier_sources.join(", ")
                        ));
                        calc_log.push_str(&format!(" = **{:.0}**", damage));
                    } else if flat_reduction > 0.0 {
                        calc_log.push_str(&format!(" = **{:.0}**", damage));
                    }

                    log.push(calc_log);
                }

                // Apply Damage (Write)
                if let Some(t) = target_opt {
                    let mut remaining_damage = damage;
                    let mut ward_absorbed_amount = 0.0;
                    let mut _thp_absorbed_amount = 0.0;

                    // 1. Arcane Ward Absorption
                    let ward_hp = t.final_state.arcane_ward_hp.unwrap_or(0);
                    if ward_hp > 0 {
                        let absorbed = remaining_damage.min(ward_hp as f64);
                        t.final_state.arcane_ward_hp = Some((ward_hp as f64 - absorbed).round() as u32);
                        remaining_damage -= absorbed;
                        ward_absorbed_amount = absorbed;

                        if log_enabled && absorbed > 0.0 {
                            log.push(format!(
                                "         (Arcane Ward absorbs {:.0} damage, {:.0} remaining)",
                                absorbed,
                                t.final_state.arcane_ward_hp.unwrap_or(0)
                            ));
                        }
                    }

                    // 2. Temp HP Absorption (only if damage remains)
                    if remaining_damage > 0.0 {
                        if let Some(mut thp) = t.final_state.temp_hp {
                            if thp > 0 {
                                let absorbed = remaining_damage.min(thp as f64);
                                thp = (thp as f64 - absorbed).round() as u32;
                                remaining_damage -= absorbed;
                                _thp_absorbed_amount = absorbed;
                                t.final_state.temp_hp = if thp > 0 { Some(thp) } else { None };

                                if log_enabled && absorbed > 0.0 {
                                    log.push(format!(
                                        "         (Temp HP absorbs {:.0} damage)",
                                        absorbed
                                    ));
                                }
                            }
                        }
                    }

                    // 3. Real HP Damage
                    if remaining_damage > 0.0 {
                        let current = t.final_state.current_hp as f64;
                        let new_hp = (current - remaining_damage).max(0.0).round() as u32;
                        t.final_state.current_hp = new_hp;
                    }

                    // Log total effective damage taken by creature (ignoring ward/thp absorption implies logical impact)
                    // But usually logs show "Taken X Damage". Here we might want to clarify.
                    if log_enabled {
                        log.push(format!(
                            "   * 💥 {} takes {:.0} damage (HP: {})",
                            target_name,
                            damage - ward_absorbed_amount,
                            t.final_state.current_hp
                        ));
                    }

                    update_stats(
                        stats,
                        &attacker.id,
                        &t.id,
                        damage - ward_absorbed_amount,
                        0.0,
                    );

                    // Concentration Check (Damage > 0 to Real HP ??? Or just any damage?)
                    // 5e Rules: "Whenever you take damage..."
                    // Arcane Ward: "Standard ward takes damage INSTEAD of you". So if ward takes it all, you take 0 damage -> No check.
                    // Temp HP: You still take damage, it just comes off THP.
                    // PHB: "Temporary hit points... If you have 0 hit points, receiving temporary hit points doesn't restore..."
                    // PHB: "Concentration... whenever you take damage."
                    // Jeremy Crawford: "If the damage to you is 0 (e.g. Immunity or Ward), you don't make the check. If you take damage to THP, you DO make the check."

                    // So: Check if remaining_damage > 0 OR (absorbed by THP > 0 and NOT ward).
                    // Actually, if Ward absorbs ALL, remaining_damage entering THP phase is 0.
                    // If Ward absorbs partial, remaining_damage > 0 hits THP.
                    // If THP absorbs all, real HP damage is 0, but YOU TOOK DAMAGE (to THP).
                    // So logic: `damage - ward_absorbed` is the "damage taken by creature".

                    let damage_taken_by_creature = damage - ward_absorbed_amount;

                    if t.final_state.current_hp == 0 {
                        cleanup_instructions
                            .push(CleanupInstruction::RemoveAllBuffsFromSource(t.id.clone()));
                        if log_enabled {
                            log.push(format!("  * 💀 **{} falls unconscious!**", t.creature.name));
                        }
                    } else if damage_taken_by_creature > 0.0 {
                        // Only check concentration if creature actually took damage (not just ward)
                        if let Some(buff_id) = t.final_state.concentrating_on.clone() {
                            let dc = (damage_taken_by_creature / 2.0).max(10.0);
                            let con_save =
                                dice::evaluate(&DiceFormula::Expr("1d20".to_string()), 1);
                            let bonus = t.creature.con_save_bonus.unwrap_or(0.0);

                            if con_save + bonus < dc {
                                cleanup_instructions.push(CleanupInstruction::BreakConcentration(
                                    t.id.clone(),
                                    buff_id.clone(),
                                ));
                                if log_enabled {
                                    log.push(format!(
                                        "         -> Drops concentration on {}!",
                                        buff_id
                                    ));
                                }
                            }
                        }
                    }
                } else {
                    // Self Damage (Simplification: Treat same as target)
                    let mut remaining_damage = damage;
                    let mut ward_absorbed_amount = 0.0;
                    let mut _thp_absorbed_amount = 0.0;

                    // 1. Arcane Ward Absorption
                    let ward_hp = attacker.final_state.arcane_ward_hp.unwrap_or(0);
                    if ward_hp > 0 {
                        let absorbed = remaining_damage.min(ward_hp as f64);
                        attacker.final_state.arcane_ward_hp = Some((ward_hp as f64 - absorbed).round() as u32);
                        remaining_damage -= absorbed;
                        ward_absorbed_amount = absorbed;
                        if log_enabled && absorbed > 0.0 {
                            log.push(format!("         (Arcane Ward absorbs {:.0})", absorbed));
                        }
                    }

                    // 2. Temp HP Absorption
                    if remaining_damage > 0.0 {
                        if let Some(mut thp) = attacker.final_state.temp_hp {
                            if thp > 0 {
                                let absorbed = remaining_damage.min(thp as f64);
                                thp = (thp as f64 - absorbed).round() as u32;
                                remaining_damage -= absorbed;
                                _thp_absorbed_amount = absorbed;
                                attacker.final_state.temp_hp =
                                    if thp > 0 { Some(thp) } else { None };
                            }
                        }
                    }

                    if remaining_damage > 0.0 {
                        let current = attacker.final_state.current_hp as f64;
                        let new_hp = (current - remaining_damage).max(0.0).round() as u32;
                        attacker.final_state.current_hp = new_hp;
                    }

                    let damage_taken_by_creature = damage - ward_absorbed_amount;
                    update_stats(
                        stats,
                        &attacker.id,
                        &attacker.id,
                        damage_taken_by_creature,
                        0.0,
                    );
                    if log_enabled {
                        log.push(format!(
                            "         Self-damaged for {:.0} real HP",
                            damage_taken_by_creature
                        ));
                    }

                    // Self-concentration check logic
                    if damage_taken_by_creature > 0.0 {
                        if let Some(buff_id) = attacker.final_state.concentrating_on.clone() {
                            let dc = (damage_taken_by_creature / 2.0).max(10.0);
                            let con_save =
                                dice::evaluate(&DiceFormula::Expr("1d20".to_string()), 1);
                            let bonus = attacker.creature.con_save_bonus.unwrap_or(0.0);

                            if con_save + bonus < dc {
                                cleanup_instructions.push(CleanupInstruction::BreakConcentration(
                                    attacker.id.clone(),
                                    buff_id.clone(),
                                ));
                                if log_enabled {
                                    log.push(format!(
                                        "         -> Drops concentration on {}!",
                                        buff_id
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        Action::Heal(a) => {
            let amount = dice::evaluate(&a.amount, 1);
            if let Some(t) = target_opt {
                // Check if target actually needs healing
                if t.final_state.current_hp < t.creature.hp {
                    let max_hp = t.creature.hp as f64;
                    let old_hp = t.final_state.current_hp as f64;
                    let new_hp = (old_hp + amount).min(max_hp).round() as u32;
                    t.final_state.current_hp = new_hp;

                    update_stats(stats, &attacker.id, &t.id, 0.0, amount);
                    if log_enabled {
                        // Recalculate based on integer change if needed, but simplistic view:
                        log.push(format!(
                            "      -> Heals {} for {:.0} HP (was at {}/{})",
                            target_name, amount, old_hp as u32, t.creature.hp
                        ));
                    }
                } else {
                    // Target doesn't need healing, waste action
                    if log_enabled {
                        log.push(format!(
                            "      -> Skips healing on {} - already at full HP ({}/{})",
                            target_name, t.final_state.current_hp, t.creature.hp
                        ));
                    }
                }
            } else {
                let max_hp = attacker.creature.hp as f64;
                let old_hp = attacker.final_state.current_hp as f64;
                let new_hp = (old_hp + amount).min(max_hp).round() as u32;
                attacker.final_state.current_hp = new_hp;

                update_stats(stats, &attacker.id, &attacker.id, 0.0, amount);
                if log_enabled {
                    log.push(format!("      -> Heals self for {:.0} HP", amount));
                }
            }
        }
        Action::Buff(a) => {
            if log_enabled {
                let spell_name = a.buff.display_name.as_deref().unwrap_or(&a.name);

                // Enhanced Bless/Bane logging
                if let Some(to_hit_bonus) = &a.buff.to_hit {
                    if let Some(name) = &a.buff.display_name {
                        if name.contains("Bless") {
                            let bonus_val = dice::evaluate(to_hit_bonus, 1);
                            log.push(format!("      -> Casts {} on {} (grants +{:.0} to attack rolls and saving throws){}",
                                spell_name, target_name, bonus_val,
                                if a.buff.concentration { " (Concentration)" } else { "" }));
                        } else if name.contains("Bane") {
                            let penalty_val = dice::evaluate(to_hit_bonus, 1);
                            log.push(format!("      -> Casts {} on {} (subtracts -{:.0} from attack rolls and saving throws){}",
                                spell_name, target_name, penalty_val,
                                if a.buff.concentration { " (Concentration)" } else { "" }));
                        } else {
                            log.push(format!(
                                "      -> Casts {} on {}{}",
                                spell_name,
                                target_name,
                                if a.buff.concentration {
                                    " (Concentration)"
                                } else {
                                    ""
                                }
                            ));
                        }
                    } else {
                        log.push(format!(
                            "      -> Casts {} on {}{}",
                            spell_name,
                            target_name,
                            if a.buff.concentration {
                                " (Concentration)"
                            } else {
                                ""
                            }
                        ));
                    }
                } else {
                    log.push(format!(
                        "      -> Casts {} on {}{}",
                        spell_name,
                        target_name,
                        if a.buff.concentration {
                            " (Concentration)"
                        } else {
                            ""
                        }
                    ));
                }
            }
            if a.buff.concentration {
                let new_buff_id = a.base().id.clone();
                let current_conc = attacker.final_state.concentrating_on.clone();

                // Only break concentration if it's a different spell
                if let Some(old_buff) = current_conc {
                    if old_buff != new_buff_id {
                        cleanup_instructions.push(CleanupInstruction::BreakConcentration(
                            attacker.id.clone(),
                            old_buff.clone(),
                        ));
                        if log_enabled {
                            log.push(format!("         -> Drops concentration on {}!", old_buff));
                        }
                    }
                }
                attacker.final_state.concentrating_on = Some(new_buff_id);
            }
            let mut buff = a.buff.clone();
            buff.source = Some(attacker.id.clone());

            if let Some(t) = target_opt {
                t.final_state.buffs.insert(a.base().id.clone(), buff);
                update_stats_buff(stats, &attacker.id, &t.id, true);
            } else {
                attacker.final_state.buffs.insert(a.base().id.clone(), buff);
                update_stats_buff(stats, &attacker.id, &attacker.id, true);
            }
        }
        Action::Debuff(a) => {
            if a.buff.concentration {
                let new_buff_id = a.base().id.clone();
                let current_conc = attacker.final_state.concentrating_on.clone();

                // Only break concentration if it's a different spell
                if let Some(old_buff) = current_conc {
                    if old_buff != new_buff_id {
                        cleanup_instructions.push(CleanupInstruction::BreakConcentration(
                            attacker.id.clone(),
                            old_buff.clone(),
                        ));
                        if log_enabled {
                            log.push(format!("         (Drops concentration on {})", old_buff));
                        }
                    }
                }
                attacker.final_state.concentrating_on = Some(new_buff_id);
            }

            let dc_val = a.save_dc;
            let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1);
            let base_save_bonus = if let Some(t) = &target_opt {
                t.creature.save_bonus
            } else {
                attacker.creature.save_bonus
            };

            // Calculate Bless bonuses and Bane penalties for saving throws
            let mut bless_bonus = 0.0;
            let mut bane_penalty = 0.0;
            let mut buff_details = Vec::new();

            if let Some(t) = &target_opt {
                for b in t.final_state.buffs.values() {
                    if let Some(f) = &b.save {
                        let val = dice::evaluate(f, 1);
                        if let Some(name) = &b.display_name {
                            if name.contains("Bless") {
                                bless_bonus += val;
                                buff_details.push(format!("{}={:.0}", name, val));
                            } else if name.contains("Bane") {
                                bane_penalty += val;
                                buff_details.push(format!("{}=-{:.0}", name, val));
                            }
                        }
                    }
                }
            } else {
                for b in attacker.final_state.buffs.values() {
                    if let Some(f) = &b.save {
                        let val = dice::evaluate(f, 1);
                        if let Some(name) = &b.display_name {
                            if name.contains("Bless") {
                                bless_bonus += val;
                                buff_details.push(format!("{}={:.0}", name, val));
                            } else if name.contains("Bane") {
                                bane_penalty += val;
                                buff_details.push(format!("{}=-{:.0}", name, val));
                            }
                        }
                    }
                }
            }

            let save_bonus = base_save_bonus + bless_bonus - bane_penalty;
            let roll = rng::get_rng().gen_range(1..=20) as f64;

            if log_enabled {
                let display_name = a.buff.display_name.as_deref().unwrap_or(&a.name);
                let bonus_breakdown = if buff_details.is_empty() {
                    format!("{:.0}", base_save_bonus)
                } else {
                    format!("{:.0} + {}", base_save_bonus, buff_details.join(" + "))
                };
                log.push(format!(
                    "      -> Debuff {} vs {}: DC {:.0} vs Save {:.0} (Rolled {:.0} + {})",
                    display_name,
                    target_name,
                    dc,
                    roll + save_bonus,
                    roll,
                    bonus_breakdown
                ));
            }

            if roll + save_bonus < dc {
                let mut buff = a.buff.clone();
                buff.source = Some(attacker.id.clone());
                if let Some(t) = target_opt {
                    t.final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &t.id, false);
                } else {
                    attacker.final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &attacker.id, false);
                }
                if log_enabled {
                    log.push("         Failed! Debuff applied.".to_string());
                }
            } else if log_enabled {
                log.push("         Saved!".to_string());
            }
        }
        Action::Template(a) => {
            if log_enabled {
                log.push(format!(
                    "    - Applying template: {}",
                    a.template_options.template_name
                ));
            }

            // Implement template resolution based on template name
            let template_name = a.template_options.template_name.as_str();
            let is_concentration =
                matches!(template_name, "Hunter's Mark" | "Hex" | "Bless" | "Bane");

            if is_concentration {
                let new_buff_id = a.base().id.clone();
                let current_conc = attacker.final_state.concentrating_on.clone();

                // Break old concentration if casting a different spell
                if let Some(old_buff) = current_conc {
                    if old_buff != new_buff_id {
                        cleanup_instructions.push(CleanupInstruction::BreakConcentration(
                            attacker.id.clone(),
                            old_buff.clone(),
                        ));
                        if log_enabled {
                            log.push(format!("         -> Drops concentration on {}!", old_buff));
                        }
                    }
                }

                // Set new concentration
                attacker.final_state.concentrating_on = Some(new_buff_id.clone());

                // Apply the buff/debuff based on template type
                let mut buff = Buff {
                    display_name: Some(template_name.to_string()),
                    duration: BuffDuration::EntireEncounter,
                    concentration: true,
                    source: Some(attacker.id.clone()),
                    damage: None,
                    to_hit: None,
                    save: None,
                    dc: None,
                    ac: None,
                    damage_reduction: None,
                    damage_multiplier: None,
                    damage_taken_multiplier: None,
                    condition: None,
                    magnitude: None,
                    triggers: Vec::new(),
                };

                // Configure buff based on template
                match template_name {
                    "Hunter's Mark" | "Hex" => {
                        // Mark grants +1d6 damage on attacks (simplified: +3.5 avg)
                        buff.damage = Some(DiceFormula::Value(3.5));
                    }
                    "Bless" => {
                        // Bless grants +1d4 to attacks and saves
                        buff.to_hit = Some(DiceFormula::Expr("1d4".to_string()));
                        buff.save = Some(DiceFormula::Expr("1d4".to_string()));
                    }
                    "Bane" => {
                        // Bane subtracts 1d4 from attacks and saves
                        buff.to_hit = Some(DiceFormula::Expr("-1d4".to_string()));
                        buff.save = Some(DiceFormula::Expr("-1d4".to_string()));
                    }
                    _ => {}
                }

                // Apply to target
                if let Some(t) = target_opt {
                    t.final_state.buffs.insert(new_buff_id, buff);
                    update_stats_buff(stats, &attacker.id, &t.id, true);
                    if log_enabled {
                        log.push(format!(
                            "      Template {} applied to {}",
                            template_name, target_name
                        ));
                    }
                } else {
                    // Self-target (e.g., Bless on self)
                    attacker.final_state.buffs.insert(new_buff_id, buff);
                    update_stats_buff(stats, &attacker.id, &attacker.id, true);
                    if log_enabled {
                        log.push(format!("      Template {} applied to self", template_name));
                    }
                }
            } else {
                // Non-concentration template (placeholder)
                if log_enabled {
                    log.push(format!(
                        "      Template {} applied (non-concentration)",
                        template_name
                    ));
                }
            }
        }
    }
    cleanup_instructions
}

// Public function to orchestrate the action application to all targets
// This handles the complex slice splitting and iteration
#[allow(clippy::too_many_arguments)]
pub fn resolve_action_execution(
    attacker_index: usize,
    allies: &mut [Combattant],
    enemies: &mut [Combattant],
    action: &Action,
    raw_targets: &[(bool, usize)],
    action_record: &CombattantAction, // To be pushed to attacker's history
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
) -> Vec<CleanupInstruction> {
    let mut all_cleanup = Vec::new();

    // Split allies slice to get mutable attacker
    let (allies_head, allies_tail) = allies.split_at_mut(attacker_index);
    let (attacker_mut, allies_after_attacker) = allies_tail
        .split_first_mut()
        .expect("Attacker index out of bounds");

    // 1. Mark action as used (for turn-based economy) and record it (Attacker state update)
    attacker_mut
        .final_state
        .used_actions
        .insert(action.base().id.clone());
    attacker_mut.actions.push(action_record.clone());

    // NEW: Mark action as used for the encounter
    attacker_mut
        .final_state
        .actions_used_this_encounter
        .insert(action.base().id.clone());

    // NEW: Mark bonus action as used if this was a bonus action
    if action.base().action_slot == Some(1) {
        attacker_mut.final_state.bonus_action_used = true;
    }

    // Decrement remaining uses if applicable
    match &action.base().freq {
        Frequency::Limited { .. } => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut
                .final_state
                .resources
                .current
                .get(&action_id)
                .unwrap_or(&0.0);
            attacker_mut
                .final_state
                .resources
                .current
                .insert(action_id, (current_uses - 1.0).max(0.0));
        }
        Frequency::Static(s) if s != "at will" => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut
                .final_state
                .resources
                .current
                .get(&action_id)
                .unwrap_or(&0.0);
            attacker_mut
                .final_state
                .resources
                .current
                .insert(action_id, (current_uses - 1.0).max(0.0));
        }
        Frequency::Recharge { .. } => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut
                .final_state
                .resources
                .current
                .get(&action_id)
                .unwrap_or(&0.0);
            attacker_mut
                .final_state
                .resources
                .current
                .insert(action_id, (current_uses - 1.0).max(0.0));
        }
        _ => {} // No decrement for "at will"
    }

    // 2. Iterate targets and apply effects
    let mut used_enemy_targets = Vec::new();

    for (is_target_enemy, mut target_idx) in raw_targets.iter().copied() {
        if is_target_enemy {
            // Verify target is still alive (might have died in previous iteration)
            if enemies[target_idx].final_state.current_hp == 0 {
                // Target died - try to find a new one
                if let Action::Atk(atk_action) = action {
                    if let Some(new_idx) = crate::targeting::select_enemy_target(
                        attacker_mut,
                        atk_action.target.clone(),
                        enemies,
                        &[],
                        None,
                    ) {
                        if log_enabled {
                            log.push(format!(
                                "      -> {} is unconscious, switching target to {}",
                                enemies[target_idx].creature.name, enemies[new_idx].creature.name
                            ));
                        }
                        target_idx = new_idx;
                    } else {
                        // No valid targets at all - skip this attack
                        if log_enabled {
                            log.push(
                                "      -> No valid targets available, skipping attack".to_string(),
                            );
                        }
                        continue;
                    }
                } else {
                    // Non-attack action - skip if target dead
                    if log_enabled {
                        log.push(format!(
                            "      -> {} is already unconscious, skipping action",
                            enemies[target_idx].creature.name
                        ));
                    }
                    continue;
                }
            }

            used_enemy_targets.push((true, target_idx));

            // Target is enemy (safe to borrow from enemies slice)
            let instructions = apply_single_effect(
                attacker_mut,
                Some(&mut enemies[target_idx]),
                action,
                stats,
                log,
                log_enabled,
            );
            all_cleanup.extend(instructions);
        } else {
            // Target is ally (need to find correct mutable reference)
            if target_idx == attacker_index {
                // Self-targeting: pass None as target_opt
                let instructions =
                    apply_single_effect(attacker_mut, None, action, stats, log, log_enabled);
                all_cleanup.extend(instructions);
            } else {
                let target_mut = if target_idx < attacker_index {
                    &mut allies_head[target_idx]
                } else {
                    &mut allies_after_attacker[target_idx - attacker_index - 1]
                };

                let instructions = apply_single_effect(
                    attacker_mut,
                    Some(target_mut),
                    action,
                    stats,
                    log,
                    log_enabled,
                );
                all_cleanup.extend(instructions);
            }
        }
    }

    all_cleanup
}
