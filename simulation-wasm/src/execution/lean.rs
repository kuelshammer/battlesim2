use crate::events::Event;
use crate::execution::engine::ActionExecutionEngine;
use crate::model::{LeanDeathEvent, LeanRoundSummary, LeanRunLog};
use std::collections::HashMap;

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use std::time::Instant;

impl ActionExecutionEngine {
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
        while !self.is_encounter_complete()
            && self.context.round_number < MAX_ROUNDS
            && total_turns < MAX_TURNS
        {
            self.context.advance_round();

            let initiative_order = self.initiative_order.clone();
            let round_number = self.context.round_number;

            // Track state at start of round for death detection
            let combatants_before_round: HashMap<String, u32> = self
                .context
                .combatants
                .values()
                .map(|c| (c.id.clone(), c.current_hp))
                .collect();

            for combatant_id in &initiative_order {
                if !self.context.is_combatant_alive(combatant_id) {
                    continue;
                }

                total_turns += 1;

                // Execute turn
                let _turn_result = self.execute_combatant_turn(combatant_id);
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
                    Event::DamageTaken {
                        target_id, damage, ..
                    } => {
                        *total_damage.entry(target_id.clone()).or_insert(0.0) += damage;
                    }
                    Event::HealingApplied {
                        target_id, amount, ..
                    } => {
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
                        let remaining_players: Vec<String> = self
                            .context
                            .combatants
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
            seed: 0,                      // Will be set by caller
            final_score: 0.0,             // Will be set by caller
            encounter_scores: Vec::new(), // Will be set by caller
            round_summaries,
            deaths: death_events,
            tpk_encounter,
            final_hp,
            survivors,
        }
    }
}
