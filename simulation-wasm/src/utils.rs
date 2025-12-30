//! Utility functions for simulation results and data transformation

use crate::model::SimulationResult;

/// Reduces memory usage of a SimulationResult by keeping only the first and last rounds
/// of each encounter, which are sufficient for decile analysis and visualization.
///
/// This is used to minimize memory footprint when storing simulation runs for later analysis.
/// For most visualization purposes, only the initial state (round 0) and final state are needed.
pub fn summarize_result(mut result: SimulationResult) -> SimulationResult {
    for encounter in &mut result.encounters {
        if encounter.rounds.len() > 2 {
            let first = encounter.rounds.first().cloned().unwrap();
            let last = encounter.rounds.last().cloned().unwrap();
            encounter.rounds = vec![first, last];
        }
    }
    result
}
