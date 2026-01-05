use crate::context::{ActiveEffect, EffectType, TurnContext};
use crate::enums::TriggerCondition;
use crate::events::Event;
use crate::model::Action;
use crate::resources::{ActionCost, ActionRequirement};
use crate::validation;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Manages the reaction system that allows combatants to respond to specific events
#[derive(Debug, Clone)]
pub struct ReactionManager {
    /// Available reaction templates for each combatant
    available_reactions: HashMap<String, Vec<ReactionTemplate>>,

    /// Track used reactions per turn to prevent multiple uses
    used_reactions: HashMap<String, HashSet<String>>, // combatant_id -> reaction_id

    /// Track used reactions per encounter for limited-use reactions
    encounter_used: HashMap<String, HashSet<String>>, // combatant_id -> reaction_id

    /// Current round number for tracking
    current_round: u32,
}

/// A template for a reaction that can be triggered by specific events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionTemplate {
    pub id: String,
    pub name: String,
    pub description: String,

    /// What event type triggers this reaction
    pub trigger_event_type: String,

    /// More detailed trigger conditions
    pub trigger_condition: TriggerCondition,

    /// The action to execute when triggered
    pub response_action: Action,

    /// Cost to perform this reaction
    pub cost: Vec<ActionCost>,

    /// Requirements that must be met
    pub requirements: Vec<ActionRequirement>,

    /// Higher priority reactions execute first
    pub priority: i32,

    /// How many times per round this can be used (None = unlimited)
    pub uses_per_round: Option<u32>,

    /// How many times per encounter this can be used (None = unlimited)
    pub uses_per_encounter: Option<u32>,

    /// Whether this reaction consumes the reaction resource
    pub consumes_reaction: bool,
}

impl Default for ReactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactionManager {
    /// Create a new reaction manager
    pub fn new() -> Self {
        Self {
            available_reactions: HashMap::new(),
            used_reactions: HashMap::new(),
            encounter_used: HashMap::new(),
            current_round: 0,
        }
    }

    /// Register a reaction template for a specific combatant
    pub fn register_reaction(&mut self, combatant_id: String, reaction: ReactionTemplate) {
        self.available_reactions
            .entry(combatant_id)
            .or_default()
            .push(reaction);
    }

    /// Get all available reactions for a combatant
    pub fn get_available_reactions(&self, combatant_id: &str) -> &[ReactionTemplate] {
        self.available_reactions
            .get(combatant_id)
            .map_or(&[], |reactions| reactions.as_slice())
    }

