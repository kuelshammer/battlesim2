use std::collections::HashMap;
use crate::model::*;
use crate::enums::*;
use crate::dice;
use crate::actions::get_attack_roll_result;
use rand::Rng;

// Helper to update encounter stats
pub fn update_stats(stats: &mut HashMap<String, EncounterStats>, attacker_id: &str, target_id: &str, damage: f64, heal: f64) {
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

// Helper to update encounter stats for buffs/debuffs
pub fn update_stats_buff(stats: &mut HashMap<String, EncounterStats>, attacker_id: &str, target_id: &str, is_buff: bool) {
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

// Function to handle defensive triggers (e.g., Shield spell)
fn process_defensive_triggers(
    _attacker: &Combattant, // Attacker (immutable)
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
    if target.final_state.used_actions.contains(&(reaction_slot_id as i32).to_string()) {
        return (final_ac, reaction_used);
    }

    let target_triggers = target.creature.triggers.clone(); 

    for trigger in target_triggers.iter() {
        if trigger.condition == TriggerCondition::OnBeingAttacked {
            if let Some(cost_slot) = trigger.cost {
                if cost_slot == (ActionSlot::Reaction as i32) {
                    if total_hit_roll >= final_ac { // Currently a hit
                        if let Action::Buff(buff_action) = &trigger.action {
                            if let Some(ac_buff_dice) = &buff_action.buff.ac {
                                let potential_ac_buff = dice::evaluate(ac_buff_dice, 1);
                                if total_hit_roll < final_ac + potential_ac_buff {
                                    // Trigger activates
                                    reaction_used = true;
                                    target.final_state.used_actions.insert((reaction_slot_id as i32).to_string());
                                    
                                    let mut buff = buff_action.buff.clone();
                                    buff.source = Some(target.id.clone()); 

                                    target.final_state.buffs.insert(buff_action.base().id.clone(), buff);
                                    update_stats_buff(stats, &target.id, &target.id, true);

                                    final_ac += potential_ac_buff;
                                    if log_enabled {
                                        log.push(format!("          {} uses {} to increase AC by {:.0} (New AC: {:.0})",
                                            target.creature.name, buff_action.base().name, potential_ac_buff, final_ac));
                                    }
                                    break; 
                                }
                            }
                        }
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
    _main_action: &Action, // The main action that just hit
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
        let mut trigger_should_fire = false;
        if trigger.condition == TriggerCondition::OnHit {
            trigger_should_fire = true;
        } else if trigger.condition == TriggerCondition::OnCriticalHit && is_crit {
            trigger_should_fire = true;
        }

        if trigger_should_fire {
            // Check if trigger is usable (resource cost)
            // For now, assume "at will" or always usable if it has no explicit cost / resource tracking.
            // If the trigger has a cost that needs to be tracked by `remaining_uses`, it would be decremented here.

            if log_enabled {
                log.push(format!("          {} triggers {} on hit/crit!", attacker.creature.name, trigger.action.base().name));
            }

            // Execute the trigger action
            match &trigger.action {
                Action::Atk(a) => {
                    // This is extra damage from a rider effect
                    let extra_damage = dice::evaluate(&a.dpr, if is_crit { 2 } else { 1 });
                    additional_damage += extra_damage;
                    if log_enabled {
                        log.push(format!("             -> Adds {:.0} additional damage ({})", extra_damage, a.base().name));
                    }
                },
                Action::Buff(a) => {
                    // Apply buff from trigger
                    let mut buff = a.buff.clone();
                    buff.source = Some(attacker.id.clone());
                    target.final_state.buffs.insert(a.base().id.clone(), buff);
                    update_stats_buff(stats, &attacker.id, &target.id, true);
                    if log_enabled {
                        log.push(format!("             -> Applies buff {} to {}", a.base().name, target.creature.name));
                    }
                },
                Action::Debuff(a) => {
                    // Apply debuff from trigger
                    let dc_val = a.save_dc;
                    let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1);
                    let save_bonus = target.creature.save_bonus;
                    let roll = rand::thread_rng().gen_range(1..=20) as f64;
                    
                    if log_enabled {
                        log.push(format!("             -> Debuff {} vs {}: DC {:.0} vs Save {:.0} (Rolled {:.0} + {:.0})", 
                            a.buff.display_name.as_deref().unwrap_or("Unknown"), target.creature.name, dc, roll + save_bonus, roll, save_bonus));
                    }

                    if roll + save_bonus < dc {
                        let mut buff = a.buff.clone();
                        buff.source = Some(attacker.id.clone());
                        target.final_state.buffs.insert(a.base().id.clone(), buff);
                        update_stats_buff(stats, &attacker.id, &target.id, false);
                        if log_enabled { log.push(format!("             Failed! Debuff applied.")); }
                    } else {
                        if log_enabled { log.push(format!("             Saved!")); }
                    }
                },
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
    let target_name = if let Some(t) = &target_opt { t.creature.name.clone() } else { attacker.creature.name.clone() };
    let _target_id = if let Some(t) = &target_opt { t.id.clone() } else { attacker.id.clone() };

    match action {
        Action::Atk(a) => {
            let (roll, is_crit, is_miss) = get_attack_roll_result(attacker);
            let to_hit_bonus = dice::evaluate(&a.to_hit, 1);
            
            let mut buff_bonus = 0.0;
            let mut buff_details = Vec::new();
            
            for b in attacker.final_state.buffs.values() {
                if let Some(f) = &b.to_hit {
                    let val = dice::evaluate(f, 1);
                    buff_bonus += val;
                    if let Some(name) = &b.display_name {
                         buff_details.push(format!("{}={:.0}", name, val));
                    } else {
                         buff_details.push(format!("{:.0}", val));
                    }
                }
            }
            
            let total_hit = roll + to_hit_bonus + buff_bonus;
            
            // Resolve Target Stats (Read)
            // Re-borrow target_opt immutably
            let (target_ac, target_buff_ac) = if let Some(t) = &target_opt {
                (t.creature.ac, t.final_state.buffs.values().filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1))).sum::<f64>())
            } else {
                (attacker.creature.ac, attacker.final_state.buffs.values().filter_map(|b| b.ac.as_ref().map(|f| dice::evaluate(f, 1))).sum::<f64>())
            };
            let total_ac_before_trigger = target_ac + target_buff_ac;

            // Trigger processing requires mutable access to target
            // We need to be careful not to borrow attacker if target IS attacker
            
            // Re-borrow target_opt mutably
            let (final_ac, reaction_used) = if let Some(t) = &mut target_opt {
                // Distinct target (re-borrowed *t)
                process_defensive_triggers(
                    attacker,
                    *t,
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
            
            if log_enabled {
                let crit_str = if is_crit { " (CRIT!)" } else if is_miss { " (MISS!)" } else { "" };
                let reaction_str = if reaction_used { " (Reaction Used)" } else { "" };

                // Enhanced Bless/Bane logging
                let bless_details: Vec<String> = buff_details.iter()
                    .filter(|d| d.contains("Bless"))
                    .cloned()
                    .collect();
                let bane_details: Vec<String> = buff_details.iter()
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

                log.push(format!("      -> Attack vs {}: Rolled {:.0} + {:.0} (base bonus) + {:.0} (buffs){}{} = {:.0} vs AC {:.0} (Base {:.0} + {:.0} target buffs){}{}. Result: {}",
                    target_name, roll, to_hit_bonus, buff_bonus, effect_str,
                    if !buff_details.is_empty() && (bless_details.is_empty() && bane_details.is_empty()) {
                        let other_buffs: Vec<String> = buff_details.iter()
                            .filter(|d| !d.contains("Bless") && !d.contains("Bane"))
                            .cloned()
                            .collect();
                        if !other_buffs.is_empty() {
                            format!(" (other: {})", other_buffs.join(", "))
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    },
                    total_hit, final_ac, target_ac, target_buff_ac, reaction_str, crit_str, if hits { "HIT" } else { "MISS" }));
            }

            if hits {
                // Calculate base damage and buff damage first
                let mut damage = dice::evaluate(&a.dpr, if is_crit { 2 } else { 1 });
                let buff_dmg: f64 = attacker.final_state.buffs.values()
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
                            if let Some(name) = &b.display_name {
                                if mult < 1.0 {
                                    multiplier_sources.push(format!("{} (Resisted)", name));
                                } else if mult > 1.0 {
                                    multiplier_sources.push(format!("{} (Vulnerable)", name));
                                }
                            }
                        }
                    }
                }

                let damage_before_multiplier = damage;
                damage = (damage * total_multiplier).floor(); // Round down damage in 5e

                if log_enabled {
                    let mut calc_log = format!("         Damage: {:.0} (Base) + {:.0} (Buffs)", damage_before_reduction - buff_dmg, buff_dmg);
                    
                    if flat_reduction > 0.0 {
                        calc_log.push_str(&format!(" - {:.0} ({})", flat_reduction, reduction_sources.join(", ")));
                    }
                    
                    if total_multiplier != 1.0 {
                        calc_log.push_str(&format!(" * {:.2} ({})", total_multiplier, multiplier_sources.join(", ")));
                    }
                    
                    calc_log.push_str(&format!(" = {:.0}", damage));
                    log.push(calc_log);
                }

                // Apply Damage (Write)
                // Consume target_opt (move) since this is the last usage
                if let Some(t) = target_opt {
                    t.final_state.current_hp -= damage;
                    if t.final_state.current_hp < 0.0 { t.final_state.current_hp = 0.0; }
                    update_stats(stats, &attacker.id, &t.id, damage, 0.0);
                    
                    if t.final_state.current_hp <= 0.0 {
                        cleanup_instructions.push(CleanupInstruction::RemoveAllBuffsFromSource(t.id.clone()));
                        if log_enabled { log.push(format!("         {} falls unconscious!", t.creature.name)); }
                    } else if let Some(buff_id) = t.final_state.concentrating_on.clone() {
                        let dc = (damage / 2.0).max(10.0);
                        let con_save = dice::evaluate(&DiceFormula::Expr("1d20".to_string()), 1); 
                        let bonus = t.creature.con_save_bonus.unwrap_or(0.0);
                        
                        if con_save + bonus < dc {
                            cleanup_instructions.push(CleanupInstruction::BreakConcentration(t.id.clone(), buff_id.clone()));
                            if log_enabled { log.push(format!("         -> Drops concentration on {}!", buff_id)); }
                        }
                    }
                } else {
                    // Self Damage
                    attacker.final_state.current_hp -= damage;
                    if attacker.final_state.current_hp < 0.0 { attacker.final_state.current_hp = 0.0; }
                    update_stats(stats, &attacker.id, &attacker.id, damage, 0.0);
                    if log_enabled { log.push(format!("         Self-damaged for {:.0} HP", damage)); }
                    // No self-concentration check logic for now
                }
            }
        },
        Action::Heal(a) => {
            let amount = dice::evaluate(&a.amount, 1);
            if let Some(t) = target_opt {
                // Check if target actually needs healing
                if t.final_state.current_hp < t.creature.hp {
                    t.final_state.current_hp += amount;
                    if t.final_state.current_hp > t.creature.hp { t.final_state.current_hp = t.creature.hp; }
                    update_stats(stats, &attacker.id, &t.id, 0.0, amount);
                    if log_enabled {
                        log.push(format!("      -> Heals {} for {:.0} HP (was at {:.0}/{:.0})", target_name, amount, t.final_state.current_hp - amount, t.creature.hp));
                    }
                } else {
                    // Target doesn't need healing, waste action
                    if log_enabled {
                        log.push(format!("      -> Skips healing on {} - already at full HP ({:.0}/{:.0})", target_name, t.final_state.current_hp, t.creature.hp));
                    }
                }
            } else {
                attacker.final_state.current_hp += amount;
                if attacker.final_state.current_hp > attacker.creature.hp { attacker.final_state.current_hp = attacker.creature.hp; }
                update_stats(stats, &attacker.id, &attacker.id, 0.0, amount);
                if log_enabled {
                    log.push(format!("      -> Heals self for {:.0} HP", amount));
                }
            }
        },
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
                            log.push(format!("      -> Casts {} on {}{}", spell_name, target_name,
                                if a.buff.concentration { " (Concentration)" } else { "" }));
                        }
                    } else {
                        log.push(format!("      -> Casts {} on {}{}", spell_name, target_name,
                            if a.buff.concentration { " (Concentration)" } else { "" }));
                    }
                } else {
                    log.push(format!("      -> Casts {} on {}{}", spell_name, target_name,
                        if a.buff.concentration { " (Concentration)" } else { "" }));
                }
            }
            if a.buff.concentration {
                let new_buff_id = a.base().id.clone();
                let current_conc = attacker.final_state.concentrating_on.clone();
                
                // Only break concentration if it's a different spell
                if let Some(old_buff) = current_conc {
                    if old_buff != new_buff_id {
                        cleanup_instructions.push(CleanupInstruction::BreakConcentration(attacker.id.clone(), old_buff.clone()));
                        if log_enabled { log.push(format!("         -> Drops concentration on {}!", old_buff)); }
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
        },
        Action::Debuff(a) => {
            if a.buff.concentration {
                let new_buff_id = a.base().id.clone();
                let current_conc = attacker.final_state.concentrating_on.clone();
                
                // Only break concentration if it's a different spell
                if let Some(old_buff) = current_conc {
                     if old_buff != new_buff_id {
                        cleanup_instructions.push(CleanupInstruction::BreakConcentration(attacker.id.clone(), old_buff.clone()));
                        if log_enabled { log.push(format!("         (Drops concentration on {})", old_buff)); }
                     }
                }
                attacker.final_state.concentrating_on = Some(new_buff_id);
            }
            
            let dc_val = a.save_dc;
            let dc = dice::evaluate(&DiceFormula::Value(dc_val), 1);
            let base_save_bonus = if let Some(t) = &target_opt { t.creature.save_bonus } else { attacker.creature.save_bonus };

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
            let roll = rand::thread_rng().gen_range(1..=20) as f64;
            
            
            if log_enabled {
                let display_name = a.buff.display_name.as_deref().unwrap_or(&a.name);
                let bonus_breakdown = if buff_details.is_empty() {
                    format!("{:.0}", base_save_bonus)
                } else {
                    format!("{:.0} + {}", base_save_bonus, buff_details.join(" + "))
                };
                log.push(format!("      -> Debuff {} vs {}: DC {:.0} vs Save {:.0} (Rolled {:.0} + {})",
                    display_name, target_name, dc, roll + save_bonus, roll, bonus_breakdown));
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
                if log_enabled { log.push(format!("         Failed! Debuff applied.")); }
            } else {
                if log_enabled { log.push(format!("         Saved!")); }
            }
        },
        Action::Template(a) => {
            // For template actions, we should resolve them to their final form first
            // For now, we'll treat them as buff actions and apply the template effect
            if log_enabled { log.push(format!("    - Applying template: {}", a.template_options.template_name)); }

            // TODO: Implement proper template resolution
            // For now, this is a placeholder that just logs the template application
            // The actual template logic should be implemented based on templateOptions.templateName

            if log_enabled {
                log.push(format!("      Template {} applied to target", a.template_options.template_name));
            }
        },
    }
    cleanup_instructions
}

// Public function to orchestrate the action application to all targets
// This handles the complex slice splitting and iteration
pub fn resolve_action_execution(
    attacker_index: usize,
    allies: &mut [Combattant],
    enemies: &mut [Combattant],
    action: &Action,
    raw_targets: &Vec<(bool, usize)>,
    action_record: &CombattantAction, // To be pushed to attacker's history
    stats: &mut HashMap<String, EncounterStats>,
    log: &mut Vec<String>,
    log_enabled: bool,
) -> Vec<CleanupInstruction> {
    let mut all_cleanup = Vec::new();

    // Split allies slice to get mutable attacker
    let (allies_head, allies_tail) = allies.split_at_mut(attacker_index);
    let (attacker_mut, allies_after_attacker) = allies_tail.split_first_mut().expect("Attacker index out of bounds");

    // 1. Mark action as used (for turn-based economy) and record it (Attacker state update)
    attacker_mut.final_state.used_actions.insert(action.base().id.clone());
    attacker_mut.actions.push(action_record.clone());

    // NEW: Mark action as used for the encounter
    attacker_mut.final_state.actions_used_this_encounter.insert(action.base().id.clone());

    // NEW: Mark bonus action as used if this was a bonus action
    if action.base().action_slot == 1 {
        attacker_mut.final_state.bonus_action_used = true;
    }

    // Decrement remaining uses if applicable
    match &action.base().freq {
        Frequency::Limited { .. } => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut.final_state.remaining_uses.get(&action_id).unwrap_or(&0.0);
            attacker_mut.final_state.remaining_uses.insert(action_id, (current_uses - 1.0).max(0.0));
        },
        Frequency::Static(s) if s != "at will" => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut.final_state.remaining_uses.get(&action_id).unwrap_or(&0.0);
            attacker_mut.final_state.remaining_uses.insert(action_id, (current_uses - 1.0).max(0.0));
        },
        Frequency::Recharge { .. } => {
            let action_id = action.base().id.clone();
            let current_uses = *attacker_mut.final_state.remaining_uses.get(&action_id).unwrap_or(&0.0);
            attacker_mut.final_state.remaining_uses.insert(action_id, (current_uses - 1.0).max(0.0));
        },
        _ => {} // No decrement for "at will"
    }

    // 2. Iterate targets and apply effects
    // 2. Iterate targets and apply effects
    let mut used_enemy_targets = Vec::new();
    
    for (is_target_enemy, mut target_idx) in raw_targets.iter().copied() {
        if is_target_enemy {
            // For attacks, always re-select target dynamically based on current state
            // This makes "enemy with least HP" reactive to damage dealt
            if matches!(action, Action::Atk(_)) {
                if let Action::Atk(atk_action) = action {
                    // IMPORTANT: Pass empty exclusion list for attacks!
                    // Attacks can target the same enemy multiple times (e.g., Multiattack)
                    // The old exclusion system was breaking target selection after enemies died
                    if let Some(new_idx) = crate::targeting::select_enemy_target(
                        atk_action.target.clone(),
                        enemies,
                        &[],  // Empty exclusion - attacks don't exclude previously targeted enemies
                        None
                    ) {
                        target_idx = new_idx;
                    } else {
                        if log_enabled {
                            log.push(format!("      -> No targets available for attack"));
                        }
                        continue; // Skip if no targets
                    }
                }
            } else {
                // For non-attacks (debuffs), only re-select if target is dead
                if enemies[target_idx].final_state.current_hp <= 0.0 {
                    if log_enabled {
                        log.push(format!("      -> {} is already unconscious, skipping action", 
                            enemies[target_idx].creature.name));
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
                log_enabled
            );
            all_cleanup.extend(instructions);
        } else {
            // Target is ally (need to find correct mutable reference)
            if target_idx == attacker_index {
                // Self-targeting: pass None as target_opt
                let instructions = apply_single_effect(
                    attacker_mut,
                    None,
                    action,
                    stats,
                    log,
                    log_enabled
                );
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
                    log_enabled
                );
                all_cleanup.extend(instructions);
            }
        }
    }

    all_cleanup
}
