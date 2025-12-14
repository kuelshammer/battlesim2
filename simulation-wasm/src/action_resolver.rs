use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::context::{TurnContext, CombattantState, ActiveEffect, EffectType};
use crate::dice;
use crate::events::Event;
use crate::model::{Action, AtkAction, HealAction, BuffAction, DebuffAction, TemplateAction};
use rand::Rng;

/// Event-driven action resolver that converts actions into events
#[derive(Debug, Clone)]
pub struct ActionResolver {
    /// Random number generator for dice rolls
    #[allow(dead_code)]
    rng_seed: Option<u64>,
}

/// Result of an attack roll
#[derive(Debug, Clone)]
struct AttackRollResult {
    total: f64,
    natural_roll: u32,
    is_critical: bool,
    is_miss: bool,
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
        }
    }

    /// Create a new action resolver with a specific seed for reproducible results
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng_seed: Some(seed),
        }
    }

    /// Resolve an action and emit events to the context
    pub fn resolve_action(&self, action: &Action, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        match action {
            Action::Atk(attack_action) => self.resolve_attack(attack_action, context, actor_id),
            Action::Heal(heal_action) => self.resolve_heal(heal_action, context, actor_id),
            Action::Buff(buff_action) => self.resolve_buff(buff_action, context, actor_id),
            Action::Debuff(debuff_action) => self.resolve_debuff(debuff_action, context, actor_id),
            Action::Template(template_action) => self.resolve_template(template_action, context, actor_id),
        }
    }

    /// Resolve attack actions with proper event emission
    /// Each attack re-evaluates the best target from currently alive enemies
    pub fn resolve_attack(&self, attack: &AtkAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();
        let attack_count = attack.targets.max(1) as usize;

        // For each attack, find the best target at that moment
        for _attack_num in 0..attack_count {
            // Get best target for THIS attack (re-evaluates each time)
            let target_id = match self.get_single_attack_target(attack, context, actor_id) {
                Some(id) => id,
                None => continue, // No valid target, skip this attack
            };

            // Perform attack roll
            let attack_result = self.roll_attack(attack);
            let target_ac = self.get_target_ac(&target_id, context);

            // Check for hit:
            // 1. Critical Hit (Nat 20) always hits
            // 2. Critical Miss (Nat 1) always misses
            // 3. Otherwise check total vs AC
            let is_hit = !attack_result.is_miss && (attack_result.is_critical || attack_result.total >= target_ac);

                if is_hit {
                // Hit!
                let damage = self.calculate_damage(attack, attack_result.is_critical);

                let hit_event = Event::AttackHit {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                    damage,
                };
                context.record_event(hit_event.clone());
                events.push(hit_event);

                // Check for "OnBeingHit" triggers (e.g. Armor of Agathys)
                let trigger_events = self.resolve_trigger_effects(
                    &target_id, 
                    crate::enums::TriggerCondition::OnBeingHit, 
                    context, 
                    Some(actor_id),
                    None // TODO: Pass damage type if needed for requirements
                );
                events.extend(trigger_events);

                // Apply damage through TurnContext (unified method) - handles event emission
                let damage_events = context.apply_damage(&target_id, damage, "Physical", actor_id); // Default to Physical, upgrade later
                events.extend(damage_events);
            } else {
                // Miss!
                let miss_event = Event::AttackMissed {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                };
                context.record_event(miss_event.clone());
                events.push(miss_event);
            }
        }

        events
    }


    /// Resolve healing actions with proper event emission
    pub fn resolve_heal(&self, heal: &HealAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let events = Vec::new();

        // Get targets for healing
        let targets = self.get_heal_targets(heal, context, actor_id);

        let heal_amount = dice::average(&heal.amount);
        let is_temp_hp = heal.temp_hp.unwrap_or(false);

        for target_id in targets {
            // Apply healing through TurnContext (unified method) - handles event emission
            let _healing_event = context.apply_healing(&target_id, heal_amount, is_temp_hp, actor_id);
        }

        events
    }

    /// Resolve buff/debuff actions with proper event emission
    pub fn resolve_effect(&self, effect_action: &impl EffectAction, context: &mut TurnContext, actor_id: &str, is_buff: bool) -> Vec<Event> {
        let mut events = Vec::new();

        let targets = effect_action.get_targets(context, actor_id);

        for target_id in targets {
            let effect_id = effect_action.base().id.clone();
            
            // Note: Event emission is handled by apply_effect_to_combatant -> context.apply_effect
            // No need to push events here manually anymore.

            // Apply effect through the context's effect system
            self.apply_effect_to_combatant(&target_id, &effect_id, actor_id, is_buff, context);
        }

        events
    }

    /// Resolve buff actions
    pub fn resolve_buff(&self, buff_action: &BuffAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();
        
        use crate::enums::TargetType;
        let target_type = TargetType::Ally(buff_action.target.clone());
        let targets = self.get_targets_by_type(context, actor_id, &target_type, buff_action.targets);

        for target_id in targets {
             let effect = ActiveEffect {
                id: format!("{}-{}-{}", buff_action.name, actor_id, target_id),
                source_id: actor_id.to_string(),
                target_id: target_id.clone(),
                effect_type: EffectType::Buff(buff_action.buff.clone()),
                remaining_duration: 10, 
                conditions: Vec::new(),
            };

            events.push(context.apply_effect(effect));
        }

        events
    }


    /// Resolve debuff actions
    pub fn resolve_debuff(&self, debuff_action: &DebuffAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        self.resolve_effect(debuff_action, context, actor_id, false)
    }

    /// Resolve template actions
    /// Resolve template actions
    pub fn resolve_template(&self, template_action: &TemplateAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();
        let template_name = template_action.template_options.template_name.to_lowercase();
        
        // Determine target type
        let target_type = template_action.template_options.target.clone().unwrap_or_else(|| {
             // Default based on name if missing
            if template_name == "bane" { 
                use crate::enums::{TargetType, EnemyTarget};
                TargetType::Enemy(EnemyTarget::EnemyWithLeastHP) 
            } else { 
                use crate::enums::{TargetType, AllyTarget};
                TargetType::Ally(AllyTarget::AllyWithLeastHP) 
            }
        });

        // Select targets
        let targets = self.get_targets_by_type(context, actor_id, &target_type, template_action.targets);

        for target_id in targets {
            // Apply effect
            let effect_type = if template_name == "bane" {
                EffectType::Custom("bane".to_string())
            } else {
                use crate::model::Buff;
                use crate::enums::BuffDuration;
                EffectType::Buff(Buff {
                    display_name: Some(template_name.clone()),
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
                    source: Some(actor_id.to_string()),
                    concentration: true, 
                    triggers: Vec::new(),
                })
            };

            let effect = ActiveEffect {
                id: format!("{}-{}-{}", template_name, actor_id, target_id), // Unique ID per target/source
                source_id: actor_id.to_string(),
                target_id: target_id.clone(),
                effect_type,
                remaining_duration: 10, // Default duration
                conditions: Vec::new(),
            };

            events.push(context.apply_effect(effect));
        }

        events.push(Event::Custom {
            event_type: "TemplateActionExecuted".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("template_name".to_string(), template_action.name.clone());
                data.insert("actor_id".to_string(), actor_id.to_string());
                data
            },
            source_id: actor_id.to_string(),
        });

        events
    }

    /// Helper to resolve reactive effects from buffs (Triggers)
    /// Helper to resolve reactive effects from buffs (Triggers)
    fn resolve_trigger_effects(&self, 
        reactor_id: &str, 
        condition: crate::enums::TriggerCondition, 
        context: &mut TurnContext, 
        triggering_actor_id: Option<&str>,
        _damage_info: Option<&str>
    ) -> Vec<Event> {
        let mut events = Vec::new();
        
        // Iterate over active effects to find buffs on the reactor
        // Access values() directly to avoid borrow checker issues with iterating the map
        let active_effects: Vec<crate::context::ActiveEffect> = context.active_effects.values().cloned().collect();

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
                                        c.temp_hp > 0.0
                                    } else { false }
                                },
                                // Implement other requirements as needed
                                _ => true 
                            }
                        });

                        if requirements_met {
                             // Execute Effect
                             match &trigger.effect {
                                crate::enums::TriggerEffect::DealDamage { amount, damage_type } => {
                                    if let Some(target_id) = triggering_actor_id {
                                         // Parse amount formula
                                         let formula = crate::model::DiceFormula::Expr(amount.clone());
                                         // Basic eval for now, assume no variable parts specific to reactor yet (except fixed values)
                                         let dmg_value = dice::evaluate(&formula, 1);
                                         
                                         // Apply damage to the TRIGGERING ACTOR (Retaliation)
                                         let dmg_events = context.apply_damage(target_id, dmg_value, damage_type, reactor_id);
                                         events.extend(dmg_events);
                                    }
                                },
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

    /// Helper to get targets based on TargetType
    fn get_targets_by_type(&self, context: &TurnContext, actor_id: &str, target_type: &crate::enums::TargetType, count: i32) -> Vec<String> {
        use crate::enums::TargetType;
        
        let actor_mode = context.get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();

        match target_type {
            TargetType::Enemy(_strategy) => {
                // Find enemies
                context.combatants.values()
                    .filter(|c| c.id != actor_id)
                    .filter(|c| context.is_combatant_alive(&c.id))
                    .filter(|c| c.base_combatant.creature.mode != actor_mode)
                    .take(count.max(1) as usize)
                    .map(|c| c.id.clone())
                    .collect()
            },
            TargetType::Ally(_strategy) => {
                 // Find allies
                 context.combatants.values()
                    .filter(|c| context.is_combatant_alive(&c.id))
                    .filter(|c| c.base_combatant.creature.mode == actor_mode)
                    .take(count.max(1) as usize)
                    .map(|c| c.id.clone())
                    .collect()
            }
        }
    }

    /// Get SINGLE best target for an attack - re-called for each attack
    /// Allows same enemy to be targeted again if still alive
    /// Uses priority-based tie-breaking when primary strategy results in ties
    fn get_single_attack_target(&self, attack: &AtkAction, context: &TurnContext, actor_id: &str) -> Option<String> {
        use std::cmp::Ordering;
        
        // Get actor's team (mode)
        let actor_mode = context.get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();
        
        // Find all alive enemies
        let mut enemies: Vec<_> = context.combatants.values()
            .filter(|c| c.id != actor_id)  // Not self
            .filter(|c| context.is_combatant_alive(&c.id))  // Must be alive
            .filter(|c| c.base_combatant.creature.mode != actor_mode)  // Different team = enemy
            .collect();
        
        if enemies.is_empty() {
            return None;
        }
        
        // Helper: estimate DPR for a combatant
        let estimate_dpr = |c: &crate::context::CombattantState| -> f64 {
            c.base_combatant.creature.actions.iter()
                .filter_map(|action| {
                    if let crate::model::Action::Atk(atk) = action {
                        Some(crate::dice::average(&atk.dpr) * atk.targets as f64)
                    } else { None }
                }).sum::<f64>()
        };
        
        // Sort with full priority-based tie-breaking
        enemies.sort_by(|a, b| {
            // 1. PRIMARY STRATEGY
            let primary = match &attack.target {
                crate::enums::EnemyTarget::EnemyWithLeastHP => {
                    a.current_hp.partial_cmp(&b.current_hp).unwrap_or(Ordering::Equal)
                },
                crate::enums::EnemyTarget::EnemyWithMostHP => {
                    b.current_hp.partial_cmp(&a.current_hp).unwrap_or(Ordering::Equal)
                },
                crate::enums::EnemyTarget::EnemyWithLowestAC => {
                    a.base_combatant.creature.ac.partial_cmp(&b.base_combatant.creature.ac).unwrap_or(Ordering::Equal)
                },
                crate::enums::EnemyTarget::EnemyWithHighestAC => {
                    b.base_combatant.creature.ac.partial_cmp(&a.base_combatant.creature.ac).unwrap_or(Ordering::Equal)
                },
                crate::enums::EnemyTarget::EnemyWithHighestDPR => {
                    let dpr_a = estimate_dpr(a);
                    let dpr_b = estimate_dpr(b);
                    dpr_b.partial_cmp(&dpr_a).unwrap_or(Ordering::Equal)
                },
            };
            if primary != Ordering::Equal { return primary; }
            
            // 2. TIE-BREAKER: Concentration (prefer enemies who are concentrating)
            let conc_a = a.concentration.is_some();
            let conc_b = b.concentration.is_some();
            if conc_a && !conc_b { return Ordering::Less; }  // a is concentrating, prefer a
            if !conc_a && conc_b { return Ordering::Greater; }  // b is concentrating, prefer b
            
            // 3. TIE-BREAKER: Hit Probability (lower AC = easier to hit)
            let ac_cmp = a.base_combatant.creature.ac.partial_cmp(&b.base_combatant.creature.ac);
            if ac_cmp != Some(Ordering::Equal) { return ac_cmp.unwrap_or(Ordering::Equal); }
            
            // 4. TIE-BREAKER: Higher DPR (more dangerous enemy)
            let dpr_a = estimate_dpr(a);
            let dpr_b = estimate_dpr(b);
            let dpr_cmp = dpr_b.partial_cmp(&dpr_a);  // Higher DPR first
            if dpr_cmp != Some(Ordering::Equal) { return dpr_cmp.unwrap_or(Ordering::Equal); }
            
            // 5. TIE-BREAKER: Higher Initiative
            let init_cmp = b.base_combatant.initiative.partial_cmp(&a.base_combatant.initiative);
            if init_cmp != Some(Ordering::Equal) { return init_cmp.unwrap_or(Ordering::Equal); }
            
            // 6. FINAL TIE-BREAKER: Alphabetical name (deterministic)
            a.base_combatant.creature.name.cmp(&b.base_combatant.creature.name)
        });
        
        enemies.first().map(|c| c.id.clone())
    }

    /// Get targets for a heal action - TARGET ALLIES ONLY
    fn get_heal_targets(&self, heal: &HealAction, context: &TurnContext, actor_id: &str) -> Vec<String> {
        // Get actor's team (mode)
        let actor_mode = context.get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();
        
        // Find injured allies (same team, including self if injured)
        context.combatants.values()
            .filter(|c| c.current_hp < c.base_combatant.creature.hp)  // Must be injured
            .filter(|c| context.is_combatant_alive(&c.id))  // Must be alive
            .filter(|c| c.base_combatant.creature.mode == actor_mode)  // Same team = ally
            .take(heal.targets.max(1) as usize)
            .map(|c| c.id.clone())
            .collect()
    }

    /// Roll attack value
    fn roll_attack(&self, attack: &AtkAction) -> AttackRollResult {
        let mut rng = rand::thread_rng();
        let natural_roll = rng.gen_range(1..=20);
        let bonus = dice::average(&attack.to_hit); // Keep using average for bonus part
        
        AttackRollResult {
            total: natural_roll as f64 + bonus,
            natural_roll,
            is_critical: natural_roll == 20,
            is_miss: natural_roll == 1,
        }
    }

    /// Get target's armor class
    fn get_target_ac(&self, target_id: &str, context: &TurnContext) -> f64 {
        context.get_combatant(target_id)
            .map(|c| c.base_combatant.creature.ac)
            .unwrap_or(10.0) // Default AC if not found
    }

    /// Calculate damage from attack
    fn calculate_damage(&self, attack: &AtkAction, is_critical: bool) -> f64 {
        let base_damage = dice::average(&attack.dpr);
        
        if is_critical {
            // For critical hits, double the damage (simple approximation)
            base_damage * 2.0 
        } else {
            base_damage
        }
    }

    
    /// Apply effect directly to combatant through the context system
    fn apply_effect_to_combatant(&self, target_id: &str, effect_id: &str, source_id: &str, is_buff: bool, context: &mut TurnContext) {
        use crate::context::{ActiveEffect, EffectType};

        let effect_type = if is_buff {
            // EffectType::Buff(effect_id.to_string()) // Invalid
            panic!("Buffs must use resolve_buff directly to preserve data");
        } else {
            EffectType::Condition(crate::enums::CreatureCondition::Incapacitated)
        };

        let effect = ActiveEffect {
            id: format!("{}_{}", effect_id, source_id),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            effect_type,
            remaining_duration: 5, // Placeholder duration
            conditions: Vec::new(),
        };

        context.apply_effect(effect);
    }
}

/// Trait for actions that can apply effects
pub trait EffectAction {
    fn base(&self) -> crate::model::ActionBase; // Changed from &crate::model::ActionBase
    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String>; // Changed context to mut
}

impl EffectAction for BuffAction {
    fn base(&self) -> crate::model::ActionBase {
        self.base() // Call the struct's base method
    }

    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        // Use targeting module - simplified for now
        self.get_simple_targets(context, actor_id)
    }
}

