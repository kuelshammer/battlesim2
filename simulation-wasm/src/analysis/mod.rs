//! Modular analysis system for encounter simulation results
//!
//! This module provides a clean separation of concerns:
//! - **types**: Core data structures (Archetypes, Vitals, Tiers)
//! - **narrative**: Game design logic (The "Director" - scoring, pacing, labeling)
//! - **statistics**: Statistical math (The "Mathematician" - percentiles, aggregation)
//! - **visualization**: UI data mapping (The "Presenter" - frontend representation)

pub mod narrative;
pub mod statistics;
pub mod types;
pub mod visualization;

// Re-export public API for backward compatibility
pub use statistics::*;
pub use types::*;

/// Run decile analysis on simulation results
pub fn run_decile_analysis(
    results: &[crate::model::SimulationResult],
    scenario_name: &str,
    party_size: usize,
    sr_count: usize,
) -> AggregateOutput {
    let refs: Vec<&crate::model::SimulationResult> = results.iter().collect();
    analyze_results_internal(
        &refs,
        None,
        scenario_name,
        party_size,
        None,
        sr_count,
        &visualization::extract_combatant_visualization_partial,
    )
}

/// Run decile analysis with event logs
pub fn run_decile_analysis_with_logs(
    runs: &mut [crate::model::SimulationRun],
    scenario_name: &str,
    party_size: usize,
    sr_count: usize,
) -> AggregateOutput {
    let results: Vec<&crate::model::SimulationResult> = runs.iter().map(|r| &r.result).collect();
    analyze_results_internal(
        &results,
        None,
        scenario_name,
        party_size,
        Some(runs),
        sr_count,
        &visualization::extract_combatant_visualization_partial,
    )
}

/// Run full day analysis across all encounters
pub fn run_day_analysis(
    results: &[crate::model::SimulationResult],
    scenario_name: &str,
    party_size: usize,
    sr_count: usize,
) -> AggregateOutput {
    let refs: Vec<&crate::model::SimulationResult> = results.iter().collect();
    analyze_results_internal(
        &refs,
        None,
        scenario_name,
        party_size,
        None,
        sr_count,
        &visualization::extract_combatant_visualization_partial,
    )
}

/// Run analysis for a specific encounter
pub fn run_encounter_analysis(
    results: &[crate::model::SimulationResult],
    encounter_idx: usize,
    scenario_name: &str,
    party_size: usize,
    sr_count: usize,
) -> AggregateOutput {
    let refs: Vec<&crate::model::SimulationResult> = results.iter().collect();
    analyze_results_internal(
        &refs,
        Some(encounter_idx),
        scenario_name,
        party_size,
        None,
        sr_count,
        &visualization::extract_combatant_visualization_partial,
    )
}

/// Run analysis for a specific encounter with event logs
pub fn run_encounter_analysis_with_logs(
    runs: &mut [crate::model::SimulationRun],
    encounter_idx: usize,
    scenario_name: &str,
    party_size: usize,
    sr_count: usize,
) -> AggregateOutput {
    // 1. Sort the runs based on cumulative score up to encounter_idx
    runs.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_cumulative_score(&a.result, encounter_idx);
        let score_b = crate::aggregation::calculate_cumulative_score(&b.result, encounter_idx);
        score_a
            .partial_cmp(&score_b)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.result.seed.cmp(&b.result.seed))
    });

    // 2. Perform analysis using refs
    let refs: Vec<&crate::model::SimulationResult> = runs.iter().map(|r| &r.result).collect();
    let mut output = analyze_results_internal(
        &refs,
        Some(encounter_idx),
        scenario_name,
        party_size,
        Some(runs),
        sr_count,
        &visualization::extract_combatant_visualization_partial,
    );

    // 3. Slice the logs to only include events for this specific encounter
    for log in &mut output.decile_logs {
        *log = visualization::slice_events_for_encounter(log, encounter_idx);
    }

    output
}
