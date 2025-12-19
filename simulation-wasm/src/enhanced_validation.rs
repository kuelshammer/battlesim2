use crate::model::*;
use crate::error_handling::{SimulationError, ErrorContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidHP(String, f64),           // creature name, hp value
    InvalidActionCost(String, String, String), // creature, action, error
    EmptyCombatantList,
    NegativeDamage(String),
    InfiniteLoopRisk(String),
    MemoryExhaustionRisk(usize),
    InvalidTargetSelection(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total_results: usize,
    pub valid_results: usize,
    pub error_count: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<(usize, SimulationError)>,
}

impl ValidationReport {
    pub fn new(total_results: usize) -> Self {
        Self {
            total_results,
            valid_results: 0,
            error_count: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, iteration: usize, error: SimulationError) {
        self.error_count += 1;
        self.errors.push((iteration, error));
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_results == 0 {
            0.0
        } else {
            (self.total_results - self.error_count) as f64 / self.total_results as f64
        }
    }
    
    pub fn is_acceptable(&self) -> bool {
        self.success_rate() >= 0.95 && self.error_count <= self.total_results / 20 // Max 5% errors
    }
}

pub fn validate_simulation_results(results: &[SimulationResult]) -> Result<ValidationReport, SimulationError> {
    let mut report = ValidationReport::new(results.len());
    
    // Check for empty results
    if results.is_empty() {
        return Err(SimulationError::EmptyResult("No simulation results provided".to_string()));
    }
    
    // Validate each result structure
    for (i, result) in results.iter().enumerate() {
        match validate_single_result(result, i) {
            Ok(()) => {
                report.valid_results += 1;
            }
            Err(error) => {
                report.add_error(i, error);
            }
        }
    }
    
    // Check if success rate meets minimum threshold
    if report.success_rate() < 0.95 {
        return Err(SimulationError::UnexpectedState(
            format!("Success rate {:.1}% below 95% threshold", report.success_rate() * 100.0)
        ));
    }
    
    // Add warnings for potential issues
    if report.error_count > 0 {
        report.add_warning(format!(
            "{} out of {} simulations had errors ({:.1}% error rate)",
            report.error_count,
            report.total_results,
            (report.error_count as f64 / report.total_results as f64) * 100.0
        ));
    }
    
    Ok(report)
}

fn validate_single_result(result: &SimulationResult, iteration: usize) -> Result<(), SimulationError> {
    let _context = ErrorContext::new("validation".to_string(), iteration, 0);
    
    // Check if result is empty
    if result.is_empty() {
        return Err(SimulationError::EmptyResult(
            format!("Simulation result {} is empty", iteration)
        ));
    }
    
    // Validate each encounter
    for (encounter_idx, encounter) in result.iter().enumerate() {
        validate_encounter(encounter, iteration, encounter_idx)?;
    }
    
    Ok(())
}

fn validate_encounter(encounter: &EncounterResult, iteration: usize, encounter_idx: usize) -> Result<(), SimulationError> {
    let _context = ErrorContext::new("validation".to_string(), iteration, encounter_idx);
    
    // Check if encounter has rounds
    if encounter.rounds.is_empty() {
        return Err(SimulationError::EmptyResult(
            format!("Encounter {} has no rounds", encounter_idx)
        ));
    }
    
    // Validate each round
    for (round_idx, round) in encounter.rounds.iter().enumerate() {
        validate_round(round, iteration, encounter_idx, round_idx)?;
    }
    
    Ok(())
}

fn validate_round(round: &Round, iteration: usize, encounter_idx: usize, round_idx: usize) -> Result<(), SimulationError> {
    let context = ErrorContext::new("validation".to_string(), iteration, encounter_idx)
        .with_round(round_idx);
    
    // Check if both teams have combatants
    if round.team1.is_empty() && round.team2.is_empty() {
        return Err(SimulationError::EmptyResult(
            format!("Round {} has no combatants", round_idx)
        ));
    }
    
    // Validate team 1 (players)
    for combatant in &round.team1 {
        validate_combatant(combatant, &context)?;
    }
    
    // Validate team 2 (monsters)
    for combatant in &round.team2 {
        validate_combatant(combatant, &context)?;
    }
    
    Ok(())
}

fn validate_combatant(combatant: &Combattant, _context: &ErrorContext) -> Result<(), SimulationError> {
    // Check if combatant has valid HP (creature.hp)
    if combatant.creature.hp == 0 {
        return Err(SimulationError::InvalidCombatant(
            format!("Combatant '{}' has invalid base HP: {}", combatant.creature.name, combatant.creature.hp)
        ));
    }
    
    // Final state current_hp cannot be negative due to u32, but it can be 0.
    // No specific check needed for "extremely negative HP" as it's impossible with u32.

    // Validate actions
    for action in &combatant.actions {
        if action.targets.is_empty() && combatant.final_state.current_hp > 0 {
            // Living combatant with actions but no targets might be an issue
            // This is a warning, not an error
        }
    }
    
    Ok(())
}

pub fn validate_scenario_for_edge_cases(players: &[Creature], encounters: &[Encounter]) -> Result<(), Vec<String>> {
    let mut warnings = Vec::new();
    
    // Check for very high damage values that might cause overflow
    for creature in players {
        for action in &creature.actions {
            if let Some(damage) = extract_max_damage(action) {
                if damage > 1000.0 {
                    warnings.push(format!(
                        "Creature '{}' action '{}' has very high damage: {}",
                        creature.name, action.base().id, damage
                    ));
                }
            }
        }
    }
    
    // Check for large numbers of combatants that might cause memory issues
    let total_combatants: i32 = players.iter().map(|p| p.count as i32).sum::<i32>()
        + encounters.iter()
            .flat_map(|e| e.monsters.iter().map(|m| m.count as i32))
            .sum::<i32>();
    
    if total_combatants > 50 {
        warnings.push(format!(
            "Large number of combatants ({}): may cause performance issues",
            total_combatants
        ));
    }
    
    // Check for potential infinite loops in action resolution
    for creature in players {
        for action in &creature.actions {
            if has_potential_infinite_loop(action) {
                warnings.push(format!(
                    "Creature '{}' action '{}' may cause infinite loops",
                    creature.name, action.base().id
                ));
            }
        }
    }
    
    if warnings.is_empty() {
        Ok(())
    } else {
        Err(warnings)
    }
}

fn extract_max_damage(_action: &Action) -> Option<f64> {
    // Simplified implementation - return conservative estimate
    Some(50.0)
}

fn has_potential_infinite_loop(_action: &Action) -> bool {
    // Simplified implementation - always return false for now
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(100);
        
        assert_eq!(report.success_rate(), 1.0);
        // With 0 errors out of 100 total, this should be acceptable
        // error_count (0) <= total_results / 20 (5) = true
        // success_rate (1.0) >= 0.95 = true
        assert!(report.is_acceptable());
        
        report.valid_results = 95;
        report.error_count = 5;
        
        assert_eq!(report.success_rate(), 0.95);
        assert!(report.is_acceptable());
        
        report.error_count = 10;
        
        assert_eq!(report.success_rate(), 0.90);
        assert!(!report.is_acceptable());
    }
    
    #[test]
    fn test_empty_result_validation() {
        let results = vec![];
        
        match validate_simulation_results(&results) {
            Err(SimulationError::EmptyResult(_)) => {
                // Expected
            }
            _ => panic!("Expected EmptyResult error"),
        }
    }
}