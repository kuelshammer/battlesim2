//! Simulation orchestration module
//!
//! This module contains complex simulation logic extracted from wasm_api.rs,
//! providing pure Rust functions for simulation orchestration.

use crate::aggregation::calculate_score;
use crate::model::{Creature, SimulationResult, SimulationRun, TimelineStep};
use crate::orchestration::runners;
use crate::sorting::{assign_party_slots, calculate_average_attack_bonus};
use js_sys::Function;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Output structure for full simulation with callback
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullSimulationOutput {
    pub results: Vec<SimulationResult>,
    pub analysis: FullAnalysisOutput,
    pub first_run_events: Vec<crate::events::Event>,
}

/// Output structure for analysis
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullAnalysisOutput {
    pub overall: crate::decile_analysis::AggregateOutput,
    pub encounters: Vec<crate::decile_analysis::AggregateOutput>,
    #[serde(rename = "averageMonsterAttackBonus")]
    pub average_monster_attack_bonus: f64,
    #[serde(rename = "partySlots")]
    pub party_slots: Vec<crate::sorting::PlayerSlot>,
}

/// Run a three-phase simulation with progress callback support
///
/// This implements the complex three-phase simulation logic:
/// 1. Survey Pass: Run lightweight simulations to gather statistics
/// 2. Selection: Identify interesting seeds based on survey results
/// 3. Deep Dive: Run detailed simulations for interesting seeds
pub fn run_simulation_with_callback_orchestration(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    callback: &Function,
) -> Result<FullSimulationOutput, Box<dyn std::error::Error>> {
    let iterations = iterations.max(100);

    // Check memory guardrails
    if crate::memory_guardrails::should_force_lightweight_mode(iterations) {
        let msg = crate::memory_guardrails::get_lightweight_mode_message(iterations);
        web_sys::console::warn_1(&msg.into());
    }

    // Use three-phase simulation
    let sim = runners::ThreePhaseSimulation::new(players.clone(), timeline.clone(), iterations);

    // Phase 1: Survey Pass
    let batch_size = (iterations / 20).max(1);
    let (summarized_results, lightweight_runs) =
        runners::run_survey_with_progress(&sim, callback, batch_size);

    // Phase 2: Selection
    let interesting_seeds = sim.run_selection(&lightweight_runs);
    let median_seed = sim.find_median_seed(&lightweight_runs);

    // Phase 3: Deep Dive with progress
    let seed_to_events =
        runners::run_deep_dive_with_progress(&sim, &interesting_seeds, callback, iterations);
    let median_run_events = seed_to_events
        .get(&median_seed)
        .cloned()
        .unwrap_or_default();

    // Combine and analyze
    let mut final_runs =
        runners::combine_results_with_events(summarized_results, lightweight_runs, &seed_to_events);
    let sr_count = timeline
        .iter()
        .filter(|s| matches!(s, TimelineStep::ShortRest(_)))
        .count();

    let overall = crate::decile_analysis::run_decile_analysis_with_logs(
        &mut final_runs,
        "Current Scenario",
        players.len(),
        sr_count,
    );

    let num_encounters = final_runs
        .first()
        .map(|r| r.result.encounters.len())
        .unwrap_or(0);
    let encounters_analysis: Vec<_> = (0..num_encounters)
        .map(|i| {
            crate::decile_analysis::run_encounter_analysis_with_logs(
                &mut final_runs,
                i,
                &format!("Encounter {}", i + 1),
                players.len(),
                sr_count,
            )
        })
        .collect();

    // Sort and extract representative results
    final_runs.sort_by(|a, b| {
        calculate_score(&a.result)
            .partial_cmp(&calculate_score(&b.result))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let reduced_results = runners::extract_representative_results(&final_runs);
    let avg_attack_bonus = calculate_average_attack_bonus(&timeline);
    let party_slots = assign_party_slots(&players, avg_attack_bonus.round() as i32);

    let output = FullSimulationOutput {
        results: reduced_results,
        analysis: FullAnalysisOutput {
            overall,
            encounters: encounters_analysis,
            average_monster_attack_bonus: avg_attack_bonus,
            party_slots,
        },
        first_run_events: if median_run_events.is_empty() {
            final_runs[final_runs.len() / 2].events.clone()
        } else {
            median_run_events
        },
    };

    Ok(output)
}

/// Run skyline analysis on simulation results
pub fn run_skyline_analysis_orchestration(
    mut results: Vec<SimulationResult>,
    party_size: usize,
    encounter_index: Option<usize>,
) -> Result<crate::percentile_analysis::SkylineAnalysis, Box<dyn std::error::Error>> {
    // Sort by appropriate score
    if let Some(idx) = encounter_index {
        results.sort_by(
            |a, b| match (a.encounters.get(idx), b.encounters.get(idx)) {
                (Some(ea), Some(eb)) => crate::aggregation::calculate_encounter_score(ea)
                    .partial_cmp(&crate::aggregation::calculate_encounter_score(eb))
                    .unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        );
    } else {
        results.sort_by(|a, b| {
            calculate_score(a)
                .partial_cmp(&calculate_score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let result_refs: Vec<&SimulationResult> = results.iter().collect();
    let analysis =
        crate::percentile_analysis::run_skyline_analysis(&result_refs, party_size, encounter_index);

    Ok(analysis)
}

/// Run decile analysis on simulation results
pub fn run_decile_analysis_orchestration(
    mut results: Vec<SimulationResult>,
    scenario_name: &str,
    party_size: usize,
) -> Result<FullAnalysisOutput, Box<dyn std::error::Error>> {
    results.sort_by(|a, b| {
        calculate_score(a)
            .partial_cmp(&calculate_score(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let actual_party_size = results
        .first()
        .and_then(|r| r.encounters.first())
        .and_then(|e| e.rounds.first())
        .map(|r| r.team1.len())
        .unwrap_or(0);

    let sr_count = results
        .first()
        .map(|r| {
            r.encounters
                .iter()
                .filter(|e| e.rounds.len() == 1 && e.rounds[0].team2.is_empty())
                .count()
        })
        .unwrap_or(0);

    let overall = crate::decile_analysis::run_decile_analysis(
        &results,
        scenario_name,
        actual_party_size,
        sr_count,
    );
    let num_encounters = results.first().map(|r| r.encounters.len()).unwrap_or(0);

    let encounters: Vec<_> = (0..num_encounters)
        .map(|i| {
            crate::decile_analysis::run_encounter_analysis(
                &results,
                i,
                &format!("Encounter {}", i + 1),
                actual_party_size,
                sr_count,
            )
        })
        .collect();

    Ok(FullAnalysisOutput {
        overall,
        encounters,
        average_monster_attack_bonus: 5.0,
        party_slots: Vec::new(),
    })
}
