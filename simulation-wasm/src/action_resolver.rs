use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::events::Event;
use crate::context::TurnContext;
use crate::model::{Action, AtkAction, HealAction, BuffAction, DebuffAction, TemplateAction};
use crate::dice;
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
    pub fn resolve_attack(&self, attack: &AtkAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();

        // Get targets
        let targets = self.get_attack_targets(attack, context, actor_id);

        for target_id in targets {
            // Check if target is valid
            if !context.is_combatant_alive(&target_id) {
                continue;
            }

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

                events.push(Event::AttackHit {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                    damage,
                });

                // Apply damage through TurnContext (unified method) - handles event emission
                let _damage_events = context.apply_damage(&target_id, damage, "Physical", actor_id);
            } else {
                // Miss!
                events.push(Event::AttackMissed {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                });
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

            if is_buff {
                events.push(Event::BuffApplied {
                    target_id: target_id.clone(),
                    buff_id: effect_id.clone(),
                    source_id: actor_id.to_string(),
                });
            } else {
                // For debuffs, convert to condition
                events.push(Event::ConditionAdded {
                    target_id: target_id.clone(),
                    condition: crate::enums::CreatureCondition::Incapacitated, // Placeholder
                    source_id: actor_id.to_string(),
                });
            }

            // Apply effect through the context's effect system
            self.apply_effect_to_combatant(&target_id, &effect_id, actor_id, is_buff, context);
        }

        events
    }

    /// Resolve buff actions
    pub fn resolve_buff(&self, buff_action: &BuffAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        self.resolve_effect(buff_action, context, actor_id, true)
    }

    /// Resolve debuff actions
    pub fn resolve_debuff(&self, debuff_action: &DebuffAction, context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        self.resolve_effect(debuff_action, context, actor_id, false)
    }

    /// Resolve template actions
    pub fn resolve_template(&self, template_action: &TemplateAction, _context: &mut TurnContext, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();

        // Placeholder implementation for template actions
        // In a full implementation, this would resolve the template and execute the resulting actions

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

    
    /// Get targets for an attack action
    fn get_attack_targets(&self, _attack: &AtkAction, context: &TurnContext, actor_id: &str) -> Vec<String> {
        // Simple implementation: find enemies (other alive combatants)
        context.combatants.keys()
            .filter(|id| *id != actor_id && context.is_combatant_alive(id))
            .take(1) // Default to 1 target for simplicity
            .cloned()
            .collect()
    }

    /// Get targets for a heal action
    fn get_heal_targets(&self, _heal: &HealAction, context: &TurnContext, _actor_id: &str) -> Vec<String> {
        // Simple implementation: find injured allies (including self)
        context.combatants.values()
            .filter(|c| c.current_hp < c.base_combatant.creature.hp)
            .filter(|c| context.is_combatant_alive(&c.id))
            .take(1) // Default to 1 target for simplicity
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
            EffectType::Buff(effect_id.to_string())
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
    fn get_simple_targets(&self, context: &mut TurnContext, _actor_id: &str) -> Vec<String> {
        // Simple implementation: return alive combatants
        context.combatants.keys()
            .filter(|id| context.is_combatant_alive(id))
            .take(self.targets.max(1) as usize)
            .cloned()
            .collect()
    }
}

impl SimpleTargeting for DebuffAction {
    fn get_simple_targets(&self, context: &mut TurnContext, actor_id: &str) -> Vec<String> {
        // Simple implementation: return enemies (other combatants)
        context.combatants.keys()
            .filter(|id| *id != actor_id && context.is_combatant_alive(id))
            .take(self.targets.max(1) as usize)
            .cloned()
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
}