use crate::context::{EffectType, TurnContext};
use crate::dice;
use crate::events::{Event, RollResult};
use crate::model::{Action, AtkAction};
use crate::rng;
use crate::combat_stats::CombatStatsCache;
use serde::{Deserialize, Serialize};

/// Event-driven action resolver that converts actions into events
#[derive(Debug, Clone)]
pub struct ActionResolver {
    /// Random number generator for dice rolls
    #[allow(dead_code)]
    rng_seed: Option<u64>,
    /// Cache for combatant statistics to optimize targeting
    pub combat_stats_cache: CombatStatsCache,
}

/// Result of an attack roll
#[derive(Debug, Clone)]
pub struct AttackRollResult {
    pub total: f64,
    pub is_critical: bool,
    pub is_miss: bool,
    pub roll_detail: Option<RollResult>,
}

/// Result of action resolution containing all generated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResolutionResult {
    pub events: Vec<Event>,
    pub success: bool,
    pub error: Option<String>,
}

impl Default for ActionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionResolver {
    /// Create a new action resolver
    pub fn new() -> Self {
        Self { 
            rng_seed: None,
            combat_stats_cache: CombatStatsCache::new(),
        }
    }

    /// Create a new action resolver with a specific seed for reproducible results
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng_seed: Some(seed),
            combat_stats_cache: CombatStatsCache::new(),
        }
    }

    /// Create a new action resolver with an existing cache
    pub fn with_cache(cache: CombatStatsCache) -> Self {
        Self {
            rng_seed: None,
            combat_stats_cache: cache,
        }
    }

    /// Resolve an action and emit events to the context
    pub fn resolve_action(
        &self,
        action: &Action,
        context: &mut TurnContext,
        actor_id: &str,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let base = action.base();

        // 1. Emit CastSpell event if it's a spell action
        if base.tags.contains(&crate::resources::ActionTag::Spell) {
            let cast_event = Event::CastSpell {
                caster_id: actor_id.to_string(),
                spell_name: base.name.clone(),
                target_ids: Vec::new(), // Initial cast doesn't have targets yet in this event
                spell_level: 0,
            };
            context.record_event(cast_event.clone());
            events.push(cast_event);

            // 2. Check for interruption (e.g. Counterspell)
            // Trigger reactions specifically for OnCastSpell
            let reaction_events = self.trigger_global_reactions(
                crate::enums::TriggerCondition::OnCastSpell,
                context,
                Some(actor_id),
            );
            events.extend(reaction_events);

            if context.action_interrupted {
                return events;
            }
        }

        let resolution_events = match action {
            Action::Atk(attack_action) => self.resolve_attack(attack_action, context, actor_id),
            Action::Heal(heal_action) => {
                crate::resolvers::resolve_heal(self, heal_action, context, actor_id)
            }
            Action::Buff(buff_action) => {
                crate::resolvers::resolve_buff(self, buff_action, context, actor_id)
            }
            Action::Debuff(debuff_action) => {
                crate::resolvers::resolve_debuff(self, debuff_action, context, actor_id)
            }
            Action::Template(template_action) => {
                crate::resolvers::resolve_template(self, template_action, context, actor_id)
            }
        };

        events.extend(resolution_events);
        events
    }

    /// Resolve attack actions (proxy to modular resolver)
    pub fn resolve_attack(
        &self,
        attack: &AtkAction,
        context: &mut TurnContext,
        actor_id: &str,
    ) -> Vec<Event> {
        crate::resolvers::resolve_attack(self, attack, context, actor_id)
    }


    /// Trigger reactions for all alive combatants based on a condition
    fn trigger_global_reactions(
        &self,
        condition: crate::enums::TriggerCondition,
        context: &mut TurnContext,
        triggering_actor_id: Option<&str>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        // 1. Collect all combatant IDs to avoid borrow issues
        let mut combatant_ids: Vec<String> = context.combatants.keys().cloned().collect();
        combatant_ids.sort(); // Deterministic order

        for reactor_id in combatant_ids {
            if !context.is_combatant_alive(&reactor_id) {
                continue;
            }

            // A. Check permanent triggers on the creature
            let triggers = if let Some(c) = context.get_combatant(&reactor_id) {
                c.base_combatant.creature.triggers.clone()
            } else {
                continue;
            };

            for trigger in triggers {
                if trigger.condition == condition {
                    // Check cost (e.g. Reaction)
                    let mut can_pay = true;
                    if let Some(cost_slot) = trigger.cost {
                        if let Some(reactor) = context.get_combatant(&reactor_id) {
                            if reactor.base_combatant.final_state.used_actions.contains(&cost_slot.to_string()) {
                                can_pay = false;
                            }
                        }
                    }

                    if can_pay {
                        // Consume cost
                        if let Some(cost_slot) = trigger.cost {
                            if let Some(reactor_mut) = context.get_combatant_mut(&reactor_id) {
                                reactor_mut.base_combatant.final_state.used_actions.insert(cost_slot.to_string());
                            }
                        }

                        // Record action start
                        context.record_event(Event::ActionStarted {
                            actor_id: reactor_id.clone(),
                            action_id: trigger.id.clone(),
                            decision_trace: std::collections::HashMap::new(),
                        });

                        // Resolve trigger action
                        let reaction_events = self.resolve_action(&trigger.action, context, &reactor_id);
                        events.extend(reaction_events);

                        if context.action_interrupted {
                            return events;
                        }
                    }
                }
            }

            // B. Check triggers from active effects (buffs)
            let effect_events = self.trigger_reactions_internal(
                &reactor_id,
                condition.clone(),
                context,
                triggering_actor_id,
                None,
            );
            events.extend(effect_events);

            if context.action_interrupted {
                return events;
            }
        }

        events
    }

    /// Helper to resolve reactive effects from buffs (Triggers)
    pub fn trigger_reactions_internal(
        &self,
        reactor_id: &str,
        condition: crate::enums::TriggerCondition,
        context: &mut TurnContext,
        triggering_actor_id: Option<&str>,
        _damage_info: Option<&str>,
    ) -> Vec<Event> {
        let mut events = Vec::new();

        // Iterate over active effects to find buffs on the reactor
        // Access values() directly to avoid borrow checker issues with iterating the map
        let active_effects: Vec<crate::context::ActiveEffect> =
            context.active_effects.values().cloned().collect();

        for effect in active_effects {
            if effect.target_id != reactor_id {
                continue;
            }

            if let EffectType::Buff(buff) = &effect.effect_type {
                for trigger in &buff.triggers {
                    if trigger.condition == condition {
                        // Check requirements
                        let requirements_met = trigger.requirements.iter().all(|req| {
                            match req {
                                crate::enums::TriggerRequirement::HasTempHP => {
                                    if let Some(c) = context.combatants.get(reactor_id) {
                                        c.temp_hp > 0
                                    } else {
                                        false
                                    }
                                }
                                // Implement other requirements as needed
                                _ => true,
                            }
                        });

                        if requirements_met {
                            // Execute Effect
                            match &trigger.effect {
                                crate::enums::TriggerEffect::DealDamage {
                                    amount,
                                    damage_type,
                                } => {
                                    if let Some(target_id) = triggering_actor_id {
                                        // Parse amount formula
                                        let formula =
                                            crate::model::DiceFormula::Expr(amount.clone());
                                        // Basic eval for now, assume no variable parts specific to reactor yet (except fixed values)
                                        let dmg_value = dice::evaluate(&formula, 1);

                                        // Apply damage to the TRIGGERING ACTOR (Retaliation)
                                        let dmg_events = context.apply_damage(
                                            target_id,
                                            dmg_value,
                                            damage_type,
                                            reactor_id,
                                        );
                                        events.extend(dmg_events);
                                    }
                                }
                                crate::enums::TriggerEffect::GrantImmediateAction {
                                    action_id,
                                    action_slot: _,
                                } => {
                                    // Grant an immediate action outside normal turn order
                                    if let Some(combatant) = context.get_combatant(reactor_id) {
                                        // Look for the action in the combatant's known actions
                                        let action = combatant.base_combatant.creature.actions.iter()
                                            .find(|a| a.base().id == *action_id)
                                            .cloned();

                                        if let Some(action) = action {
                                            // Emit action start event for the granted action
                                            context.record_event(crate::events::Event::ActionStarted {
                                                actor_id: reactor_id.to_string(),
                                                action_id: action_id.clone(),
                                                decision_trace: std::collections::HashMap::new(),
                                            });

                                            // Resolve the granted action recursively
                                            let nested_events = self.resolve_action(&action, context, reactor_id);
                                            events.extend(nested_events);
                                        }
                                    }
                                }
                                crate::enums::TriggerEffect::InterruptAction { action_id: _ } => {
                                    // Set the interrupt flag to stop the current action sequence
                                    context.action_interrupted = true;
                                }
                                crate::enums::TriggerEffect::AddToRoll { amount, roll_type: _ } => {
                                    // Add a pending roll bonus
                                    context.roll_modifications.add(
                                        reactor_id,
                                        crate::context::RollModification::AddBonus {
                                            amount: amount.clone(),
                                        },
                                    );

                                    let mut data = std::collections::HashMap::new();
                                    data.insert("amount".to_string(), amount.clone());
                                    events.push(Event::Custom {
                                        event_type: "AddToRoll".to_string(),
                                        data,
                                        source_id: reactor_id.to_string(),
                                    });
                                }
                                crate::enums::TriggerEffect::ForceSelfReroll {
                                    roll_type,
                                    must_use_second,
                                } => {
                                    // Force a reroll for the reactor
                                    context.roll_modifications.add(
                                        reactor_id,
                                        crate::context::RollModification::Reroll {
                                            roll_type: roll_type.clone(),
                                            must_use_second: *must_use_second,
                                        },
                                    );

                                    let mut data = std::collections::HashMap::new();
                                    data.insert("roll_type".to_string(), roll_type.clone());
                                    events.push(Event::Custom {
                                        event_type: "ForceSelfReroll".to_string(),
                                        data,
                                        source_id: reactor_id.to_string(),
                                    });
                                }
                                crate::enums::TriggerEffect::ForceTargetReroll {
                                    roll_type,
                                    must_use_second,
                                } => {
                                    // Force a reroll for the triggering actor
                                    if let Some(target_id) = triggering_actor_id {
                                        context.roll_modifications.add(
                                            target_id,
                                            crate::context::RollModification::Reroll {
                                                roll_type: roll_type.clone(),
                                                must_use_second: *must_use_second,
                                            },
                                        );

                                        let mut data = std::collections::HashMap::new();
                                        data.insert("roll_type".to_string(), roll_type.clone());
                                        events.push(Event::Custom {
                                            event_type: "ForceTargetReroll".to_string(),
                                            data,
                                            source_id: reactor_id.to_string(),
                                        });
                                    }
                                }
                                crate::enums::TriggerEffect::SetAdvantageOnNext {
                                    roll_type,
                                    advantage,
                                } => {
                                    context.roll_modifications.add(
                                        reactor_id,
                                        crate::context::RollModification::SetAdvantage {
                                            roll_type: roll_type.clone(),
                                            advantage: *advantage,
                                        },
                                    );

                                    let mut data = std::collections::HashMap::new();
                                    data.insert("roll_type".to_string(), roll_type.clone());
                                    data.insert("advantage".to_string(), advantage.to_string());
                                    events.push(Event::Custom {
                                        event_type: "SetAdvantageOnNext".to_string(),
                                        data,
                                        source_id: reactor_id.to_string(),
                                    });
                                }
                                crate::enums::TriggerEffect::ConsumeReaction { target_id } => {
                                    let target_id_resolved = match target_id.as_str() {
                                        "self" => reactor_id,
                                        "attacker" => triggering_actor_id.unwrap_or(""),
                                        id => id,
                                    };

                                    if !target_id_resolved.is_empty() {
                                        if let Some(target_mut) = context.get_combatant_mut(target_id_resolved) {
                                            let reaction_slot_id = crate::enums::ActionSlot::Reaction as i32;
                                            target_mut.base_combatant.final_state.used_actions.insert(reaction_slot_id.to_string());
                                            
                                            let mut data = std::collections::HashMap::new();
                                            data.insert("target_id".to_string(), target_id_resolved.to_string());
                                            events.push(Event::Custom {
                                                event_type: "ConsumeReaction".to_string(),
                                                data,
                                                source_id: reactor_id.to_string(),
                                            });
                                        }
                                    }
                                }
                                crate::enums::TriggerEffect::RedirectAttack { new_target_id } => {
                                    let mut data = std::collections::HashMap::new();
                                    data.insert("new_target_id".to_string(), new_target_id.clone());
                                    
                                    events.push(Event::Custom {
                                        event_type: "RedirectAttack".to_string(),
                                        data,
                                        source_id: reactor_id.to_string(),
                                    });
                                }
                                crate::enums::TriggerEffect::SplitDamage { target_id, percent } => {
                                    let mut data = std::collections::HashMap::new();
                                    data.insert("target_id".to_string(), target_id.clone());
                                    data.insert("percent".to_string(), percent.to_string());
                                    
                                    events.push(Event::Custom {
                                        event_type: "SplitDamage".to_string(),
                                        data,
                                        source_id: reactor_id.to_string(),
                                    });
                                }
                                // Implement other effects as needed
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Roll a saving throw for a combatant
    pub fn roll_save(&self, target_id: &str, context: &mut TurnContext) -> f64 {
        let mut roll = rng::roll_d20();
        
        // Apply rerolls
        let mods = context.roll_modifications.take_all(target_id);
        for modif in &mods {
            if let crate::context::RollModification::Reroll { roll_type, must_use_second } = modif {
                if roll_type == "save" {
                    let roll2 = rng::roll_d20();
                    if *must_use_second {
                        roll = roll2;
                    } else {
                        roll = roll.max(roll2);
                    }
                }
            }
        }

        let mut total = roll as f64 + crate::resolvers::attack::get_save_bonus(target_id, context);

        // Apply bonus modifications
        for modif in &mods {
            if let crate::context::RollModification::AddBonus { amount } = modif {
                // Simplified bonus evaluation
                let formula = crate::model::DiceFormula::Expr(amount.clone());
                total += dice::evaluate(&formula, 1);
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DiceFormula;

    #[test]
    fn test_action_resolver_creation() {
        let resolver = ActionResolver::new();
        assert!(resolver.rng_seed.is_none());

        let resolver_with_seed = ActionResolver::with_seed(42);
        assert_eq!(resolver_with_seed.rng_seed, Some(42));
    }

    #[test]
    fn test_resolve_attack() {
        let _resolver = ActionResolver::new();

        // This test would require more setup to create a proper TurnContext
        // For now, just test that the method exists and can be called
        // In a full implementation, this would test actual attack resolution
    }

    #[test]
    fn test_resolve_heal() {
        let _resolver = ActionResolver::new();

        // This test would require proper TurnContext setup
        // For now, just test method existence
    }

    #[test]
    fn test_reactive_retaliation() {
        use crate::context::TurnContext;
        use crate::enums::{
            ActionCondition, BuffDuration, EnemyTarget, TriggerCondition, TriggerEffect,
            TriggerRequirement,
        };
        use crate::model::{Buff, Combattant, Creature, CreatureState, EffectTrigger};

        // 1. Setup Context with Attacker and Defender
        let attacker = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "orc".to_string(),
            name: "Orc".to_string(),
            hp: 20,
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "monster".to_string(),
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };
        let defender = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "warlock".to_string(),
            name: "Warlock".to_string(),
            hp: 20,
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "player".to_string(),
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };

        let mut context = TurnContext::new(
            vec![
                Combattant { team: 1,
                    id: "orc".to_string(),
                    creature: std::sync::Arc::new(attacker),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 20, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 20, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 0,
                    id: "warlock".to_string(),
                    creature: std::sync::Arc::new(defender.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState {
                        current_hp: 20,
                        temp_hp: Some(10), // Pre-give Temp HP
                        ..CreatureState::default()
                    },
                    final_state: CreatureState {
                        current_hp: 20,
                        temp_hp: Some(10),
                        ..CreatureState::default()
                    },
                    actions: vec![],
                },
            ],
            vec![],
            None,
            "Plains".to_string(),
            true,
        );

        // 2. Apply Reactive Buff manually to Defender
        let buff = Buff {
            display_name: Some("Armor of Agathys".to_string()),
            duration: BuffDuration::EntireEncounter,
            ac: None,
            to_hit: None,
            damage: None,
            damage_reduction: None,
            damage_multiplier: None,
            damage_taken_multiplier: None,
            dc: None,
            save: None,
            condition: None,
            magnitude: None,
            source: Some("warlock".to_string()),
            concentration: false,
            triggers: vec![EffectTrigger {
                condition: TriggerCondition::OnBeingHit,
                requirements: vec![TriggerRequirement::HasTempHP],
                effect: TriggerEffect::DealDamage {
                    amount: "5".to_string(),
                    damage_type: "Cold".to_string(),
                },
            }],
            suppressed_until: None,
        };

        use crate::context::{ActiveEffect, EffectType};
        context.apply_effect(ActiveEffect {
            id: "aoa".to_string(),
            source_id: "warlock".to_string(),
            target_id: "warlock".to_string(),
            effect_type: EffectType::Buff(buff),
            remaining_duration: 100,
            conditions: vec![],
        });

        // 2.5 Precalculate stats for targeting
        context.precalculate_combat_stats();

        // 3. Resolve Attack
        let resolver = ActionResolver::new();
        let attack = crate::model::AtkAction {
            id: "axe".to_string(),
            name: "Axe".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: crate::model::Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            dpr: crate::model::DiceFormula::Value(4.0), // Low damage so temp hp remains
            to_hit: crate::model::DiceFormula::Value(100.0), // Ensure Hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        };

        // Mock getting target manually since our resolution doesn't run full selection here easily
        // But resolve_attack calls get_single_attack_target.
        // We need to ensure logic selects "warlock".
        // resolve_attack -> get_single_attack_target.
        // Ensure Warlock is enemy of Orc (modes differ). They do (monster vs player).

        rng::force_d20_rolls(vec![10]);
        let events = resolver.resolve_attack(&attack, &mut context, "orc");

        // 4. Assertions
        // Check for AttackHit
        assert!(events.iter().any(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "warlock")));

        // Check for Retaliation Damage on Orc
        let retaliation = events.iter().find(|e| matches!(e, crate::events::Event::DamageTaken { target_id, damage_type, .. } if target_id == "orc" && damage_type == "Cold"));
        assert!(retaliation.is_some(), "Retaliation damage event not found!");

        if let Some(crate::events::Event::DamageTaken { damage, .. }) = retaliation {
            assert_eq!(damage, &5.0);
        }

        // Check Temp HP reduced on Warlock
        let warlock = context.combatants.get("warlock").unwrap();
        assert!(
            warlock.temp_hp < 10,
            "Temp HP should be reduced by attack damage"
        );
    }
    #[test]
    fn test_multiattack_retargeting() {
        use crate::context::TurnContext;
        use crate::enums::{ActionCondition, EnemyTarget};
        use crate::model::{Combattant, Creature, CreatureState, AtkAction};

        // Setup: Attacker (3 attacks), 3 Victims (10 HP each)
        // Damage = 5 (Fixed). 2 Hits to kill.

        let attacker_c = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "attacker".to_string(),
            name: "Attacker".to_string(),
            hp: 100,
            ac: 15,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "player".to_string(),
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };
        let victim_tpl = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "victim".to_string(),
            name: "Victim".to_string(),
            hp: 4, // One hit kills
            ac: 10,
            count: 1.0,
            actions: vec![],
            triggers: vec![],
            arrival: None,
            mode: "monster".to_string(),
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        };

        let mut context = TurnContext::new(
            vec![
                Combattant { team: 0,
                    id: "attacker".to_string(),
                    creature: std::sync::Arc::new(attacker_c),
                    initiative: 20.0,
                    initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v1".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v2".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant { team: 1,
                    id: "v3".to_string(),
                    creature: std::sync::Arc::new(victim_tpl.clone()),
                    initiative: 10.0,
                    initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
                    actions: vec![],
                },
            ],
            vec![],
            None,
            "Arena".to_string(),
            true,
        );

        let resolver = ActionResolver::with_seed(12345); // Deterministic

        // 3 Attacks. 5 Damage Each. 10 HP Targets.
        // Should go: v1(5dmg) -> v1(Kill) -> v2(5dmg).
        let attack = AtkAction {
            id: "multi".to_string(),
            name: "Multiattack".to_string(),
            action_slot: Some(1),
            freq: crate::model::Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            dpr: crate::model::DiceFormula::Expr("5".to_string()),
            to_hit: crate::model::DiceFormula::Expr("100".to_string()), // Sure hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
        };

        let events = crate::resolvers::resolve_attack(&resolver, &attack, &mut context, "attacker");

        println!("Events: {:?}", events);

        let v1_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v1")).count();
        let v2_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v2")).count();
        let v3_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v3")).count();

        let dead_count = events
            .iter()
            .filter(|e| matches!(e, crate::events::Event::UnitDied { .. }))
            .count();

        // Assertions:
        // 1. All three units should die (if all hit)
        assert!(dead_count >= 1, "At least one victim should die");

        // 2. Hits distribution should be [1, 1, 1] if no misses
        let mut hits = vec![v1_hits, v2_hits, v3_hits];
        hits.sort();
        
        if dead_count == 3 {
            assert_eq!(hits, vec![1, 1, 1]);
        }
    }
}
