use crate::action_resolver::ActionResolver;
use crate::context::{CombattantState, TurnContext};
use crate::events::Event;
use crate::model::{Action, Combattant};
use crate::reactions::ReactionManager;
use crate::validation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Import the validation module

/// Central coordinator for all action processing in combat encounters
#[derive(Debug, Clone)]
pub struct ActionExecutionEngine {
    /// Centralized state management for the current encounter
    context: TurnContext,

    /// Manages reaction templates and execution
    reaction_manager: ReactionManager,

    /// Resolves actions into events
    action_resolver: ActionResolver,
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
    pub round_snapshots: Vec<Vec<CombattantState>>, // Snapshots of all combatants at end of each round
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
            Vec::new(),             // battlefield_conditions
            None,                   // weather
            "Standard".to_string(), // terrain
        );

        let mut engine = Self {
            context,
            reaction_manager: ReactionManager::new(),
            action_resolver: ActionResolver::new(),
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

        let mut round_snapshots = Vec::new();

        // Main combat loop with improved draw detection and memory management
        const MAX_ROUNDS: u32 = 50; // Increased limit with better draw detection
        const MAX_TURNS: u32 = 200; // Prevent infinite loops from extremely long battles
        while !self.is_encounter_complete() && self.context.round_number < MAX_ROUNDS && total_turns < MAX_TURNS {
            self.context.advance_round();

            let initiative_order = self.get_initiative_order();

            for combatant_id in initiative_order {
                if !self.context.is_combatant_alive(&combatant_id) {
                    continue;
                }

                total_turns += 1;

                // Execute turn with all actions and reactions
                let _turn_result = self.execute_combatant_turn(&combatant_id);

                // Process pending events (moves them to event history)
                let _reactions = self.context.process_events();

                // Process end-of-turn effects
                self.context.update_effects();

                // Check if encounter is complete after each turn
                if self.is_encounter_complete() {
                    break;
                }
            }

            // Capture optimized snapshot at end of round
            // Use references instead of cloning for memory efficiency
            // Only clone essential data, skip heavy structures where possible
            // Take snapshots less frequently to reduce memory pressure
            if self.context.round_number % 5 == 0 || self.context.round_number == MAX_ROUNDS {
                let snapshot: Vec<CombattantState> = self.context.combatants
                    .values()
                    .map(|state| CombattantState {
                        id: state.id.clone(),
                        base_combatant: state.base_combatant.clone(), // This is necessary for reference
                        current_hp: state.current_hp,
                        temp_hp: state.temp_hp,
                        conditions: state.conditions.clone(),
                        concentration: state.concentration.clone(),
                        position: state.position.clone(),
                        resources: crate::resources::ResourceLedger::new(), // Use empty ledger for snapshot
                        arcane_ward_hp: state.arcane_ward_hp,
                        cached_stats: None, // Don't clone cached stats
                    })
                    .collect();
                round_snapshots.push(snapshot);
            }

            // Continue to max rounds limit - let combat run its course
            // The round limit serves as the safety mechanism
        }

        // Generate final results
        self.generate_encounter_results(total_turns, round_snapshots)
    }

    /// Execute a single turn for a combatant
    pub fn execute_combatant_turn(&mut self, combatant_id: &str) -> TurnResult {
        let start_hp = self
            .context
            .get_combatant(combatant_id)
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

        let end_hp = self
            .context
            .get_combatant(combatant_id)
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
    pub fn execute_action_with_reactions(
        &mut self,
        actor_id: &str,
        action: Action,
    ) -> ActionResult {
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

        // Deduct usage for limited frequency actions
        match &action.base().freq {
            crate::model::Frequency::Static(s) if s == "at will" => {}
            _ => {
                // Determine action ID (which tracks usage)
                let tracking_id = action.base().id.clone();

                if let Some(combatant) = self.context.get_combatant_mut(actor_id) {
                    let current = *combatant
                        .resources
                        .current
                        .get(&tracking_id)
                        .unwrap_or(&0.0);
                    combatant
                        .resources
                        .current
                        .insert(tracking_id.clone(), (current - 1.0).max(0.0));

                    // Also mark as used in this encounter (for "1/encounter" tracking if used elsewhere)
                    // actions_used_this_encounter is not available on CombattantState, but resource deduction is sufficient.
                }
            }
        }

        // Record action start
        self.context.record_event(Event::ActionStarted {
            actor_id: actor_id.to_string(),
            action_id: action_id.clone(),
        });

        // Process action and generate events (placeholder implementation)
        let events = self.process_action(&action, actor_id);

        // Emit all events to context -> REMOVED because process_action now handles emission directly
        // for event in &events {
        //    self.context.record_event(event.clone());
        // }

        // Record the action in the combatant's history for logging
        if let Some(combatant) = self.context.get_combatant_mut(actor_id) {
            let mut targets = HashMap::new();

            // Reconstruct targets from events
            for event in &events {
                match event {
                    Event::AttackHit { target_id, .. }
                    | Event::AttackMissed { target_id, .. }
                    | Event::DamageTaken { target_id, .. }
                    | Event::HealingApplied { target_id, .. }
                    | Event::BuffApplied { target_id, .. }
                    | Event::ConditionAdded { target_id, .. } => {
                        *targets.entry(target_id.clone()).or_insert(0) += 1;
                    }
                    Event::Custom {
                        event_type, data, ..
                    } if event_type == "EffectApplied" => {
                        if let Some(target_id) = data.get("target_id") {
                            *targets.entry(target_id.clone()).or_insert(0) += 1;
                        }
                    }
                    _ => {}
                }
            }

            // If it's a multi-target action but no specific target events were generated yet
            // (e.g. clean miss or no targets found), we might miss logging targets.
            // But for now this is better than nothing.

            let action_record = crate::model::CombattantAction {
                action: action.clone(),
                targets,
            };

            combatant.base_combatant.actions.push(action_record);
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
            let reactions_refs = self
                .reaction_manager
                .get_triggered_reactions(triggering_event, &self.context);
            reactions_refs
                .into_iter()
                .map(|(id, reaction)| (id, reaction.clone()))
                .collect()
        };

        let mut results = Vec::new();

        for (combatant_id, reaction) in triggered_reactions {
            // Check if combatant can still react (might be dead, etc.)
            if !self.context.is_combatant_alive(&combatant_id) {
                continue;
            }

            match self.reaction_manager.execute_reaction(
                &combatant_id,
                &reaction,
                &mut self.context,
            ) {
                Ok(()) => {
                    results.push(ReactionResult {
                        combatant_id: combatant_id.clone(),
                        reaction_id: reaction.id.clone(),
                        success: true,
                        events_generated: self.context.event_bus.get_recent_events(5).to_vec(),
                        error: None,
                    });
                }
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

    /// Process an action and generate events using the ActionResolver
    fn process_action(&mut self, action: &Action, actor_id: &str) -> Vec<Event> {
        // Use the ActionResolver to convert the action into events
        self.action_resolver
            .resolve_action(action, &mut self.context, actor_id)
    }

    /// Get a random enemy target (different team than actor)
    #[allow(dead_code)]
    fn get_random_target(&self, actor_id: &str) -> Option<String> {
        // Get actor's team (mode)
        let actor_mode = self
            .context
            .get_combatant(actor_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();

        // Find enemies (different team)
        let alive_combatants = self.context.get_alive_combatants();
        for combatant in alive_combatants {
            if combatant.id != actor_id && combatant.base_combatant.creature.mode != actor_mode {
                return Some(combatant.id.clone());
            }
        }

        None // No valid enemy targets found
    }

    /// Select actions for a combatant (basic AI implementation)
    fn select_actions_for_combatant(&self, combatant_id: &str) -> Vec<Action> {
        let Some(combatant_state) = self.context.get_combatant(combatant_id) else {
            return Vec::new(); // Combatant not found or dead
        };

        // Track which action slots have been used
        use std::collections::HashSet;
        let mut used_slots: HashSet<Option<i32>> = HashSet::new();
        let mut selected_actions = Vec::new();

        // Score all valid actions
        let mut scored_actions: Vec<(Action, f64)> = Vec::new();

        for action in combatant_state.base_combatant.creature.actions.iter() {
            // 1. Check requirements
            if !validation::check_action_requirements(action, &self.context, combatant_id) {
                continue;
            }

            // 1.5 Check frequency limit
            match &action.base().freq {
                crate::model::Frequency::Static(s) if s == "at will" => {}
                _ => {
                    let uses = *combatant_state
                        .resources
                        .current
                        .get(&action.base().id)
                        .unwrap_or(&0.0);
                    if uses < 1.0 {
                        continue;
                    }
                }
            }

            // 2. Check affordability (costs)
            if !self.context.can_afford(&action.base().cost, combatant_id) {
                continue;
            }

            // 3. Score the action based on combat situation
            let score = self.score_action(action, combatant_id);
            if score > 0.0 {
                scored_actions.push((action.clone(), score));
            }
        }

        // Sort by score (highest first)
        scored_actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select best action per slot type
        for (action, _score) in scored_actions {
            let slot = action.base().action_slot;

            // Skip if we've already selected an action for this slot
            if used_slots.contains(&slot) {
                continue;
            }

            used_slots.insert(slot);
            selected_actions.push(action);

            // D&D 5e: Limit to 1 action per turn (plus bonus action)
            if selected_actions.len() >= 1 {
                break;
            }
        }

        selected_actions
    }

    /// Score an action based on combat situation
    fn score_action(&self, action: &Action, combatant_id: &str) -> f64 {
        // Get combatant's team (mode)
        let actor_mode = self
            .context
            .get_combatant(combatant_id)
            .map(|c| c.base_combatant.creature.mode.clone())
            .unwrap_or_default();

        match action {
            Action::Atk(atk) => {
                // Check if there are any enemies to attack
                let living_enemies = self
                    .context
                    .get_alive_combatants()
                    .iter()
                    .any(|c| c.base_combatant.creature.mode != actor_mode);

                if !living_enemies {
                    return 0.0; // No enemies to attack
                }

                // Attacks are valuable
                let base_damage = crate::dice::average(&atk.dpr);
                let num_targets = atk.targets as f64;
                base_damage * num_targets * 10.0 // * 10 to scale up
            }
            Action::Heal(heal) => {
                // Only valuable if allies are injured
                let allies: Vec<_> = self
                    .context
                    .get_alive_combatants()
                    .into_iter()
                    .filter(|c| c.base_combatant.creature.mode == actor_mode)
                    .collect();
                let injured_allies = allies
                    .iter()
                    .filter(|c| c.current_hp < c.base_combatant.creature.hp * 0.5)
                    .count();

                if injured_allies > 0 {
                    let heal_amount = crate::dice::average(&heal.amount);
                    heal_amount * injured_allies as f64 * 15.0 // Higher priority
                } else {
                    0.0 // No one needs healing
                }
            }
            Action::Buff(_buff) => {
                // Valuable at start of combat or if targets don't have buff yet
                let round = self.context.round_number;
                if round <= 2 {
                    50.0 // High priority in early rounds
                } else {
                    20.0 // Lower priority later
                }
            }
            Action::Debuff(_debuff) => {
                // Valuable against strong enemies
                let enemies: Vec<_> = self
                    .context
                    .get_alive_combatants()
                    .into_iter()
                    .filter(|c| c.base_combatant.creature.mode != actor_mode)
                    .collect();
                let strong_enemies = enemies.iter().filter(|e| e.current_hp > 20.0).count();

                if strong_enemies > 0 {
                    30.0 * strong_enemies as f64
                } else {
                    10.0
                }
            }
            Action::Template(_) => {
                // Check if already concentrating
                let is_concentrating = self
                    .context
                    .get_combatant(combatant_id)
                    .map(|c| c.concentration.is_some())
                    .unwrap_or(false);

                if is_concentrating {
                    // If already concentrating, only use template if it's very important or a bonus action
                    // For now, drastically reduce score
                    5.0
                } else {
                    // Valuable at start of combat (like Buffs)
                    let round = self.context.round_number;
                    if round <= 2 {
                        100.0 // Very high priority in early rounds!
                    } else {
                        40.0 // Lower priority later, but still decent
                    }
                }
            }
        }
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
            b.base_combatant
                .initiative
                .partial_cmp(&a.base_combatant.initiative)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combatants.into_iter().map(|c| c.id.clone()).collect()
    }

    /// Check if encounter is complete (all alive combatants are on the same team)
    fn is_encounter_complete(&self) -> bool {
        let alive_combatants = self.context.get_alive_combatants();
        
        // Calculate total HP for each team
        let mut player_total_hp = 0;
        let mut monster_total_hp = 0;

        for combatant in &alive_combatants {
            let hp = combatant.current_hp;
            if hp == 0 {
                continue; // Skip dead combatants
            }
            
            match combatant.base_combatant.creature.mode.as_str() {
                "player" => player_total_hp += hp,
                "monster" => monster_total_hp += hp,
                _ => {} // Ignore other modes
            }
        }

        // If product of team HP sums is effectively zero (very close to zero), combat is ended

        if player_total_hp * monster_total_hp == 0 {
            return true;
        }

        // If no one is alive, encounter is complete
        if alive_combatants.is_empty() {
            return true;
        }

        // If only one combatant is alive, encounter is complete
        if alive_combatants.len() == 1 {
            return true;
        }

        // Check if all alive combatants are on the same team
        let first_mode = &alive_combatants[0].base_combatant.creature.mode;
        alive_combatants
            .iter()
            .all(|c| &c.base_combatant.creature.mode == first_mode)
    }

    /// Generate final encounter results
    fn generate_encounter_results(
        &self,
        total_turns: u32,
        round_snapshots: Vec<Vec<CombattantState>>,
    ) -> EncounterResult {
        let winner = self.determine_winner();
        let event_history = self.context.event_bus.get_all_events().to_vec();
        let statistics = self.calculate_statistics(&event_history);

        EncounterResult {
            winner,
            total_rounds: self.context.round_number,
            total_turns,
            final_combatant_states: self.context.combatants.values().cloned().collect(),
            round_snapshots,
            event_history,
            statistics,
        }
    }

    /// Determine the winner of the encounter
    fn determine_winner(&self) -> Option<String> {
        let alive_combatants = self.context.get_alive_combatants();
        
        // Calculate total HP for each team
        let mut player_total_hp = 0;
        let mut monster_total_hp = 0;

        for combatant in &alive_combatants {
            let hp = combatant.current_hp;
            if hp == 0 {
                continue; // Skip dead combatants
            }
            
            match combatant.base_combatant.creature.mode.as_str() {
                "player" => player_total_hp += hp,
                "monster" => monster_total_hp += hp,
                _ => {} // Ignore other modes
            }
        }

        // Determine winner based on team HP sums
        if player_total_hp > 0.0 && monster_total_hp == 0.0 {
            // All monsters are dead, players win
            return Some("Players".to_string());
        } else if monster_total_hp > 0.0 && player_total_hp == 0.0 {
            // All players are dead, monsters win  
            return Some("Monsters".to_string());
        } else if player_total_hp == 0.0 && monster_total_hp == 0.0 {
            // Everyone is dead, draw
            return None;
        } else {
            // Multiple survivors on both teams - for now return None (draw)
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
                Event::AttackHit {
                    attacker_id,
                    damage,
                    ..
                } => {
                    *total_damage_dealt.entry(attacker_id.clone()).or_insert(0.0) += damage;
                    *attacks_landed.entry(attacker_id.clone()).or_insert(0) += 1;

                    // Check if it was a critical hit (simplified check)
                    // In a real implementation, this would be determined by the attack
                    if *damage > 20.0 {
                        // Arbitrary threshold for demo
                        critical_hits += 1;
                    }
                }
                Event::AttackMissed { attacker_id, .. } => {
                    *attacks_missed.entry(attacker_id.clone()).or_insert(0) += 1;
                }
                Event::HealingApplied {
                    source_id, amount, ..
                } => {
                    *total_healing_dealt.entry(source_id.clone()).or_insert(0.0) += amount;
                }
                Event::ActionStarted { .. } => {
                    total_actions_executed += 1;
                }
                Event::Custom { event_type, .. } => {
                    if event_type == "ReactionAction" {
                        reactions_triggered += 1;
                    }
                }
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
    use crate::model::{Combattant, Creature, CreatureState};

    #[test]
    fn test_action_execution_engine_creation() {
        let creature = Creature {
            id: "warrior1".to_string(),
            name: "Test Warrior".to_string(),
            count: 1.0,
            hp: 30.0,
            ac: 15.0,
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
            mode: "monster".to_string(),
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
        // Create a PLAYER creature
        let player_creature = Creature {
            id: "player1".to_string(),
            name: "Test Player".to_string(),
            count: 1.0,
            hp: 30.0,
            ac: 15.0,
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
            mode: "player".to_string(), // PLAYER team
        };

        // Create a MONSTER creature
        let monster_creature = Creature {
            id: "monster1".to_string(),
            name: "Test Monster".to_string(),
            count: 1.0,
            hp: 30.0,
            ac: 15.0,
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
            mode: "monster".to_string(), // MONSTER team
        };

        let combatant1 = Combattant {
            id: "player1".to_string(),
            creature: player_creature,
            initiative: 10.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let combatant2 = Combattant {
            id: "monster1".to_string(),
            creature: monster_creature,
            initiative: 5.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2]);

        // Should NOT be complete with 2 alive combatants on DIFFERENT teams
        assert!(!engine.is_encounter_complete());
    }

    #[test]
    fn test_initiative_order() {
        let creature = Creature {
            id: "test".to_string(), // Added ID
            name: "Test".to_string(),
            count: 1.0,
            hp: 30.0,
            ac: 15.0,
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
            mode: "monster".to_string(),
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