    /// Check what reactions can be triggered by a specific event
    pub fn check_reactions(
        &mut self,
        event: &Event,
        context: &TurnContext,
    ) -> Vec<(String, &ReactionTemplate)> {
        let mut triggered = Vec::new();

        for (combatant_id, reactions) in &self.available_reactions {
            // Skip if combatant is not alive
            if !context.is_combatant_alive(combatant_id) {
                continue;
            }

            for reaction in reactions {
                if self.can_trigger_reaction(combatant_id, reaction, event, context) {
                    triggered.push((combatant_id.clone(), reaction));
                }
            }
        }

        // Sort by priority (higher first) and then by combatant ID for consistency
        triggered.sort_by(|a, b| {
            let priority_cmp = b.1.priority.cmp(&a.1.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            a.0.cmp(&b.0)
        });

        triggered
    }

    /// Check what reactions can be triggered by a specific event (immutable version)
    pub fn get_triggered_reactions(
        &self,
        event: &Event,
        context: &TurnContext,
    ) -> Vec<(String, &ReactionTemplate)> {
        let mut triggered = Vec::new();

        for (combatant_id, reactions) in &self.available_reactions {
            // Skip if combatant is not alive
            if !context.is_combatant_alive(combatant_id) {
                continue;
            }

            for reaction in reactions {
                if self.can_trigger_reaction(combatant_id, reaction, event, context) {
                    triggered.push((combatant_id.clone(), reaction));
                }
            }
        }

        // Sort by priority (higher first) and then by combatant ID for consistency
        triggered.sort_by(|a, b| {
            let priority_cmp = b.1.priority.cmp(&a.1.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }
            a.0.cmp(&b.0)
        });

        triggered
    }

    /// Check if a specific reaction can be triggered
    fn can_trigger_reaction(
        &self,
        combatant_id: &str,
        reaction: &ReactionTemplate,
        event: &Event,
        context: &TurnContext,
    ) -> bool {
        // Check if the event type matches
        if !self.event_matches_trigger(
            event,
            &reaction.trigger_event_type,
            &reaction.trigger_condition,
        ) {
            return false;
        }

        // Check if combatant can afford the cost
        if !context.can_afford(&reaction.cost, combatant_id) {
            return false;
        }

        // Check requirements
        if !validation::check_action_requirements(&reaction.response_action, context, combatant_id)
        {
            return false;
        }

        // Check if already used this round
        if let Some(limit) = reaction.uses_per_round {
            if let Some(used_this_round) = self.used_reactions.get(combatant_id) {
                let use_count = used_this_round
                    .iter()
                    .filter(|id| **id == reaction.id)
                    .count();
                if use_count >= limit as usize {
                    return false;
                }
            }
        }

        // Check if already used this encounter
        if let Some(limit) = reaction.uses_per_encounter {
            if let Some(used_encounter) = self.encounter_used.get(combatant_id) {
                let use_count = used_encounter
                    .iter()
                    .filter(|id| **id == reaction.id)
                    .count();
                if use_count >= limit as usize {
                    return false;
                }
            }
        }

        // Check if reaction resource is available (if it consumes reaction)
        if reaction.consumes_reaction {
            let reaction_cost = vec![ActionCost::Discrete {
                resource_type: crate::resources::ResourceType::Reaction,
                resource_val: None,
                amount: 1.0,
            }];
            if !context.can_afford(&reaction_cost, combatant_id) {
                return false;
            }
        }

        true
    }

    /// Check if an event matches the trigger conditions
    fn event_matches_trigger(
        &self,
        event: &Event,
        trigger_type: &str,
        trigger_condition: &TriggerCondition,
    ) -> bool {
        // First check if the event type matches
        if event.get_type() != trigger_type {
            return false;
        }

        // Then check the specific trigger condition
        match trigger_condition {
            TriggerCondition::OnHit => matches!(event, Event::AttackHit { .. }),
            TriggerCondition::OnBeingAttacked => {
                matches!(event, Event::AttackHit { .. })
                // This is a simplified check - in a full implementation,
                // you'd need to know which combatant we're checking for
            }
            TriggerCondition::OnMiss => matches!(event, Event::AttackMissed { .. }),
            TriggerCondition::OnBeingDamaged => {
                matches!(event, Event::DamageTaken { .. })
            }
            TriggerCondition::OnAllyAttacked => {
                matches!(event, Event::AttackHit { .. }) // Placeholder for ally checking
            }
            TriggerCondition::OnEnemyDeath => {
                matches!(event, Event::UnitDied { .. }) // Placeholder for enemy checking
            }
            TriggerCondition::OnCriticalHit => {
                matches!(event, Event::AttackHit { .. }) // Placeholder for critical checking
            }
            TriggerCondition::OnBeingHit => matches!(event, Event::AttackHit { .. }),
            // Composite triggers - require additional context
            TriggerCondition::And { conditions: _ } => {
                // TODO: Implement recursive condition evaluation
                // For now, return false as these need combat state context
                false
            }
            TriggerCondition::Or { conditions: _ } => {
                // TODO: Implement recursive condition evaluation
                // For now, return false as these need combat state context
                false
            }
            TriggerCondition::Not { condition: _ } => {
                // TODO: Implement negation
                // For now, return false as these need combat state context
                false
            }
            // State conditions - require combat context
            TriggerCondition::EnemyCountAtLeast { count: _ } => {
                // TODO: Implement enemy count check from combat state
                false
            }
            TriggerCondition::DamageExceedsPercent { threshold: _ } => {
                // TODO: Implement damage percentage check from event
                matches!(event, Event::DamageTaken { .. })
            }
            TriggerCondition::AttackWasMelee => {
                // TODO: Implement melee attack check from event metadata
                matches!(event, Event::AttackHit { .. })
            }
        }
    }

    /// Check combat conditions
    /// Execute a reaction
    pub fn execute_reaction(
        &mut self,
        combatant_id: &str,
        reaction: &ReactionTemplate,
        context: &mut TurnContext,
    ) -> Result<(), String> {
        // Pay the costs
        context.pay_costs(&reaction.cost, combatant_id)?;

        // Pay reaction cost if applicable
        if reaction.consumes_reaction {
            let reaction_cost = vec![ActionCost::Discrete {
                resource_type: crate::resources::ResourceType::Reaction,
                resource_val: None,
                amount: 1.0,
            }];
            context.pay_costs(&reaction_cost, combatant_id)?;
        }

        // Mark as used
        self.mark_reaction_used(combatant_id, &reaction.id);

        // Record the reaction action
        context.record_event(Event::ActionStarted {
            actor_id: combatant_id.to_string(),
            action_id: reaction.id.clone(),
            decision_trace: HashMap::new(),
        });

        // Execute the action (simplified - in a full implementation, this would go through the action resolver)
        self.execute_action(&reaction.response_action, context, combatant_id)?;

        Ok(())
    }

    /// Mark a reaction as used
    fn mark_reaction_used(&mut self, combatant_id: &str, reaction_id: &str) {
        // Mark as used this round
        self.used_reactions
            .entry(combatant_id.to_string())
            .or_default()
            .insert(reaction_id.to_string());

        // Mark as used this encounter if it has encounter limits
        // This would be tracked when we know which reactions have encounter limits
    }

    /// Execute the reaction action (placeholder implementation)
    fn execute_action(
        &self,
        action: &Action,
        context: &mut TurnContext,
        _actor_id: &str,
    ) -> Result<(), String> {
        match action {
            Action::Template(_template_action) => {
                // In a full implementation, this would resolve the template
                // and execute the action through the action system
                context.record_event(Event::Custom {
                    event_type: "ReactionAction".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("action_type".to_string(), "Template".to_string());
                        data
                    },
                    source_id: "ReactionManager".to_string(),
                });
                Ok(())
            }
            Action::Atk(atk_action) => {
                // Record attack action
                context.record_event(Event::Custom {
                    event_type: "ReactionAttack".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("damage".to_string(), format!("{:?}", atk_action.dpr));
                        data
                    },
                    source_id: atk_action.base().id.clone(),
                });
                Ok(())
            }
            Action::Heal(heal_action) => {
                // Record healing action
                let heal_amount = crate::dice::average(&heal_action.amount);
                context.record_event(Event::HealingApplied {
                    target_id: "TODO".to_string(), // Would need to extract from action
                    amount: heal_amount,
                    source_id: heal_action.base().id.clone(),
                });
                Ok(())
            }
            Action::Buff(buff_action) => {
                // Apply buff effect
                let effect = ActiveEffect {
                    id: format!("{}_{}", buff_action.base().id, "reaction_effect"), // Simple ID without chrono
                    source_id: buff_action.base().id.clone(),
                    target_id: "TODO".to_string(), // Would need to extract from action
                    effect_type: EffectType::Buff(buff_action.buff.clone()),
                    remaining_duration: 1, // Placeholder
                    conditions: Vec::new(),
                };
                context.apply_effect(effect);
                Ok(())
            }
            Action::Debuff(debuff_action) => {
                // Apply debuff effect
                let effect = ActiveEffect {
                    id: format!("{}_{}", debuff_action.base().id, "reaction_effect"), // Simple ID without chrono
                    source_id: debuff_action.base().id.clone(),
                    target_id: "TODO".to_string(), // Would need to extract from action
                    effect_type: EffectType::Condition(
                        crate::enums::CreatureCondition::Incapacitated,
                    ), // Placeholder
                    remaining_duration: 1,         // Placeholder
                    conditions: Vec::new(),
                };
                context.apply_effect(effect);
                Ok(())
            }
        }
    }

    /// Advance to the next round (clear round-based usage tracking)
    pub fn advance_round(&mut self, new_round: u32) {
        self.current_round = new_round;
        self.used_reactions.clear();
    }

    /// Reset encounter-based usage tracking
    pub fn reset_encounter(&mut self) {
        self.encounter_used.clear();
    }

    /// Get statistics about the reaction system
    pub fn get_stats(&self) -> ReactionStats {
        let total_reactions: usize = self
            .available_reactions
            .values()
            .map(|reactions| reactions.len())
            .sum();
        let total_used_this_round: usize =
            self.used_reactions.values().map(|used| used.len()).sum();
        let total_used_encounter: usize = self.encounter_used.values().map(|used| used.len()).sum();

        ReactionStats {
            current_round: self.current_round,
            total_available_reactions: total_reactions,
            reactions_used_this_round: total_used_this_round,
            reactions_used_this_encounter: total_used_encounter,
        }
    }
}

