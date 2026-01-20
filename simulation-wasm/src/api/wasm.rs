use crate::aggregation::{calculate_cumulative_score, calculate_score};
use crate::api::runner::run_single_event_driven_simulation;
use crate::model::{Creature, SimulationResult, SimulationRun, TimelineStep};
use crate::sorting::{assign_party_slots, calculate_average_attack_bonus, PlayerSlot};
use wasm_bindgen::prelude::*;

use std::collections::HashMap;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullAnalysisOutput {
    pub overall: crate::decile_analysis::AggregateOutput,
    pub encounters: Vec<crate::decile_analysis::AggregateOutput>,
    #[serde(rename = "averageMonsterAttackBonus")]
    pub average_monster_attack_bonus: f64,
    #[serde(rename = "partySlots")]
    pub party_slots: Vec<PlayerSlot>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullSimulationOutput {
    pub results: Vec<SimulationResult>,
    pub analysis: FullAnalysisOutput,
    pub first_run_events: Vec<crate::events::Event>,
}

#[wasm_bindgen]
pub struct ChunkedSimulationRunner {
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    current_iteration: usize,
    summarized_results: Vec<crate::model::SimulationResult>,
    lightweight_runs: Vec<crate::model::LightweightRun>,
    base_seed: u64,
}

#[wasm_bindgen]
impl ChunkedSimulationRunner {
    #[wasm_bindgen(constructor)]
    pub fn new(
        players: JsValue,
        timeline: JsValue,
        seed: Option<u64>,
    ) -> Result<ChunkedSimulationRunner, JsValue> {
        // Debug: Log incoming players JSON to browser console
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&players);

        let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
        let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

        let max_capacity = 10100;

        Ok(ChunkedSimulationRunner {
            players,
            timeline,
            current_iteration: 0,
            summarized_results: Vec::with_capacity(max_capacity),
            lightweight_runs: Vec::with_capacity(max_capacity),
            base_seed: seed.unwrap_or(0),
        })
    }

    pub fn run_chunk(&mut self, chunk_size: usize) -> usize {
        let start = self.current_iteration;
        let end = start + chunk_size;

        for i in start..end {
            let seed = self.base_seed.wrapping_add(i as u64);
            crate::rng::seed_rng(seed);

            let (result, _) =
                run_single_event_driven_simulation(&self.players, &self.timeline, false);

            let score = calculate_score(&result);
            let mut encounter_scores = Vec::new();
            for (idx, _) in result.encounters.iter().enumerate() {
                encounter_scores.push(calculate_cumulative_score(&result, idx));
            }

            let has_death = result.encounters.iter().any(|e| {
                e.rounds
                    .last()
                    .map(|r| r.team1.iter().any(|c| c.final_state.current_hp == 0))
                    .unwrap_or(false)
            });

            self.lightweight_runs.push(crate::model::LightweightRun {
                seed,
                encounter_scores,
                final_score: score,
                total_hp_lost: 0.0,
                total_survivors: 0,
                has_death,
                first_death_encounter: None,
            });

            self.summarized_results
                .push(crate::utils::summarize_result(result, seed));
        }

        self.current_iteration = end;
        self.current_iteration
    }

    pub fn get_analysis(&mut self, k_factor: u32) -> Result<JsValue, JsValue> {
        let selected_seeds =
            crate::seed_selection::select_interesting_seeds_with_tiers(&self.lightweight_runs);
        let interesting_seeds: Vec<u64> = selected_seeds.iter().map(|s| s.seed).collect();

        let mut seed_to_events = HashMap::new();
        let mut median_run_events = Vec::new();

        let mut global_scores: Vec<(usize, f64)> = self
            .lightweight_runs
            .iter()
            .enumerate()
            .map(|(i, r)| (i, r.final_score))
            .collect();
        global_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let total_runs = self.lightweight_runs.len();
        if total_runs == 0 {
            return Err(JsValue::from_str("No runs to analyze"));
        }

        let median_seed = self.lightweight_runs[global_scores[total_runs / 2].0].seed;

        for &seed in &interesting_seeds {
            crate::rng::seed_rng(seed);
            let (_, events) =
                run_single_event_driven_simulation(&self.players, &self.timeline, true);

            if seed == median_seed {
                median_run_events = events.clone();
            }
            seed_to_events.insert(seed, events);
        }

        let mut final_runs: Vec<SimulationRun> = self
            .summarized_results
            .iter()
            .zip(self.lightweight_runs.iter())
            .map(|(result, light)| {
                let events = seed_to_events.get(&light.seed).cloned().unwrap_or_default();
                SimulationRun {
                    result: result.clone(),
                    events,
                }
            })
            .collect();

        let sr_count = self
            .timeline
            .iter()
            .filter(|s| matches!(s, crate::model::TimelineStep::ShortRest(_)))
            .count();

        let overall = crate::decile_analysis::run_decile_analysis_with_logs(
            &mut final_runs,
            "Current Scenario",
            self.players.len(),
            sr_count,
        );

        let num_encounters = final_runs
            .first()
            .map(|r| r.result.encounters.len())
            .unwrap_or(0);
        let mut encounters_analysis = Vec::new();
        for i in 0..num_encounters {
            let analysis = crate::decile_analysis::run_encounter_analysis_with_logs(
                &mut final_runs,
                i,
                &format!("Encounter {}", i + 1),
                self.players.len(),
                sr_count,
            );
            encounters_analysis.push(analysis);
        }

        final_runs.sort_by(|a, b| {
            let score_a = calculate_score(&a.result);
            let score_b = calculate_score(&b.result);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.result.seed.cmp(&b.result.seed))
        });

        let reduced_results = if k_factor > 1 {
            let bucket_size = total_runs / 100;
            let mut bucket_medians = Vec::with_capacity(100);

            if bucket_size > 0 {
                for percentile in 0..100 {
                    let start_idx = percentile * bucket_size;
                    let median_idx = start_idx + (bucket_size / 2);

                    if median_idx < total_runs {
                        bucket_medians.push(final_runs[median_idx].result.clone());
                    }
                }
            } else {
                bucket_medians = final_runs.iter().map(|r| r.result.clone()).collect();
            }
            bucket_medians
        } else {
            let median_idx = total_runs / 2;
            let decile = total_runs as f64 / 10.0;
            let representative_indices = vec![
                (decile * 0.5) as usize,
                (decile * 2.5) as usize,
                median_idx,
                (decile * 7.5) as usize,
                (decile * 9.5) as usize,
            ];

            let mut reps = Vec::new();
            for &idx in &representative_indices {
                if idx < total_runs {
                    reps.push(final_runs[idx].result.clone());
                }
            }
            reps
        };

        let avg_attack_bonus = calculate_average_attack_bonus(&self.timeline);
        let avg_attack_bonus_int = avg_attack_bonus.round() as i32;
        let party_slots = assign_party_slots(&self.players, avg_attack_bonus_int);

        let output = FullSimulationOutput {
            results: reduced_results,
            analysis: FullAnalysisOutput {
                overall,
                encounters: encounters_analysis,
                average_monster_attack_bonus: avg_attack_bonus,
                party_slots,
            },
            first_run_events: if median_run_events.is_empty() {
                final_runs[total_runs / 2].events.clone()
            } else {
                median_run_events
            },
        };

        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        serde::Serialize::serialize(&output, &serializer)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
    }
}
