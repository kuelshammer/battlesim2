use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::context::{TurnContext, CombattantState};
use crate::reactions::ReactionManager;
use crate::events::Event;
use crate::model::{Action, Combattant};

/// Central coordinator for all action processing in combat encounters
#[derive(Debug, Clone)]
pub struct ActionExecutionEngine {
    /// Centralized state management for the current encounter
    context: TurnContext,

    /// Manages reaction templates and execution
    reaction_manager: ReactionManager,
}

/// Result of executing a single action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub actor_id: String,
    pub action_id: String,
    pub success: bool,
    pub events_generated: Vec<Event>,
    pub reactions_triggered: Vec<ReactionResult>,
    pub error: Option<String>,
}

/// Result of a reaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionResult {
    pub combatant_id: String,
    pub reaction_id: String,
    pub success: bool,
    pub events_generated: Vec<Event>,
    pub error: Option<String>,
}

/// Result of executing a complete turn for a combatant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub combatant_id: String,
    pub round_number: u32,
    pub action_results: Vec<ActionResult>,
    pub effects_applied: Vec<String>, // Effect IDs applied during this turn
    pub start_hp: f64,
    pub end_hp: f64,
}

/// Result of a complete encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterResult {
    pub winner: Option<String>,
    pub total_rounds: u32,
    pub total_turns: u32,
    pub final_combatant_states: Vec<CombattantState>,
    pub event_history: Vec<Event>,
    pub statistics: EncounterStatistics,
}

/// Statistics collected during an encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterStatistics {
    pub total_damage_dealt: HashMap<String, f64>,
    pub total_healing_dealt: HashMap<String, f64>,
    pub attacks_landed: HashMap<String, u32>,
    pub attacks_missed: HashMap<String, u32>,
    pub reactions_triggered: u32,
    pub critical_hits: u32,
    pub total_actions_executed: u32,
}

impl ActionExecutionEngine {
    /// Create a new execution engine for the given combatants
    pub fn new(combatants: Vec<Combattant>) -> Self {
        // Initialize TurnContext with empty battlefield conditions
        let context = TurnContext::new(
            combatants.clone(),
            Vec::new(), // battlefield_conditions
            None, // weather
            "Standard".to_string(), // terrain
        );

        let mut engine = Self {
            context,
            reaction_manager: ReactionManager::new(),
        };

        // Register reactions from combatants (placeholder for now)
        engine.register_default_reactions(&combatants);

        engine
    }

    /// Execute a full combat encounter until completion
    pub fn execute_encounter(&mut self) -> EncounterResult {
        let mut total_turns = 0u32;

        // Initialize encounter
        self.context.record_event(Event::EncounterStarted {
            combatant_ids: self.context.combatants.keys().cloned().collect(),
        });

        // Main combat loop
        while !self.is_encounter_complete() {
            self.context.advance_round();

            let initiative_order = self.get_initiative_order();

            for combatant_id in initiative_order {
                if !self.context.is_combatant_alive(&combatant_id) {
                    continue;
                }

                total_turns += 1;

                // Execute turn with all actions and reactions
                let _turn_result = self.execute_combatant_turn(&combatant_id);

                // Process end-of-turn effects
                self.context.update_effects();

                // Check if encounter is complete after each turn
                if self.is_encounter_complete() {
                    break;
                }
            }
        }

        // Generate final results
        self.generate_encounter_results(total_turns)
    }

    /// Execute a single turn for a combatant
    pub fn execute_combatant_turn(&mut self, combatant_id: &str) -> TurnResult {
        let start_hp = self.context.get_combatant(combatant_id)
            .map(|c| c.current_hp)
            .unwrap_or(0.0);

        self.context.start_new_turn(combatant_id.to_string());

        // For now, use a simple AI that executes available attacks
        // In a full implementation, this would use the combatant's AI or player input
        let actions = self.select_actions_for_combatant(combatant_id);

        let mut action_results = Vec::new();
        let mut effects_applied = Vec::new();

        for action in actions {
            let action_result = self.execute_action_with_reactions(combatant_id, action);

            // Collect effect IDs from events
            for event in &action_result.events_generated {
                if let Event::BuffApplied { buff_id, .. } = event {
                    effects_applied.push(buff_id.clone());
                }
                if let Event::ConditionAdded { .. } = event {
                    effects_applied.push(format!("{:?}", event));
                }
            }

            action_results.push(action_result);
        }

        self.context.end_current_turn();

        let end_hp = self.context.get_combatant(combatant_id)
            .map(|c| c.current_hp)
            .unwrap_or(0.0);

        TurnResult {
            combatant_id: combatant_id.to_string(),
            round_number: self.context.round_number,
            action_results,
            effects_applied,
            start_hp,
            end_hp,
        }
    }

