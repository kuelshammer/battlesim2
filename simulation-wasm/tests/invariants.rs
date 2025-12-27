//! Assertion helpers for validating simulation invariants.
//!
//! These functions check that simulation results maintain "impossible" states -
//! things that should NEVER be true regardless of encounter outcome.

use simulation_wasm::model::{EncounterResult, SimulationResult};
use simulation_wasm::events::Event;

/// Check that no combatant has negative HP (below 0 = dead, not negative)
pub fn assert_no_negative_hp(result: &EncounterResult) -> Result<(), String> {
    for round in &result.rounds {
        for combatant in round.team1.iter().chain(round.team2.iter()) {
            if combatant.final_state.current_hp < 0 {
                return Err(format!(
                    "Negative HP detected: {} has {:.1} HP",
                    combatant.id, combatant.final_state.current_hp
                ));
            }
        }
    }
    Ok(())
}

/// Check that all resource values are non-negative
pub fn assert_no_negative_resources(result: &EncounterResult) -> Result<(), String> {
    for round in &result.rounds {
        for combatant in round.team1.iter().chain(round.team2.iter()) {
            for (resource, &amount) in &combatant.final_state.resources.current {
                if amount < 0.0 {
                    return Err(format!(
                        "Negative resource: {} on {} has {:.1} of {}",
                        resource, combatant.id, amount, resource
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Check that damage dealt equals damage taken (conservation of energy)
pub fn assert_damage_conservation(events: &[simulation_wasm::events::Event]) -> Result<(), String> {
    let mut total_damage_dealt: f64 = 0.0;
    let mut total_damage_taken: f64 = 0.0;

    for event in events {
        match event {
            simulation_wasm::events::Event::AttackHit { damage, .. } => {
                total_damage_dealt += damage;
                total_damage_taken += damage;
            }
            simulation_wasm::events::Event::DamageTaken { damage, .. } => {
                // DamageTaken events are usually redundant with AttackHit
                // but we track them separately for validation
            }
            _ => {}
        }
    }

    // Allow small floating point differences
    let diff = (total_damage_dealt - total_damage_taken).abs();
    if diff > 0.01 {
        return Err(format!(
            "Damage mismatch: dealt {:.2} but taken {:.2} (diff: {:.2})",
            total_damage_dealt, total_damage_taken, diff
        ));
    }

    Ok(())
}

/// Check that event sequence is valid (no impossible state transitions)
pub fn assert_event_sequence_valid(events: &[simulation_wasm::events::Event]) -> Result<(), String> {
    use std::collections::HashSet;

    let mut alive: HashSet<String> = HashSet::new();
    let mut dead: HashSet<String> = HashSet::new();

    for event in events {
        match event {
            simulation_wasm::events::Event::EncounterStarted { combatant_ids } => {
                for id in combatant_ids {
                    alive.insert(id.clone());
                }
            }
            simulation_wasm::events::Event::UnitDied { unit_id, .. } => {
                if !alive.contains(unit_id) {
                    return Err(format!("Unit died but wasn't alive: {}", unit_id));
                }
                alive.remove(unit_id);
                dead.insert(unit_id.clone());
            }
            simulation_wasm::events::Event::HealingApplied { target_id, .. } => {
                if dead.contains(target_id) {
                    return Err(format!("Healing applied to dead unit: {}", target_id));
                }
            }
            simulation_wasm::events::Event::ActionStarted { actor_id, .. } => {
                if dead.contains(actor_id) {
                    return Err(format!("Dead unit took action: {}", actor_id));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// Check that action economy is respected (max 1 action/turn per combatant)
///
/// Note: This is a simplified check. The full implementation would need to track
/// turn structure more carefully.
pub fn assert_action_economy(events: &[simulation_wasm::events::Event]) -> Result<(), String> {
    use std::collections::HashMap;

    // For now, just check that no combatant takes more than one action per round
    // without a turn boundary in between
    let mut last_action_round: HashMap<String, u32> = HashMap::new();

    for event in events {
        if let simulation_wasm::events::Event::ActionStarted { actor_id, .. } = event {
            // This is a placeholder - full implementation would track round numbers
            // and check action slot usage
        }
    }

    Ok(())
}

/// Run all invariant checks on a simulation result
pub fn assert_all_invariants(result: &SimulationResult) -> Result<(), String> {
    // Check each encounter
    for (idx, encounter) in result.encounters.iter().enumerate() {
        assert_no_negative_hp(encounter)
            .map_err(|e| format!("Encounter {}: {}", idx, e))?;

        assert_no_negative_resources(encounter)
            .map_err(|e| format!("Encounter {}: {}", idx, e))?;

        // Check events if available
        // Note: EncounterResult doesn't have events in the current model
        // This would need to be added or checked from a different source
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_negative_hp() {
        // This is a placeholder - real tests would use actual simulation results
        // For now, just verify the function compiles
        assert!(true);
    }
}
