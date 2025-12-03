use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::events::{Event, EventBus};
use crate::resources::{ResourceLedger, ResourceType, ResetType, ActionCost};
use crate::model::{Action, Combattant};
use crate::enums::CreatureCondition;

/// Central context that maintains all game state during a combat encounter
/// Acts as the "single source of truth" for turn-based resource and event management
#[derive(Debug, Clone)]
pub struct TurnContext {
    // Resource Management
    pub resource_ledger: ResourceLedger,

    // Event Tracking
    pub event_bus: EventBus,
    pub round_number: u32,
    pub current_turn_owner: Option<String>,

    // Combat State
    pub combatants: HashMap<String, CombattantState>,
    pub active_effects: HashMap<String, ActiveEffect>,

    // Environmental Context
    pub battlefield_conditions: Vec<String>, // Simplified to string for now
    pub weather: Option<String>, // Simplified to string for now
    pub terrain: String, // Simplified to string for now
}

/// State of a combatant within the current encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombattantState {
    pub id: String,
    pub base_combatant: Combattant,
    pub current_hp: f64,
    pub temp_hp: f64,
    pub conditions: Vec<CreatureCondition>,
    pub concentration: Option<String>, // ID of spell/concentration source
    pub position: Option<String>, // Simplified position for future expansion
}

/// An active effect applied to a combatant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub effect_type: EffectType,
    pub remaining_duration: i32,
    pub conditions: Vec<String>, // Simplified to string for now
}

/// Types of effects that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    Buff(String), // Effect identifier
    DamageOverTime { damage_per_round: f64, damage_type: String },
    HealingOverTime { healing_per_round: f64 },
    Condition(CreatureCondition),
    Custom(String),
}

impl TurnContext {
    /// Create a new turn context for a combat encounter
    pub fn new(
        combatants: Vec<Combattant>,
        battlefield_conditions: Vec<String>,
        weather: Option<String>,
        terrain: String,
    ) -> Self {
        // Create resource ledger with default resources
        let mut resource_ledger = ResourceLedger::new();

        // Initialize basic resources that all combatants share
        resource_ledger.register_resource(ResourceType::Action, 1.0, Some(ResetType::ShortRest));
        resource_ledger.register_resource(ResourceType::BonusAction, 1.0, Some(ResetType::ShortRest));
        resource_ledger.register_resource(ResourceType::Reaction, 1.0, Some(ResetType::ShortRest));

        // Create combatant states
        let combatant_states: HashMap<String, CombattantState> = combatants
            .into_iter()
            .map(|c| {
                let state = CombattantState {
                    id: c.id.clone(),
                    current_hp: c.creature.hp,
                    temp_hp: 0.0,
                    conditions: Vec::new(),
                    concentration: None,
                    position: None,
                    base_combatant: c,
                };
                (state.id.clone(), state)
            })
            .collect();

        Self {
            resource_ledger,
            event_bus: EventBus::new(1000), // Keep last 1000 events
            round_number: 0,
            current_turn_owner: None,
            combatants: combatant_states,
            active_effects: HashMap::new(),
            battlefield_conditions,
            weather,
            terrain,
        }
    }

    /// Start a new turn for the specified unit
    pub fn start_new_turn(&mut self, unit_id: String) {
        // Emit turn start event
        self.event_bus.emit_event(Event::TurnStarted {
            unit_id: unit_id.clone(),
            round_number: self.round_number,
        });

        self.current_turn_owner = Some(unit_id);

        // Reset turn-based resources
        self.resource_ledger.reset_by_type(&ResetType::Turn);
    }

    /// End the current turn
    pub fn end_current_turn(&mut self) {
        if let Some(owner) = self.current_turn_owner.clone() {
            // Emit turn end event
            self.event_bus.emit_event(Event::TurnEnded {
                unit_id: owner.clone(),
                round_number: self.round_number,
            });
        }

        self.current_turn_owner = None;
    }

