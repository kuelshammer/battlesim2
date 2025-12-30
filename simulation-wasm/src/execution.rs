use crate::action_resolver::ActionResolver;
use crate::context::{CombattantState, TurnContext};
use crate::events::Event;
use crate::model::{Action, Combattant, LeanRunLog, LeanRoundSummary, LeanDeathEvent};
use crate::reactions::ReactionManager;
use crate::validation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use std::time::Instant;

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
    pub start_hp: u32,
    pub end_hp: u32,
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
    pub fn new(combatants: Vec<Combattant>, log_enabled: bool) -> Self {
        // Initialize TurnContext with empty battlefield conditions
        let context = TurnContext::new(
            combatants.clone(),
            Vec::new(),             // battlefield_conditions
            None,                   // weather
            "Standard".to_string(), // terrain
            log_enabled,
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
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        let encounter_start = Instant::now();

        let mut total_turns = 0u32;

        // Initialize encounter
        self.context.record_event(Event::EncounterStarted {
            combatant_ids: {
                let mut ids: Vec<String> = self.context.combatants.keys().cloned().collect();
                ids.sort();
                ids
            },
        });

        let mut round_snapshots = Vec::new();

        // Main combat loop with improved draw detection and memory management
        const MAX_ROUNDS: u32 = 50; // Increased limit with better draw detection
        const MAX_TURNS: u32 = 200; // Prevent infinite loops from extremely long battles
        while !self.is_encounter_complete() && self.context.round_number < MAX_ROUNDS && total_turns < MAX_TURNS {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("--- Round {} ---", self.context.round_number + 1).into());
            #[cfg(not(target_arch = "wasm32"))]
            println!("--- Round {} ---", self.context.round_number + 1);
            
            #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
            let round_start = Instant::now();

            self.context.advance_round();

            let initiative_order = self.get_initiative_order();

            for combatant_id in &initiative_order {
                if !self.context.is_combatant_alive(&combatant_id) {
                    continue;
                }

                total_turns += 1;

                // Execute turn with all actions and reactions
                let turn_result = self.execute_combatant_turn(&combatant_id);
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("  Combatant {}: {} actions", combatant_id, turn_result.action_results.len()).into());
                #[cfg(not(target_arch = "wasm32"))]
                println!("  Combatant {}: {} actions", combatant_id, turn_result.action_results.len());

                // Process pending events (moves them to event history)
                let _reactions = self.context.process_events();

                // Process end-of-turn effects
                self.context.update_effects();

                // Check if encounter is complete after each turn
                if self.is_encounter_complete() {
                    break;
                }
            }

            // Capture snapshot at end of every round ONLY if logging is enabled
            if self.context.log_enabled {
                let mut snapshot: Vec<CombattantState> = self.context.combatants
                    .values()
                    .cloned()
                    .collect();
                snapshot.sort_by(|a, b| a.id.cmp(&b.id));
                round_snapshots.push(snapshot);
            }

            #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
            {
                let round_duration = round_start.elapsed();
                log::info!(
                    "Round {} completed in {:?} ({} turns this round)",
                    self.context.round_number,
                    round_duration,
                    initiative_order.len()
                );
            }
        }

        // Capture a final snapshot of the absolute end state
        if self.context.log_enabled {
            let mut final_snapshot: Vec<CombattantState> = self.context.combatants
                .values()
                .map(|state| state.clone())
                .collect();
            final_snapshot.sort_by(|a, b| a.id.cmp(&b.id));
            round_snapshots.push(final_snapshot);
        }

        // Record encounter end event
        let winner = self.determine_winner();
        self.context.record_event(Event::EncounterEnded {
            winner: winner.clone(),
            reason: if self.context.round_number >= MAX_ROUNDS {
                "Maximum rounds reached".to_string()
            } else if total_turns >= MAX_TURNS {
                "Maximum turns reached".to_string()
            } else {
                "Combat resolved".to_string()
            },
        });

        // Process final events to ensure EncounterEnded moves to history
        let _ = self.context.process_events();

        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        {
            let encounter_duration = encounter_start.elapsed();
            log::info!(
                "Encounter completed in {:?} - {} rounds, {} turns, winner: {:?}",
                encounter_duration,
                self.context.round_number,
                total_turns,
                winner
            );
        }

        // Generate final results
        self.generate_encounter_results(total_turns, round_snapshots, winner)
    }

    /// Execute a full combat encounter with lean event collection (Tier B)
    /// Collects only aggregate statistics per round instead of per-attack events
    /// Memory: ~10-30 KB per run vs ~200-500 KB for full event logs
    pub fn execute_encounter_lean(&mut self, encounter_index: usize) -> LeanRunLog {
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        let encounter_start = Instant::now();

        let mut total_turns = 0u32;
        let mut round_summaries = Vec::new();
        let mut death_events = Vec::new();
        let mut tpk_encounter: Option<usize> = None;

        // Main combat loop (same as execute_encounter but with lean collection)
        const MAX_ROUNDS: u32 = 50;
        const MAX_TURNS: u32 = 200;
        while !self.is_encounter_complete() && self.context.round_number < MAX_ROUNDS && total_turns < MAX_TURNS {
            self.context.advance_round();

            let initiative_order = self.get_initiative_order();
            let round_number = self.context.round_number;

            // Track state at start of round for death detection
            let combatants_before_round: HashMap<String, u32> = self.context.combatants
                .values()
                .map(|c| (c.id.clone(), c.current_hp))
                .collect();

            for combatant_id in &initiative_order {
                if !self.context.is_combatant_alive(&combatant_id) {
                    continue;
                }

                total_turns += 1;

                // Execute turn
                let _turn_result = self.execute_combatant_turn(&combatant_id);
                let _reactions = self.context.process_events();
                self.context.update_effects();

                if self.is_encounter_complete() {
                    break;
                }
            }

            // Collect aggregate statistics for this round
            let mut total_damage: HashMap<String, f64> = HashMap::new();
            let mut total_healing: HashMap<String, f64> = HashMap::new();
            let mut deaths_this_round = Vec::new();
            let mut survivors_this_round = Vec::new();

            // Get all events from this round and aggregate them
            let all_events = self.context.event_bus.get_all_events();

            for event in all_events.iter() {
                match event {
                    Event::DamageTaken { target_id, damage, .. } => {
                        *total_damage.entry(target_id.clone()).or_insert(0.0) += damage;
                    }
                    Event::HealingApplied { target_id, amount, .. } => {
                        *total_healing.entry(target_id.clone()).or_insert(0.0) += amount;
                    }
                    _ => {}
                }
            }

            // Check for deaths (HP went from >0 to 0)
            for (combatant_id, hp_before) in &combatants_before_round {
                if let Some(combatant) = self.context.get_combatant(combatant_id) {
                    if *hp_before > 0 && combatant.current_hp == 0 {
                        // This combatant died this round
                        let is_player = combatant.side == 0;
                        death_events.push(LeanDeathEvent {
                            combatant_id: combatant_id.clone(),
                            round: round_number,
                            encounter_index,
                            was_player: is_player,
                        });
                        deaths_this_round.push(combatant_id.clone());

                        // Check for TPK (all players dead)
                        let remaining_players: Vec<String> = self.context.combatants
                            .values()
                            .filter(|c| c.side == 0 && c.current_hp > 0)
                            .map(|c| c.id.clone())
                            .collect();

                        if remaining_players.is_empty() && tpk_encounter.is_none() {
                            tpk_encounter = Some(encounter_index);
                        }
                    }
                }
            }

            // Collect survivors
            for combatant in self.context.get_alive_combatants() {
                survivors_this_round.push(combatant.id.clone());
            }

            round_summaries.push(LeanRoundSummary {
                round_number,
                encounter_index,
                total_damage,
                total_healing,
                deaths_this_round,
                survivors_this_round,
            });

            #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
            {
                let round_duration = encounter_start.elapsed();
                log::debug!(
                    "Lean Round {} completed in {:?}",
                    round_number,
                    round_duration
                );
            }
        }

        // Collect final state
        let mut final_hp: HashMap<String, u32> = HashMap::new();
        let mut survivors = Vec::new();

        for combatant in self.context.combatants.values() {
            final_hp.insert(combatant.id.clone(), combatant.current_hp);
            if combatant.current_hp > 0 {
                survivors.push(combatant.id.clone());
            }
        }
        survivors.sort();

        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        {
            let encounter_duration = encounter_start.elapsed();
            log::info!(
                "Lean encounter completed in {:?} - {} rounds, {} deaths",
                encounter_duration,
                self.context.round_number,
                death_events.len()
            );
        }

        // Note: We don't have access to seed and scores here, those are added by the caller
        LeanRunLog {
            seed: 0,  // Will be set by caller
            final_score: 0.0,  // Will be set by caller
            encounter_scores: Vec::new(),  // Will be set by caller
            round_summaries,
            deaths: death_events,
            tpk_encounter,
            final_hp,
            survivors,
        }
    }

    /// Execute a single turn for a combatant
    pub fn execute_combatant_turn(&mut self, combatant_id: &str) -> TurnResult {
        let start_hp = self
            .context
            .get_combatant(combatant_id)
            .map(|c| c.current_hp)
            .unwrap_or(0);

        self.context.start_new_turn(combatant_id.to_string());

        // For now, use a simple AI that executes available attacks
        // In a full implementation, this would use the combatant's AI or player input
        let (actions, decision_trace) = self.select_actions_for_combatant(combatant_id);

        let mut action_results = Vec::new();
        let mut effects_applied = Vec::new();

        for action in actions {
            let action_result = self.execute_action_with_reactions(combatant_id, action, Some(decision_trace.clone()));

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
            .unwrap_or(0);

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
        decision_trace: Option<HashMap<String, f64>>,
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
            decision_trace: decision_trace.unwrap_or_default(),
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
    fn select_actions_for_combatant(&mut self, combatant_id: &str) -> (Vec<Action>, HashMap<String, f64>) {
        let mut decision_trace = HashMap::new();
        // Clone the actions list to avoid borrowing self.context while iterating
        let actions = {
            let Some(combatant_state) = self.context.get_combatant(combatant_id) else {
                return (Vec::new(), decision_trace);
            };
            combatant_state.base_combatant.creature.actions.clone()
        };

        // Track which action slots have been used
        use std::collections::HashSet;
        let mut used_slots: HashSet<Option<i32>> = HashSet::new();
        let mut selected_actions = Vec::new();

        // Score all valid actions and keep track of their original index for priority
        let mut valid_actions: Vec<(usize, Action, f64)> = Vec::new();

        for (index, action) in actions.into_iter().enumerate() {
            // 1. Check requirements
            if !validation::check_action_requirements(&action, &self.context, combatant_id) {
                if index < 3 {
                    self.context.record_event(Event::ActionSkipped {
                        actor_id: combatant_id.to_string(),
                        action_id: action.base().id.clone(),
                        reason: "Requirements not met".to_string(),
                    });
                }
                continue;
            }

            // 1.5 Check frequency limit
            let is_frequent = match &action.base().freq {
                crate::model::Frequency::Static(s) if s == "at will" => true,
                _ => {
                    if let Some(combatant_state) = self.context.get_combatant(combatant_id) {
                        let uses = *combatant_state
                            .resources
                            .current
                            .get(&action.base().id)
                            .unwrap_or(&0.0);
                        uses >= 1.0
                    } else {
                        false
                    }
                }
            };

            if !is_frequent {
                if index < 3 {
                    self.context.record_event(Event::ActionSkipped {
                        actor_id: combatant_id.to_string(),
                        action_id: action.base().id.clone(),
                        reason: "Frequency limit reached".to_string(),
                    });
                }
                continue;
            }

            // 2. Check affordability (costs)
            if !self.context.can_afford(&action.base().cost, combatant_id) {
                if index < 3 {
                    self.context.record_event(Event::ActionSkipped {
                        actor_id: combatant_id.to_string(),
                        action_id: action.base().id.clone(),
                        reason: "Insufficient resources (slots/actions)".to_string(),
                    });
                }
                continue;
            }

            // 3. Score the action based on combat situation
            let score = self.score_action(&action, combatant_id);
            decision_trace.insert(action.base().name.clone(), score);
            
            // For the AI to pick it, the score must be > 0 (it must be useful)
            if score > 0.0 {
                valid_actions.push((index, action.clone(), score));
            } else if index < 3 {
                self.context.record_event(Event::ActionSkipped {
                    actor_id: combatant_id.to_string(),
                    action_id: action.base().id.clone(),
                    reason: "AI determined no benefit (e.g. already concentrating/no targets)".to_string(),
                });
            }
        }

        // Sort by original index (lowest index = highest priority)
        // This ensures the "Strategy & Priority" order from the UI is respected.
        valid_actions.sort_by_key(|a| a.0);

        // Select best action per slot type
        for (_index, action, _score) in valid_actions {
            let base_slot = action.base().action_slot;
            
            // Normalize slot: Action=0, Bonus Action=1, Reaction=4
            // D&D 5e: Action=0, Bonus Action=1.
            // MM monsters use 0 for their main actions.
            let slot = match base_slot {
                None => Some(0), 
                other => other,
            };

            // Skip if we've already selected an action for this slot (e.g. already picked an Action)
            if used_slots.contains(&slot) {
                continue;
            }

            // EXTRA CHECK: Since we are sorting by index, the FIRST valid action we encounter
            // for a slot SHOULD be the one we pick.

            used_slots.insert(slot);
            selected_actions.push(action);

            // D&D 5e: Limit to one action per distinct slot type per turn.
            // We continue the loop to check for other slots (like Bonus Action).
        }

        (selected_actions, decision_trace)
    }

    /// Score an action based on combat situation
    fn score_action(&self, action: &Action, combatant_id: &str) -> f64 {
        // Get combatant's team (mode)
        let Some(combatant) = self.context.get_combatant(combatant_id) else {
            return 0.0;
        };
        let actor_mode = combatant.base_combatant.creature.mode.clone();
        let is_concentrating = combatant.concentration.is_some();

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
                (base_damage * num_targets * 10.0).max(1.0) // Ensure at least 1.0 if valid
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
                    .filter(|c| c.current_hp < c.base_combatant.creature.hp)
                    .count();

                if injured_allies > 0 {
                    let heal_amount = crate::dice::average(&heal.amount);
                    (heal_amount * injured_allies as f64 * 15.0).max(10.0)
                } else {
                    0.0 // No one needs healing
                }
            }
            Action::Buff(buff) => {
                // Don't cast concentration buffs if already concentrating
                if is_concentrating && buff.buff.concentration {
                    return 0.0;
                }

                // Check if any allies lack this buff
                let _buff_name = buff.buff.display_name.as_ref().unwrap_or(&buff.name);
                let allies = self.context.get_alive_combatants();
                let _allies_needing_buff = allies.iter()
                    .filter(|c| c.base_combatant.creature.mode == actor_mode)
                    .filter(|c| !c.base_combatant.creature.actions.iter().any(|_| {
                        // Check if combatant already has this buff
                        // Simplified check: Does combatant have a buff with the same name?
                        // We would need access to active_effects here or check buffs map
                        false // Placeholder
                    }))
                    .count();
                
                // If the buff targets others, we should check if others are available
                // For now, assume it's always valid if round is early
                let round = self.context.round_number;
                if round <= 2 {
                    50.0 // High priority in early rounds
                } else {
                    20.0 // Lower priority later
                }
            }
            Action::Debuff(debuff) => {
                // Don't cast concentration debuffs if already concentrating
                if is_concentrating && debuff.buff.concentration {
                    return 0.0;
                }

                // Valuable against strong enemies
                let enemies: Vec<_> = self
                    .context
                    .get_alive_combatants()
                    .into_iter()
                    .filter(|c| c.base_combatant.creature.mode != actor_mode)
                    .collect();
                let strong_enemies = enemies.iter().filter(|e| e.current_hp > 20).count();

                if strong_enemies > 0 {
                    30.0 * strong_enemies as f64
                } else {
                    10.0
                }
            }
            Action::Template(tmpl) => {
                // Check if template is concentration
                // Note: We don't have easy access to template tags here without resolving
                // But we can check for common concentration spell names or just rely on the concentration flag
                // if it was set in the UI. 
                // For now, use the same logic as Template score but check for concentration more carefully
                if is_concentrating {
                    // Check if this specific template name usually requires concentration
                    let name = &tmpl.template_options.template_name;
                    if name == "Bless" || name == "Bane" || name == "Haste" || name == "Hypnotic Pattern" {
                        return 0.0;
                    }
                    5.0
                } else {
                    let round = self.context.round_number;
                    if round <= 2 {
                        100.0
                    } else {
                        40.0
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
                .then_with(|| a.id.cmp(&b.id)) // Tie-breaker: Consistent ID ordering
        });
        combatants.into_iter().map(|c| c.id.clone()).collect()
    }

    /// Check if encounter is complete (all alive combatants are on the same team)
    fn is_encounter_complete(&self) -> bool {
        let alive_combatants = self.context.get_alive_combatants();
        
        // If no one is alive, encounter is complete
        if alive_combatants.is_empty() {
            return true;
        }

        // If only one combatant is alive, encounter is complete
        if alive_combatants.len() == 1 {
            return true;
        }

        // Check if all alive combatants are on the same team (side)
        let first_side = alive_combatants[0].side;
        alive_combatants
            .iter()
            .all(|c| c.side == first_side)
    }

    /// Generate final encounter results
    fn generate_encounter_results(
        &self,
        total_turns: u32,
        round_snapshots: Vec<Vec<CombattantState>>,
        winner: Option<String>,
    ) -> EncounterResult {
        let event_history = self.context.event_bus.get_all_events().to_vec();
        let statistics = self.calculate_statistics(&event_history);

        EncounterResult {
            winner,
            total_rounds: self.context.round_number,
            total_turns,
            final_combatant_states: {
                let mut states: Vec<CombattantState> = self.context.combatants.values().cloned().collect();
                states.sort_by(|a, b| a.id.cmp(&b.id));
                states
            },
            round_snapshots,
            event_history,
            statistics,
        }
    }

    /// Determine the winner of the encounter
    fn determine_winner(&self) -> Option<String> {
        let alive_combatants = self.context.get_alive_combatants();
        
        // Calculate total HP for each side
        let mut team1_hp = 0;
        let mut team2_hp = 0;

        for combatant in &alive_combatants {
            if combatant.side == 0 {
                team1_hp += combatant.current_hp;
            } else {
                team2_hp += combatant.current_hp;
            }
        }

        // Determine winner based on side HP sums
        if team1_hp > 0 && team2_hp == 0 {
            // All of side 1 are dead, side 0 wins
            Some("Players".to_string())
        } else if team2_hp > 0 && team1_hp == 0 {
            // All of side 0 are dead, side 1 wins
            Some("Monsters".to_string())
        } else if team1_hp == 0 && team2_hp == 0 {
            // Everyone is dead, draw
            None
        } else {
            // Multiple survivors on both sides - for now return None (draw)
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
            mode: "monster".to_string(),
        };

        let combatant = Combattant { team: 0,
            id: "warrior1".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            final_state: CreatureState {
                current_hp: 30,
                ..CreatureState::default()
            },
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant], true);

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
            mode: "player".to_string(), // PLAYER team
        };

        // Create a MONSTER creature
        let monster_creature = Creature {
            id: "monster1".to_string(),
            name: "Test Monster".to_string(),
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
            mode: "monster".to_string(), // MONSTER team
        };

        let combatant1 = Combattant { team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(player_creature),
            initiative: 10.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let combatant2 = Combattant { team: 1,
            id: "monster1".to_string(),
            creature: std::sync::Arc::new(monster_creature),
            initiative: 5.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2], true);

        // Should NOT be complete with 2 alive combatants on DIFFERENT teams
        assert!(!engine.is_encounter_complete());
    }

    #[test]
    fn test_initiative_order() {
        let creature = Creature {
            id: "test".to_string(), // Added ID
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
            mode: "monster".to_string(),
        };

        let combatant1 = Combattant { team: 0,
            id: "fast".to_string(),
            creature: std::sync::Arc::new(creature.clone()),
            initiative: 15.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let combatant2 = Combattant { team: 0,
            id: "slow".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 5.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2], true);
        let order = engine.get_initiative_order();

        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "fast"); // Higher initiative first
        assert_eq!(order[1], "slow");
    }

    #[test]
    fn test_execute_encounter_basic() {
        let player_creature = Creature {
            id: "player1".to_string(),
            name: "Test Player".to_string(),
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

        let monster_creature = Creature {
            id: "monster1".to_string(),
            name: "Test Monster".to_string(),
            count: 1.0,
            hp: 10,
            ac: 12,
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

        let player = Combattant {
            team: 0,
            id: "player1".to_string(),
            creature: std::sync::Arc::new(player_creature),
            initiative: 10.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let monster = Combattant {
            team: 1,
            id: "monster1".to_string(),
            creature: std::sync::Arc::new(monster_creature),
            initiative: 5.0,
            initial_state: CreatureState { current_hp: 10, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 10, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let mut engine = ActionExecutionEngine::new(vec![player, monster], true);

        // Execute the encounter
        let result = engine.execute_encounter();

        // Verify structure
        assert!(!result.round_snapshots.is_empty(), "Should have at least one round");
        assert!(result.total_rounds <= 100, "Should not exceed max rounds");

        // Verify combatants exist in results
        assert_eq!(result.final_combatant_states.len(), 2, "Should have 2 combatants");
    }

    #[test]
    fn test_empty_combatants() {
        let mut engine = ActionExecutionEngine::new(vec![], true);

        // Should complete immediately with no combatants
        assert!(engine.is_encounter_complete());

        let result = engine.execute_encounter();
        // Empty encounter should have no rounds or very few
        assert_eq!(result.total_rounds, 0);
        assert_eq!(result.final_combatant_states.len(), 0);
    }

    #[test]
    fn test_single_combatant() {
        let creature = Creature {
            id: "solo".to_string(),
            name: "Solo".to_string(),
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
            mode: "monster".to_string(),
        };

        let combatant = Combattant {
            team: 0,
            id: "solo".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let mut engine = ActionExecutionEngine::new(vec![combatant], true);

        // Single combatant should complete immediately
        assert!(engine.is_encounter_complete());

        let result = engine.execute_encounter();
        // Should handle gracefully - no rounds because there's nothing to fight
        assert!(result.round_snapshots.is_empty() || result.total_rounds <= 100);
    }

    #[test]
    fn test_get_context_stats() {
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
            mode: "monster".to_string(),
        };

        let combatant = Combattant {
            team: 0,
            id: "test".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 10.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant], true);

        let stats = engine.get_context_stats();
        assert_eq!(stats.total_combatants, 1);
        assert_eq!(stats.round_number, 0);
    }

    #[test]
    fn test_same_team_combatants() {
        // Two combatants on the same team should not fight
        let creature = Creature {
            id: "ally".to_string(),
            name: "Ally".to_string(),
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

        let combatant1 = Combattant {
            team: 0,
            id: "ally1".to_string(),
            creature: std::sync::Arc::new(creature.clone()),
            initiative: 10.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let combatant2 = Combattant {
            team: 0,
            id: "ally2".to_string(),
            creature: std::sync::Arc::new(creature),
            initiative: 5.0,
            initial_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            final_state: CreatureState { current_hp: 30, ..CreatureState::default() },
            actions: Vec::new(),
        };

        let engine = ActionExecutionEngine::new(vec![combatant1, combatant2], true);

        // Same team combatants should complete immediately (no enemies)
        assert!(engine.is_encounter_complete());
    }
}