    /// Execute an action and process all resulting reactions
    pub fn execute_action_with_reactions(&mut self, actor_id: &str, action: Action) -> ActionResult {
        let action_id = action.base().id.clone();

        // Check if combatant can afford the action
        if !self.context.can_afford(&action.base().cost, actor_id) {
            return ActionResult {
                actor_id: actor_id.to_string(),
                action_id,
                success: false,
                events_generated: Vec::new(),
                reactions_triggered: Vec::new(),
                error: Some("Cannot afford action costs".to_string()),
            };
        }

        // Pay costs
        if let Err(e) = self.context.pay_costs(&action.base().cost, actor_id) {
            return ActionResult {
                actor_id: actor_id.to_string(),
                action_id,
                success: false,
                events_generated: Vec::new(),
                reactions_triggered: Vec::new(),
                error: Some(format!("Failed to pay costs: {}", e)),
            };
        }

        // Record action start
        self.context.record_event(Event::ActionStarted {
            actor_id: actor_id.to_string(),
            action_id: action_id.clone(),
        });

        // Process action and generate events (placeholder implementation)
        let events = self.process_action(&action, actor_id);

        // Emit all events to context
        for event in &events {
            self.context.record_event(event.clone());
        }

        // Process reaction phase for each event
        let mut reactions_triggered = Vec::new();
        for event in &events {
            let event_reactions = self.process_reaction_phase(event);
            reactions_triggered.extend(event_reactions);
        }

        ActionResult {
            actor_id: actor_id.to_string(),
            action_id,
            success: true,
            events_generated: events,
            reactions_triggered,
            error: None,
        }
    }

    /// Process reaction phase after an action
    pub fn process_reaction_phase(&mut self, triggering_event: &Event) -> Vec<ReactionResult> {
        // Get reactions that would trigger for this event (collect as owned data)
        let triggered_reactions: Vec<(String, crate::reactions::ReactionTemplate)> = {
            let reactions_refs = self.reaction_manager.get_triggered_reactions(triggering_event, &self.context);
            reactions_refs.into_iter().map(|(id, reaction)| (id, reaction.clone())).collect()
        };

        let mut results = Vec::new();

        for (combatant_id, reaction) in triggered_reactions {
            // Check if combatant can still react (might be dead, etc.)
            if !self.context.is_combatant_alive(&combatant_id) {
                continue;
            }

            match self.reaction_manager.execute_reaction(&combatant_id, &reaction, &mut self.context) {
                Ok(()) => {
                    results.push(ReactionResult {
                        combatant_id: combatant_id.clone(),
                        reaction_id: reaction.id.clone(),
                        success: true,
                        events_generated: self.context.event_bus.get_recent_events(5).to_vec(),
                        error: None,
                    });
                },
                Err(e) => {
                    results.push(ReactionResult {
                        combatant_id: combatant_id.clone(),
                        reaction_id: reaction.id.clone(),
                        success: false,
                        events_generated: Vec::new(),
                        error: Some(e),
                    });
                }
            }
        }

        results
    }