    /// Advance to the next round
    pub fn advance_round(&mut self) {
        self.round_number += 1;

        // Emit round start event
        self.event_bus.emit_event(Event::RoundStarted { round_number: self.round_number });

        // Reset round-based resources
        self.resource_ledger.reset_by_type(&ResetType::Round);

        // Update all effects
        self.update_effects();
    }

    /// Check if a combatant can afford the specified costs
    pub fn can_afford(&self, costs: &[ActionCost], _unit_id: &str) -> bool {
        for cost in costs {
            match cost {
                ActionCost::Discrete(resource_type, amount) => {
                    if !self.resource_ledger.has(resource_type, *amount) {
                        return false;
                    }
                },
                ActionCost::Variable(resource_type, _min, max) => {
                    if !self.resource_ledger.has(resource_type, *max) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Pay the specified costs for a combatant
    pub fn pay_costs(&mut self, costs: &[ActionCost], unit_id: &str) -> Result<(), String> {
        for cost in costs {
            match cost {
                ActionCost::Discrete(resource_type, amount) => {
                    if let Err(e) = self.resource_ledger.consume(resource_type, *amount) {
                        return Err(format!("Failed to pay cost: {}", e));
                    }

                    // Emit resource consumed event
                    self.event_bus.emit_event(Event::ResourceConsumed {
                        unit_id: unit_id.to_string(),
                        resource_type: format!("{:?}", resource_type),
                        amount: *amount,
                    });
                },
                ActionCost::Variable(resource_type, _min, max) => {
                    if let Err(e) = self.resource_ledger.consume(resource_type, *max) {
                        return Err(format!("Failed to pay cost: {}", e));
                    }

                    // Emit resource consumed event
                    self.event_bus.emit_event(Event::ResourceConsumed {
                        unit_id: unit_id.to_string(),
                        resource_type: format!("{:?}", resource_type),
                        amount: *max,
                    });
                }
            }
        }
        Ok(())
    }

    /// Record an event in the system
    pub fn record_event(&mut self, event: Event) {
        self.event_bus.emit_event(event);
    }

    /// Process pending events and return triggered reactions
    pub fn process_events(&mut self) -> Vec<(String, Action)> {
        let reactions = self.event_bus.process_pending_events();

        // Convert reaction actions - this is a placeholder implementation
        // In a full implementation, you'd resolve templates through an action resolver
        let converted_reactions: Vec<(String, Action)> = reactions
            .into_iter()
            .map(|(owner_id, action)| {
                // For now, just return the action as-is
                // TODO: Implement proper template to action conversion
                (owner_id, action)
            })
            .collect();

        converted_reactions
    }

    /// Apply an active effect to a target
    pub fn apply_effect(&mut self, effect: ActiveEffect) {
        // Emit effect application event
        self.event_bus.emit_event(Event::Custom {
            event_type: "EffectApplied".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("effect_id".to_string(), effect.id.clone());
                data.insert("target_id".to_string(), effect.target_id.clone());
                data.insert("source_id".to_string(), effect.source_id.clone());
                data
            },
            source_id: effect.source_id.clone(),
        });

        self.active_effects.insert(effect.id.clone(), effect);
    }

    /// Update all active effects (called at end of turn)
    pub fn update_effects(&mut self) {
        let mut effects_to_remove = Vec::new();

        for (effect_id, effect) in &self.active_effects {
            // Apply effect logic based on type
            match &effect.effect_type {
                EffectType::DamageOverTime { damage_per_round, damage_type } => {
                    // Apply damage
                    if let Some(combatant) = self.combatants.get_mut(&effect.target_id) {
                        let final_damage = *damage_per_round;
                        combatant.current_hp = (combatant.current_hp - final_damage).max(0.0);

                        self.event_bus.emit_event(Event::DamageTaken {
                            target_id: effect.target_id.clone(),
                            damage: final_damage,
                            damage_type: damage_type.clone(),
                        });
                    }
                },
                EffectType::HealingOverTime { healing_per_round } => {
                    // Apply healing
                    if let Some(combatant) = self.combatants.get_mut(&effect.target_id) {
                        combatant.current_hp = (combatant.current_hp + healing_per_round).min(combatant.base_combatant.creature.hp);

                        self.event_bus.emit_event(Event::HealingApplied {
                            target_id: effect.target_id.clone(),
                            amount: *healing_per_round,
                            source_id: effect.source_id.clone(),
                        });
                    }
                },
                EffectType::Condition(condition) => {
                    // Ensure condition is applied
                    if let Some(combatant) = self.combatants.get_mut(&effect.target_id) {
                        if !combatant.conditions.contains(condition) {
                            combatant.conditions.push(condition.clone());
                        }
                    }
                },
                _ => {} // Buffs and Custom effects handled elsewhere
            }

            // Decrease duration
            if effect.remaining_duration <= 0 {
                effects_to_remove.push(effect_id.clone());
            }
        }

        // Remove expired effects
        for effect_id in effects_to_remove {
            if let Some(effect) = self.active_effects.remove(&effect_id) {
                // Emit effect expiration event
                self.event_bus.emit_event(Event::Custom {
                    event_type: "EffectExpired".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("effect_id".to_string(), effect_id.clone());
                        data.insert("target_id".to_string(), effect.target_id.clone());
                        data
                    },
                    source_id: effect.source_id,
                });
            }
        }
    }

    /// Get all active effects on a specific target
    pub fn get_effects_on_target(&self, target_id: &str) -> Vec<&ActiveEffect> {
        self.active_effects
            .values()
            .filter(|effect| effect.target_id == target_id)
            .collect()
    }

    /// Get combatant state by ID
    pub fn get_combatant(&self, combatant_id: &str) -> Option<&CombattantState> {
        self.combatants.get(combatant_id)
    }

    /// Get mutable combatant state by ID
    pub fn get_combatant_mut(&mut self, combatant_id: &str) -> Option<&mut CombattantState> {
        self.combatants.get_mut(combatant_id)
    }

    /// Check if a combatant is alive
    pub fn is_combatant_alive(&self, combatant_id: &str) -> bool {
        self.combatants
            .get(combatant_id)
            .map_or(false, |c| c.current_hp > 0.0)
    }

    /// Get all alive combatants
    pub fn get_alive_combatants(&self) -> Vec<&CombattantState> {
        self.combatants
            .values()
            .filter(|c| c.current_hp > 0.0)
            .collect()
    }

    /// Get statistics about the current context
    pub fn get_stats(&self) -> ContextStats {
        ContextStats {
            round_number: self.round_number,
            current_turn_owner: self.current_turn_owner.clone(),
            total_combatants: self.combatants.len(),
            alive_combatants: self.get_alive_combatants().len(),
            total_effects: self.active_effects.len(),
            pending_events: self.event_bus.pending_count(),
        }
    }
}

/// Statistics about the turn context
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub round_number: u32,
    pub current_turn_owner: Option<String>,
    pub total_combatants: usize,
    pub alive_combatants: usize,
    pub total_effects: usize,
    pub pending_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::ActionCost;
    use crate::model::{Creature, CreatureState};

    #[test]
    fn test_turn_context_creation() {
        let creature = Creature {
            name: "Player 1".to_string(),
            hp: 30.0,
            ac: 15,
            initiative: 10,
            speed: 30,
            attacks: Vec::new(),
            buffs: Vec::new(),
            debuffs: Vec::new(),
            conditions: Vec::new(),
            spell_slots: None,
        };

        let combatants = vec![
            Combattant {
                id: "player1".to_string(),
                creature,
                initiative: 10.0,
                initial_state: CreatureState::default(),
                final_state: CreatureState::default(),
                actions: Vec::new(),
            },
        ];

        let context = TurnContext::new(
            combatants,
            Vec::new(),
            None,
            "Plains".to_string(),
        );

        assert_eq!(context.round_number, 0);
        assert!(context.current_turn_owner.is_none());
        assert_eq!(context.combatants.len(), 1);
        assert!(context.is_combatant_alive("player1"));
    }

    #[test]
    fn test_turn_management() {
        let creature = Creature {
            name: "Player 1".to_string(),
            hp: 30.0,
            ac: 15,
            initiative: 10,
            speed: 30,
            attacks: Vec::new(),
            buffs: Vec::new(),
            debuffs: Vec::new(),
            conditions: Vec::new(),
            spell_slots: None,
        };

        let combatants = vec![
            Combattant {
                id: "player1".to_string(),
                creature,
                initiative: 10.0,
                initial_state: CreatureState::default(),
                final_state: CreatureState::default(),
                actions: Vec::new(),
            },
        ];

        let mut context = TurnContext::new(
            combatants,
            Vec::new(),
            None,
            "Plains".to_string(),
        );

        context.start_new_turn("player1".to_string());
        assert_eq!(context.current_turn_owner, Some("player1".to_string()));

        context.end_current_turn();
        assert!(context.current_turn_owner.is_none());

        context.advance_round();
        assert_eq!(context.round_number, 1);
    }

    #[test]
    fn test_resource_management() {
        let creature = Creature {
            name: "Player 1".to_string(),
            hp: 30.0,
            ac: 15,
            initiative: 10,
            speed: 30,
            attacks: Vec::new(),
            buffs: Vec::new(),
            debuffs: Vec::new(),
            conditions: Vec::new(),
            spell_slots: None,
        };

        let combatants = vec![
            Combattant {
                id: "player1".to_string(),
                creature,
                initiative: 10.0,
                initial_state: CreatureState::default(),
                final_state: CreatureState::default(),
                actions: Vec::new(),
            },
        ];

        let mut context = TurnContext::new(
            combatants,
            Vec::new(),
            None,
            "Plains".to_string(),
        );

        let costs = vec![ActionCost::Discrete(crate::resources::ResourceType::Action, 1.0)];

        assert!(context.can_afford(&costs, "player1"));

        let result = context.pay_costs(&costs, "player1");
        assert!(result.is_ok());

        assert!(!context.can_afford(&costs, "player1"));
    }

    #[test]
    fn test_effect_management() {
        let creature = Creature {
            name: "Player 1".to_string(),
            hp: 30.0,
            ac: 15,
            initiative: 10,
            speed: 30,
            attacks: Vec::new(),
            buffs: Vec::new(),
            debuffs: Vec::new(),
            conditions: Vec::new(),
            spell_slots: None,
        };

        let combatants = vec![
            Combattant {
                id: "player1".to_string(),
                creature,
                initiative: 10.0,
                initial_state: CreatureState::default(),
                final_state: CreatureState::default(),
                actions: Vec::new(),
            },
        ];

        let mut context = TurnContext::new(
            combatants,
            Vec::new(),
            None,
            "Plains".to_string(),
        );

        let effect = ActiveEffect {
            id: "test_effect".to_string(),
            source_id: "source".to_string(),
            target_id: "player1".to_string(),
            effect_type: EffectType::DamageOverTime {
                damage_per_round: 5.0,
                damage_type: "Fire".to_string(),
            },
            remaining_duration: 3,
            conditions: Vec::new(),
        };

        context.apply_effect(effect);
        assert_eq!(context.active_effects.len(), 1);

        let effects = context.get_effects_on_target("player1");
        assert_eq!(effects.len(), 1);

        // Update effects (should apply damage)
        context.update_effects();
        let combatant = context.get_combatant("player1").unwrap();
        assert_eq!(combatant.current_hp, 25.0); // 30 - 5 = 25
    }
}