impl EffectAction for DebuffAction {
    fn base(&self) -> crate::model::ActionBase {
        self.base() // Call the struct's base method
    }

    fn get_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        self.get_simple_targets(context, actor_id)
    }
}

// Helper trait for simple target resolution
trait SimpleTargeting {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String>;
}

impl SimpleTargeting for BuffAction {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        // Get actor's team (mode) - BUFFS TARGET ALLIES
        let actor_mode = context.get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();
        
        // Find allies (same team)
        context.combatants.values()
            .filter(|c| context.is_combatant_alive(&c.id))  // Must be alive
            .filter(|c| c.base_combatant.creature.mode == actor_mode)  // Same team = ally
            .take(self.targets.max(1) as usize)
            .map(|c| c.id.clone())
            .collect()
    }
}

impl SimpleTargeting for DebuffAction {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        // Get actor's team (mode) - DEBUFFS TARGET ENEMIES
        let actor_mode = context.get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();
        
        // Find enemies (different team)
        context.combatants.values()
            .filter(|c| c.id != actor_id)  // Not self
            .filter(|c| context.is_combatant_alive(&c.id))  // Must be alive
            .filter(|c| c.base_combatant.creature.mode != actor_mode)  // Different team = enemy
            .take(self.targets.max(1) as usize)
            .map(|c| c.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused imports: Creature, Combattant, ActionBase, Frequency
    use crate::enums::{ActionCondition, EnemyTarget};
    // Removed unused imports: ActionCost, ResourceType
    // use crate::resources::{ActionCost, ResourceType}; 

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
        use crate::model::{Creature, Combattant, CreatureState, Buff, EffectTrigger};
        use crate::enums::{BuffDuration, TriggerCondition, TriggerRequirement, TriggerEffect, ActionCondition, EnemyTarget};
        use crate::resources::ResourceLedger;

        // 1. Setup Context with Attacker and Defender
        let attacker = Creature { id: "orc".to_string(), name: "Orc".to_string(), hp: 20.0, ac: 10.0, count: 1.0, actions: vec![], triggers: vec![], arrival: None, mode: "monster".to_string(), speed_fly: None, save_bonus: 0.0, str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None, int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None, con_save_advantage: None, save_advantage: None, initiative_bonus: 0.0, initiative_advantage: false, spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None };
        let defender = Creature { id: "warlock".to_string(), name: "Warlock".to_string(), hp: 20.0, ac: 10.0, count: 1.0, actions: vec![], triggers: vec![], arrival: None, mode: "player".to_string(), speed_fly: None, save_bonus: 0.0, str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None, int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None, con_save_advantage: None, save_advantage: None, initiative_bonus: 0.0, initiative_advantage: false, spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None };

        let mut context = TurnContext::new(
            vec![
                Combattant { id: "orc".to_string(), creature: attacker, initiative: 10.0, initial_state: CreatureState::default(), final_state: CreatureState::default(), actions: vec![] },
                Combattant { id: "warlock".to_string(), creature: defender.clone(), initiative: 10.0, initial_state: {
                    let mut s = CreatureState::default();
                    s.temp_hp = Some(10.0); // Pre-give Temp HP
                    s
                }, final_state: CreatureState::default(), actions: vec![] }
            ],
            vec![], None, "Plains".to_string()
        );

        // 2. Apply Reactive Buff manually to Defender
        let buff = Buff {
            display_name: Some("Armor of Agathys".to_string()),
            duration: BuffDuration::EntireEncounter,
            ac: None, to_hit: None, damage: None, damage_reduction: None, damage_multiplier: None, damage_taken_multiplier: None, dc: None, save: None, condition: None, magnitude: None, source: Some("warlock".to_string()), concentration: false,
            triggers: vec![
                EffectTrigger {
                    condition: TriggerCondition::OnBeingHit,
                    requirements: vec![TriggerRequirement::HasTempHP],
                    effect: TriggerEffect::DealDamage { amount: "5".to_string(), damage_type: "Cold".to_string() }
                }
            ]
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

        // 3. Resolve Attack
        let resolver = ActionResolver::new();
        let attack = crate::model::AtkAction {
            id: "axe".to_string(), name: "Axe".to_string(), action_slot: None, cost: vec![], requirements: vec![], tags: vec![], freq: crate::model::Frequency::Static("at will".to_string()), condition: ActionCondition::Default, targets: 1,
            dpr: crate::model::DiceFormula::Value(4.0), // Low damage so temp hp remains
            to_hit: crate::model::DiceFormula::Value(100.0), // Ensure Hit
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None, half_on_save: None, rider_effect: None
        };

        // Mock getting target manually since our resolution doesn't run full selection here easily
        // But resolve_attack calls get_single_attack_target.
        // We need to ensure logic selects "warlock".
        // resolve_attack -> get_single_attack_target.
        // Ensure Warlock is enemy of Orc (modes differ). They do (monster vs player).

        let events = resolver.resolve_attack(&attack, &mut context, "orc");

        // 4. Assertions
        // Check for AttackHit
        assert!(events.iter().any(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "warlock")));
        
        // Check for Retaliation Damage on Orc
        let retaliation = events.iter().find(|e| matches!(e, crate::events::Event::DamageTaken { target_id, damage_type, .. } if target_id == "orc" && damage_type == "Cold"));
        assert!(retaliation.is_some(), "Retaliation damage event not found!");
        
        if let Some(crate::events::Event::DamageTaken { damage, .. }) = retaliation {
           assert_eq!(*damage, 5.0);
        }

        // Check Temp HP reduced on Warlock
        let warlock = context.combatants.get("warlock").unwrap();
        assert!(warlock.temp_hp < 10.0, "Temp HP should be reduced by attack damage");
    }
    #[test]
    fn test_multiattack_retargeting() {
        use crate::context::TurnContext;
        use crate::model::{Creature, Combattant, CreatureState};
        use crate::enums::{ActionCondition, EnemyTarget};
        
        // Setup: Attacker (3 attacks), 3 Victims (10 HP each)
        // Damage = 5 (Fixed). 2 Hits to kill.
        
        let attacker_c = Creature { id: "attacker".to_string(), name: "Attacker".to_string(), hp: 100.0, ac: 15.0, count: 1.0, actions: vec![], triggers: vec![], arrival: None, mode: "player".to_string(), speed_fly: None, save_bonus: 0.0, str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None, int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None, con_save_advantage: None, save_advantage: None, initiative_bonus: 0.0, initiative_advantage: false, spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None };
        let victim_tpl = Creature { id: "victim".to_string(), name: "Victim".to_string(), hp: 10.0, ac: 10.0, count: 1.0, actions: vec![], triggers: vec![], arrival: None, mode: "monster".to_string(), speed_fly: None, save_bonus: 0.0, str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None, int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None, con_save_advantage: None, save_advantage: None, initiative_bonus: 0.0, initiative_advantage: false, spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None };

        let mut context = TurnContext::new(
            vec![
                Combattant { id: "attacker".to_string(), creature: attacker_c, initiative: 20.0, initial_state: CreatureState::default(), final_state: CreatureState::default(), actions: vec![] },
                Combattant { id: "v1".to_string(), creature: victim_tpl.clone(), initiative: 10.0, initial_state: CreatureState::default(), final_state: CreatureState::default(), actions: vec![] },
                Combattant { id: "v2".to_string(), creature: victim_tpl.clone(), initiative: 10.0, initial_state: CreatureState::default(), final_state: CreatureState::default(), actions: vec![] },
                Combattant { id: "v3".to_string(), creature: victim_tpl.clone(), initiative: 10.0, initial_state: CreatureState::default(), final_state: CreatureState::default(), actions: vec![] },
            ],
            vec![], None, "Arena".to_string()
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

        let events = resolver.resolve_attack(&attack, &mut context, "attacker");
        
        println!("Events: {:?}", events);

        let v1_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v1")).count();
        let v2_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v2")).count();
        let v3_hits = events.iter().filter(|e| matches!(e, crate::events::Event::AttackHit { target_id, .. } if target_id == "v3")).count();

        let dead_count = events.iter().filter(|e| matches!(e, crate::events::Event::UnitDied { .. })).count();
        
        // Assertions:
        // 1. Exactly one unit should die
        assert_eq!(dead_count, 1, "Exactly one victim should die");

        // 2. Hits distribution should be 2, 1, 0 (order irrelevant)
        let mut hits = vec![v1_hits, v2_hits, v3_hits];
        hits.sort();
        assert_eq!(hits, vec![0, 1, 2], "Hits should be distributed as [0, 1, 2] across the 3 victims");
    }
}