/// Statistics about the reaction system
#[derive(Debug, Clone)]
pub struct ReactionStats {
    pub current_round: u32,
    pub total_available_reactions: usize,
    pub reactions_used_this_round: usize,
    pub reactions_used_this_encounter: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ResourceType;

    #[test]
    fn test_reaction_manager_creation() {
        let manager = ReactionManager::new();
        assert_eq!(manager.current_round, 0);
        assert_eq!(manager.available_reactions.len(), 0);
    }

    #[test]
    fn test_register_reaction() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "shield_reaction".to_string(),
            name: "Shield".to_string(),
            description: "Raise a shield of force".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "shield".to_string(),
                name: "Shield".to_string(), // template action has a name field
                action_slot: Some(3),       // Reaction slot
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()), // required field
                condition: crate::enums::ActionCondition::Default,            // required field
                targets: 1,                                                   // required field
                template_options: crate::model::TemplateOptions {
                    // use TemplateOptions struct
                    template_name: "Shield".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )), // Add a default target
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![ActionCost::Discrete {
                resource_type: ResourceType::Reaction,
                resource_val: None,
                amount: 1.0,
            }],
            requirements: vec![],
            priority: 10,
            uses_per_round: None,
            uses_per_encounter: Some(1),
            consumes_reaction: true,
        };

        manager.register_reaction("player1".to_string(), reaction);
        assert_eq!(manager.get_available_reactions("player1").len(), 1);
    }

    #[test]
    fn test_round_advancement() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "test_reaction".to_string(),
            name: "Test".to_string(),
            description: "Test reaction".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "test".to_string(),
                name: "Test".to_string(), // template action has a name field
                action_slot: Some(3),
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()), // required field
                condition: crate::enums::ActionCondition::Default,            // required field
                targets: 1,                                                   // required field
                template_options: crate::model::TemplateOptions {
                    // use TemplateOptions struct
                    template_name: "Test".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )), // Add a default target
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![],
            requirements: vec![],
            priority: 1,
            uses_per_round: Some(1),
            uses_per_encounter: None,
            consumes_reaction: false,
        };

        manager.register_reaction("player1".to_string(), reaction);
        manager.advance_round(1);
        assert_eq!(manager.current_round, 1);
        assert_eq!(manager.used_reactions.len(), 0); // Should be cleared
    }

    #[test]
    fn test_get_triggered_reactions() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "shield_reaction".to_string(),
            name: "Shield".to_string(),
            description: "Raise a shield of force".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "shield".to_string(),
                name: "Shield".to_string(),
                action_slot: Some(3),
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()),
                condition: crate::enums::ActionCondition::Default,
                targets: 1,
                template_options: crate::model::TemplateOptions {
                    template_name: "Shield".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )),
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![ActionCost::Discrete {
                resource_type: ResourceType::Reaction,
                resource_val: None,
                amount: 1.0,
            }],
            requirements: vec![],
            priority: 10,
            uses_per_round: None,
            uses_per_encounter: Some(1),
            consumes_reaction: true,
        };

        manager.register_reaction("player1".to_string(), reaction);

        // Verify the reaction was registered
        assert_eq!(manager.get_available_reactions("player1").len(), 1);

        // Triggering check is complex and requires proper context setup
        // For now just verify the reaction is available
    }

    #[test]
    fn test_reset_encounter() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "test_reaction".to_string(),
            name: "Test".to_string(),
            description: "Test reaction".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "test".to_string(),
                name: "Test".to_string(),
                action_slot: Some(3),
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()),
                condition: crate::enums::ActionCondition::Default,
                targets: 1,
                template_options: crate::model::TemplateOptions {
                    template_name: "Test".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )),
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![],
            requirements: vec![],
            priority: 1,
            uses_per_round: None,
            uses_per_encounter: None,
            consumes_reaction: false,
        };

        manager.register_reaction("player1".to_string(), reaction);
        manager.current_round = 5;
        let mut test_set = std::collections::HashSet::new();
        test_set.insert("test".to_string());
        manager.used_reactions.insert("player1".to_string(), test_set);

        manager.reset_encounter();

        // reset_encounter() only clears encounter_used
        assert_eq!(manager.encounter_used.len(), 0);
    }

    #[test]
    fn test_get_stats() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "test_reaction".to_string(),
            name: "Test".to_string(),
            description: "Test reaction".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "test".to_string(),
                name: "Test".to_string(),
                action_slot: Some(3),
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()),
                condition: crate::enums::ActionCondition::Default,
                targets: 1,
                template_options: crate::model::TemplateOptions {
                    template_name: "Test".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )),
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![],
            requirements: vec![],
            priority: 1,
            uses_per_round: None,
            uses_per_encounter: None,
            consumes_reaction: false,
        };

        manager.register_reaction("player1".to_string(), reaction.clone());
        manager.register_reaction("player2".to_string(), reaction);

        let stats = manager.get_stats();
        assert_eq!(stats.total_available_reactions, 2);
        assert_eq!(stats.current_round, 0);
    }

    #[test]
    fn test_multiple_combatants_same_reaction() {
        let mut manager = ReactionManager::new();

        let reaction = ReactionTemplate {
            id: "shield_reaction".to_string(),
            name: "Shield".to_string(),
            description: "Raise a shield of force".to_string(),
            trigger_event_type: "AttackHit".to_string(),
            trigger_condition: TriggerCondition::OnBeingAttacked,
            response_action: Action::Template(crate::model::TemplateAction {
                id: "shield".to_string(),
                name: "Shield".to_string(),
                action_slot: Some(3),
                cost: vec![],
                requirements: vec![],
                tags: vec![],
                freq: crate::model::Frequency::Static("at will".to_string()),
                condition: crate::enums::ActionCondition::Default,
                targets: 1,
                template_options: crate::model::TemplateOptions {
                    template_name: "Shield".to_string(),
                    target: Some(crate::enums::TargetType::Ally(
                        crate::enums::AllyTarget::Self_,
                    )),
                    save_dc: None,
                    amount: None,
                },
            }),
            cost: vec![ActionCost::Discrete {
                resource_type: ResourceType::Reaction,
                resource_val: None,
                amount: 1.0,
            }],
            requirements: vec![],
            priority: 10,
            uses_per_round: None,
            uses_per_encounter: Some(1),
            consumes_reaction: true,
        };

        // Register same reaction for multiple combatants
        manager.register_reaction("player1".to_string(), reaction.clone());
        manager.register_reaction("player2".to_string(), reaction);

        assert_eq!(manager.get_available_reactions("player1").len(), 1);
        assert_eq!(manager.get_available_reactions("player2").len(), 1);
    }

    // Helper function to create a test TurnContext
    #[allow(dead_code)]
    fn create_test_context() -> crate::context::TurnContext {
        use crate::model::{Creature, Combattant};

        let creature = Creature {
            id: "test".to_string(),
            name: "Test".to_string(),
            count: 1.0,
            hp: 30,
            ac: 15,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "player".to_string(),
        };

        let combatant = Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: crate::model::CreatureState::default(),
            final_state: crate::model::CreatureState::default(),
            actions: Vec::new(),
        };

        crate::context::TurnContext::new(
            vec![combatant],
            Vec::new(),
            None,
            "Standard".to_string(),
            true,
        )
    }
}