    /// Process an action and generate events (placeholder implementation)
    fn process_action(&mut self, action: &Action, actor_id: &str) -> Vec<Event> {
        let mut events = Vec::new();

        match action {
            Action::Atk(attack_action) => {
                // Simple attack processing - would be more complex in full implementation
                let target_id = self.get_random_target(actor_id);
                if let Some(target) = target_id {
                    // Simplified damage calculation
                    let damage = crate::dice::average(&attack_action.dpr);

                    events.push(Event::AttackHit {
                        attacker_id: actor_id.to_string(),
                        target_id: target.clone(),
                        damage,
                    });

                    events.push(Event::DamageTaken {
                        target_id: target,
                        damage,
                        damage_type: "Physical".to_string(),
                    });
                }
            },
            Action::Heal(heal_action) => {
                // Simple healing processing
                let heal_amount = crate::dice::average(&heal_action.amount);
                let target_id = actor_id.to_string(); // Self-target for simplicity

                events.push(Event::HealingApplied {
                    target_id: target_id.clone(),
                    amount: heal_amount,
                    source_id: actor_id.to_string(),
                });
            },
            Action::Buff(_buff_action) => {
                // Placeholder for buff processing
                events.push(Event::BuffApplied {
                    target_id: actor_id.to_string(),
                    buff_id: action.base().id.clone(),
                    source_id: actor_id.to_string(),
                });
            },
            Action::Debuff(_debuff_action) => {
                // Placeholder for debuff processing
                let target_id = self.get_random_target(actor_id);
                if let Some(target) = target_id {
                    events.push(Event::ConditionAdded {
                        target_id: target.clone(),
                        condition: crate::enums::CreatureCondition::Incapacitated,
                        source_id: actor_id.to_string(),
                    });
                }
            },
            Action::Template(_template_action) => {
                // Placeholder for template actions
                events.push(Event::Custom {
                    event_type: "TemplateAction".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("action_id".to_string(), action.base().id.clone());
                        data
                    },
                    source_id: actor_id.to_string(),
                });
            }
        }

