use crate::action_resolver::ActionResolver;
use crate::context::{CombattantState, TurnContext};
use crate::events::Event;
use crate::model::{Action, Combattant};
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
    pub(crate) context: TurnContext,

    /// Manages reaction templates and execution
    pub(crate) reaction_manager: ReactionManager,

    /// Resolves actions into events
    pub(crate) action_resolver: ActionResolver,

    /// Pre-calculated initiative order for the encounter
    pub(crate) initiative_order: Vec<String>,
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

        // Pre-calculate initiative order
        let mut sorted_combatants = combatants;
        sorted_combatants.sort_by(|a, b| {
            b.initiative
                .partial_cmp(&a.initiative)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        let initiative_order = sorted_combatants.into_iter().map(|c| c.id).collect();

        let mut engine = Self {
            context: context.clone(),
            reaction_manager: ReactionManager::new(),
            action_resolver: ActionResolver::with_cache(context.combat_stats_cache.clone()),
            initiative_order,
        };

        // Register reactions from combatants (placeholder for now)
        engine.register_default_reactions(&[]); // combatants consumed by map above

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

            let initiative_order = self.initiative_order.clone();

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

                    // Track cumulative expenditure
                    let weight = crate::intensity_calculation::get_resource_weight(
                        &tracking_id,
                        &combatant.resources.reset_rules,
                        combatant.base_combatant.creature.con_modifier.unwrap_or(0.0)
                    );
                    combatant.cumulative_spent += weight;
                }
            }
        }

        // Record action start
        self.context.record_event(Event::ActionStarted {
            actor_id: actor_id.to_string(),
            action_id: action_id.clone(),
            decision_trace: decision_trace.unwrap_or_default(),
        });

        // Process action and generate events
        let events = self.process_action(&action, actor_id);

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
    pub(crate) fn process_action(&mut self, action: &Action, actor_id: &str) -> Vec<Event> {
        // Use the ActionResolver to convert the action into events
        self.action_resolver
            .resolve_action(action, &mut self.context, actor_id)
    }

    /// Select actions for a combatant (basic AI implementation)
    pub(crate) fn select_actions_for_combatant(&mut self, combatant_id: &str) -> (Vec<Action>, HashMap<String, f64>) {
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
        valid_actions.sort_by_key(|a| a.0);

        // Select best action per slot type
        for (_index, action, _score) in valid_actions {
            let base_slot = action.base().action_slot;
            
            let slot = match base_slot {
                None => Some(0), 
                other => other,
            };

            if used_slots.contains(&slot) {
                continue;
            }

            used_slots.insert(slot);
            selected_actions.push(action);
        }

        (selected_actions, decision_trace)
    }

    /// Score an action based on combat situation
    pub(crate) fn score_action(&self, action: &Action, combatant_id: &str) -> f64 {
        // Get combatant's team (mode)
        let Some(combatant) = self.context.get_combatant(combatant_id) else {
            return 0.0;
        };
        let actor_mode = combatant.base_combatant.creature.mode.clone();
        let is_concentrating = combatant.concentration.is_some();

        match action {
            Action::Atk(atk) => {
                let living_enemies = self
                    .context
                    .get_alive_combatants()
                    .iter()
                    .any(|c| c.base_combatant.creature.mode != actor_mode);

                if !living_enemies {
                    return 0.0;
                }

                let base_damage = crate::dice::average(&atk.dpr);
                let num_targets = atk.targets as f64;
                (base_damage * num_targets * 10.0).max(1.0)
            }
            Action::Heal(heal) => {
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
                    0.0
                }
            }
            Action::Buff(buff) => {
                if is_concentrating && buff.buff.concentration {
                    return 0.0;
                }

                let round = self.context.round_number;
                if round <= 2 {
                    50.0
                } else {
                    20.0
                }
            }
            Action::Debuff(debuff) => {
                if is_concentrating && debuff.buff.concentration {
                    return 0.0;
                }

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
                if is_concentrating {
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
    pub(crate) fn register_default_reactions(&mut self, _combatants: &[Combattant]) {
        // Placeholder
    }

    /// Check if encounter is complete (all alive combatants are on the same team)
    pub(crate) fn is_encounter_complete(&self) -> bool {
        let alive_combatants = self.context.get_alive_combatants();
        
        if alive_combatants.is_empty() || alive_combatants.len() == 1 {
            return true;
        }

        let first_side = alive_combatants[0].side;
        alive_combatants
            .iter()
            .all(|c| c.side == first_side)
    }

    /// Generate final encounter results
    pub(crate) fn generate_encounter_results(
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
    pub(crate) fn determine_winner(&self) -> Option<String> {
        let alive_combatants = self.context.get_alive_combatants();
        
        let mut team1_hp = 0;
        let mut team2_hp = 0;

        for combatant in &alive_combatants {
            if combatant.side == 0 {
                team1_hp += combatant.current_hp;
            } else {
                team2_hp += combatant.current_hp;
            }
        }

        if team1_hp > 0 && team2_hp == 0 {
            Some("Players".to_string())
        } else if team2_hp > 0 && team1_hp == 0 {
            Some("Monsters".to_string())
        } else {
            None
        }
    }

    /// Calculate encounter statistics from event history
    pub(crate) fn calculate_statistics(&self, events: &[Event]) -> EncounterStatistics {
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

                    if *damage > 20.0 {
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
