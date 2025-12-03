use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::events::Event;
use crate::context::TurnContext;
use crate::model::{Action, AtkAction, HealAction, BuffAction, DebuffAction, TemplateAction};
use crate::dice;

/// Event-driven action resolver that converts actions into events
#[derive(Debug, Clone)]
pub struct ActionResolver {
    /// Random number generator for dice rolls
    #[allow(dead_code)]
    rng_seed: Option<u64>,
}

/// Result of action resolution containing all generated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResolutionResult {
    pub events: Vec<Event>,
    pub success: bool,
    pub error: Option<String>,
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

        // Find targets for this attack
        let targets = self.get_attack_targets(attack, context, actor_id);

        for target_id in targets {
            // Check if target is still alive
            if !context.is_combatant_alive(&target_id) {
                events.push(Event::AttackMissed {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                });
                continue;
            }

            // Perform attack roll
            let attack_roll = self.roll_attack(attack);
            let target_ac = self.get_target_ac(&target_id, context);

            if attack_roll >= target_ac {
                // Hit!
                let damage = self.calculate_damage(attack);

                events.push(Event::AttackHit {
                    attacker_id: actor_id.to_string(),
                    target_id: target_id.clone(),
                    damage,
                });

                events.push(Event::DamageTaken {
                    target_id: target_id.clone(),
                    damage,
                    damage_type: "Physical".to_string(), // Would be determined from attack in full implementation
                });

                // Apply damage to combatant state
                self.apply_damage_to_combatant(&target_id, damage, context);
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
        let mut events = Vec::new();

        // Get targets for healing
        let targets = self.get_heal_targets(heal, context, actor_id);

        let heal_amount = dice::average(&heal.amount);
        let is_temp_hp = heal.temp_hp.unwrap_or(false);

        for target_id in targets {
            if is_temp_hp {
                events.push(Event::TempHPGranted {
                    target_id: target_id.clone(),
                    amount: heal_amount,
                    source_id: actor_id.to_string(),
                });
            } else {
                events.push(Event::HealingApplied {
                    target_id: target_id.clone(),
                    amount: heal_amount,
                    source_id: actor_id.to_string(),
                });
            }

            // Apply healing to combatant state
            self.apply_healing_to_combatant(&target_id, heal_amount, is_temp_hp, context);
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

    /// Apply damage to a combatant with proper event emission
    pub fn apply_damage(&self, target_id: &str, damage: f64, damage_type: &str, source_id: &str, context: &mut TurnContext) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(combatant) = context.get_combatant_mut(target_id) {
            let actual_damage = damage;
            combatant.current_hp = (combatant.current_hp - actual_damage).max(0.0);

            events.push(Event::DamageTaken {
                target_id: target_id.to_string(),
                damage: actual_damage,
                damage_type: damage_type.to_string(),
            });

            if combatant.current_hp <= 0.0 {
                events.push(Event::UnitDied {
                    unit_id: target_id.to_string(),
                    killer_id: Some(source_id.to_string()),
                    damage_type: Some(damage_type.to_string()),
                });
            }
        }

        events
    }

    /// Apply healing to a combatant with proper event emission
    pub fn apply_healing(&self, target_id: &str, amount: f64, source_id: &str, context: &mut TurnContext) -> Event {
        if let Some(combatant) = context.get_combatant_mut(target_id) {
            let max_hp = combatant.base_combatant.creature.hp;
            let actual_healing = (combatant.current_hp + amount).min(max_hp) - combatant.current_hp;
            combatant.current_hp += actual_healing;

            Event::HealingApplied {
                target_id: target_id.to_string(),
                amount: actual_healing,
                source_id: source_id.to_string(),
            }
        } else {
            Event::HealingApplied {
                target_id: target_id.to_string(),
                amount: 0.0,
                source_id: source_id.to_string(),
            }
        }
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
    fn roll_attack(&self, attack: &AtkAction) -> f64 {
        let base_roll = dice::average(&attack.to_hit);
        // In a full implementation, this would add d20 roll and bonuses
        base_roll + 10.0 // Simplified - assume average roll of 10 + d20
    }

    /// Get target's armor class
    fn get_target_ac(&self, target_id: &str, context: &TurnContext) -> f64 {
        context.get_combatant(target_id)
            .map(|c| c.base_combatant.creature.ac)
            .unwrap_or(10.0) // Default AC if not found
    }

    /// Calculate damage from attack
    fn calculate_damage(&self, attack: &AtkAction) -> f64 {
        dice::average(&attack.dpr)
    }

    /// Apply damage directly to combatant state
    fn apply_damage_to_combatant(&self, target_id: &str, damage: f64, context: &mut TurnContext) {
        if let Some(combatant) = context.get_combatant_mut(target_id) {
            combatant.current_hp = (combatant.current_hp - damage).max(0.0);
        }
    }

    /// Apply healing directly to combatant state
    fn apply_healing_to_combatant(&self, target_id: &str, amount: f64, is_temp_hp: bool, context: &mut TurnContext) {
        if let Some(combatant) = context.get_combatant_mut(target_id) {
            if is_temp_hp {
                combatant.temp_hp += amount;
            } else {
                let max_hp = combatant.base_combatant.creature.hp;
                combatant.current_hp = (combatant.current_hp + amount).min(max_hp);
            }
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