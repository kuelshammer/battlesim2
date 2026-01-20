//! Utility functions for simulation results and data transformation

use crate::dice;
use crate::model::{Creature, SimulationResult};
use crate::rng; // Import rng module
use std::hash::{Hash, Hasher};

/// Reduces memory usage of a SimulationResult by keeping only the first and last rounds
/// of each encounter, which are sufficient for decile analysis and visualization.
///
/// This is used to minimize memory footprint when storing simulation runs for later analysis.
/// For most visualization purposes, only the initial state (round 0) and final state are needed.
pub fn summarize_result(mut result: SimulationResult, seed: u64) -> SimulationResult {
    result.seed = seed;
    for encounter in &mut result.encounters {
        if encounter.rounds.len() > 2 {
            let first = encounter.rounds.first().cloned();
            let last = encounter.rounds.last().cloned();

            if let (Some(f), Some(l)) = (first, last) {
                encounter.rounds = vec![f, l];
            }
        }
    }
    result
}

// Hashing utilities for deterministic behavior

/// Hashes a f64 value using its bit representation for consistent hashing
pub fn hash_f64<H: Hasher>(val: f64, state: &mut H) {
    val.to_bits().hash(state);
}

/// Hashes an optional f64 value for consistent hashing
pub fn hash_opt_f64<H: Hasher>(val: Option<f64>, state: &mut H) {
    val.map(|v| v.to_bits()).hash(state);
}

// Initiative utilities

/// Rolls initiative for a creature, including advantage handling and bonus calculation
pub fn roll_initiative(c: &Creature) -> f64 {
    let roll = if c.initiative_advantage {
        let r1 = rng::roll_d20();
        let r2 = rng::roll_d20();
        r1.max(r2) as f64
    } else {
        rng::roll_d20() as f64
    };

    let bonus = dice::evaluate(&c.initiative_bonus, 1); // Use dice::evaluate with multiplier 1

    roll + bonus
}
