//! Balancer orchestration module
//!
//! This module contains auto-balance and encounter adjustment logic
//! extracted from wasm_api.rs, providing pure Rust functions for
//! encounter balancing.

use crate::model::{Creature, TimelineStep};

/// Result structure for auto-adjustment operations
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAdjustmentResult {
    pub monsters: Vec<Creature>,
    pub analysis: crate::decile_analysis::AggregateOutput,
}

/// Run auto-balance encounter adjustment
///
/// This function coordinates the encounter balancing process by:
/// 1. Running simulations with current monster setup
/// 2. Analyzing difficulty through decile analysis
/// 3. Using the AutoBalancer to optimize monster levels/difficulty
pub fn run_auto_adjust_encounter_orchestration(
    players: Vec<Creature>,
    monsters: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    encounter_index: usize,
) -> Result<AutoAdjustmentResult, Box<dyn std::error::Error>> {
    let balancer = crate::auto_balancer::AutoBalancer::new();
    let (optimized_monsters, analysis) =
        balancer.balance_encounter(players, monsters, timeline, encounter_index);

    Ok(AutoAdjustmentResult {
        monsters: optimized_monsters,
        analysis,
    })
}