        events
    }

    /// Get a random target (simplified - would use proper targeting in full implementation)
    fn get_random_target(&self, actor_id: &str) -> Option<String> {
        let alive_combatants = self.context.get_alive_combatants();

        for combatant in alive_combatants {
            if combatant.id != actor_id {
                return Some(combatant.id.clone());
            }
        }

        None // No valid targets found
    }

    /// Select actions for a combatant (simple AI for now)
    fn select_actions_for_combatant(&self, _combatant_id: &str) -> Vec<Action> {
        // For now, return empty vector - this would be implemented with AI
        // In a full implementation, this would:
        // 1. Check available actions for the combatant
        // 2. Evaluate combat situation
        // 3. Select best actions based on AI or player input
        // 4. Return valid actions that the combatant can afford

        Vec::new()
    }

    /// Register default reactions for combatants
    fn register_default_reactions(&mut self, _combatants: &[Combattant]) {
        // This is a placeholder for registering reactions from combatants
        // In a full implementation, combatants would have reaction templates defined

        // for combatant in combatants {
        //     // Example: Add a simple defensive reaction
        //     // This would come from combatant data in a real implementation
        // }
    }

    /// Get combatants sorted by initiative
    fn get_initiative_order(&self) -> Vec<String> {
        let mut combatants: Vec<_> = self.context.combatants.values().collect();
        combatants.sort_by(|a, b| {
            b.base_combatant.initiative.partial_cmp(&a.base_combatant.initiative)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combatants.into_iter().map(|c| c.id.clone()).collect()
    }

    /// Check if encounter is complete
    fn is_encounter_complete(&self) -> bool {
        let alive_combatants = self.context.get_alive_combatants();

        // Simple completion check: if only one or zero combatants are alive
        alive_combatants.len() <= 1

        // TODO: In a full implementation, check team composition when teams are added
        // let first_team = alive_combatants[0].base_combatant.team;
        // alive_combatants.iter().all(|c| c.base_combatant.team == first_team)
    }

    /// Generate final encounter results
    fn generate_encounter_results(&self, total_turns: u32) -> EncounterResult {
        let winner = self.determine_winner();
        let event_history = self.context.event_bus.get_all_events().to_vec();
        let statistics = self.calculate_statistics(&event_history);

        EncounterResult {
            winner,
            total_rounds: self.context.round_number,
            total_turns,
            final_combatant_states: self.context.combatants.values().cloned().collect(),
            event_history,
            statistics,
        }
    }

    /// Determine the winner of the encounter
    fn determine_winner(&self) -> Option<String> {
        let alive_combatants = self.context.get_alive_combatants();

        if alive_combatants.is_empty() {
            None // Draw - everyone died
        } else if alive_combatants.len() == 1 {
            Some(alive_combatants[0].id.clone())
        } else {
            // Multiple survivors - for now return None (draw)
            // TODO: In a full implementation, check team composition when teams are added
            None
        }
    }

    /// Calculate encounter statistics from event history
    fn calculate_statistics(&self, events: &[Event]) -> EncounterStatistics {
        let mut total_damage_dealt = HashMap::new();
        let mut total_healing_dealt = HashMap::new();
        let mut attacks_landed = HashMap::new();
        let mut attacks_missed = HashMap::new();
        let mut reactions_triggered = 0u32;
        let mut critical_hits = 0u32;
        let mut total_actions_executed = 0u32;

        for event in events {
            match event {
                Event::AttackHit { attacker_id, damage, .. } => {
                    *total_damage_dealt.entry(attacker_id.clone()).or_insert(0.0) += damage;
                    *attacks_landed.entry(attacker_id.clone()).or_insert(0) += 1;

                    // Check if it was a critical hit (simplified check)
                    // In a real implementation, this would be determined by the attack
                    if *damage > 20.0 { // Arbitrary threshold for demo
                        critical_hits += 1;
                    }
                },
                Event::AttackMissed { attacker_id, .. } => {
                    *attacks_missed.entry(attacker_id.clone()).or_insert(0) += 1;
                },
                Event::HealingApplied { source_id, amount, .. } => {
                    *total_healing_dealt.entry(source_id.clone()).or_insert(0.0) += amount;
                },
                Event::ActionStarted { .. } => {
                    total_actions_executed += 1;
                },
                Event::Custom { event_type, .. } => {
                    if event_type == "ReactionAction" {
                        reactions_triggered += 1;
                    }
                },
                _ => {}
            }
        }

        EncounterStatistics {
            total_damage_dealt,
            total_healing_dealt,
            attacks_landed,
            attacks_missed,
            reactions_triggered,
            critical_hits,
            total_actions_executed,
        }
    }

    /// Get current context statistics
    pub fn get_context_stats(&self) -> crate::context::ContextStats {
        self.context.get_stats()
    }

    /// Get reaction manager statistics
    pub fn get_reaction_stats(&self) -> crate::reactions::ReactionStats {
        self.reaction_manager.get_stats()
    }

    /// Get event bus statistics
    pub fn get_event_bus_stats(&self) -> crate::events::EventBusStats {
        self.context.event_bus.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Creature, CreatureState};

    #[test]
    fn test_action_execution_engine_creation() {
        let creature = Creature {
            name: "Test Warrior".to_string(),
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

        let combatant = Combattant {
            id: "warrior1".to_string(),
            creature,
            initiative: 10.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant]);

        assert_eq!(engine.context.combatants.len(), 1);
        assert_eq!(engine.context.round_number, 0);
        assert!(engine.context.is_combatant_alive("warrior1"));
    }

    #[test]
    fn test_encounter_completion() {
        let creature = Creature {
            name: "Test Warrior".to_string(),
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

        let combatant1 = Combattant {
            id: "warrior1".to_string(),
            creature: creature.clone(),
            initiative: 10.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let combatant2 = Combattant {
            id: "warrior2".to_string(),
            creature,
            initiative: 5.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2]);

        // Should not be complete with 2 alive combatants on different teams
        assert!(!engine.is_encounter_complete());
    }

    #[test]
    fn test_initiative_order() {
        let creature = Creature {
            name: "Test".to_string(),
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

        let combatant1 = Combattant {
            id: "fast".to_string(),
            creature: creature.clone(),
            initiative: 15.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let combatant2 = Combattant {
            id: "slow".to_string(),
            creature,
            initiative: 5.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2]);
        let order = engine.get_initiative_order();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "fast"); // Higher initiative first
        assert_eq!(order[1], "slow");
    }
}