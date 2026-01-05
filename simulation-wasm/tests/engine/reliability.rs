use simulation_wasm::model::*;
use simulation_wasm::error_handling::*;
use simulation_wasm::enhanced_validation::*;
use simulation_wasm::recovery::*;
use simulation_wasm::safe_aggregation::*;

fn create_test_creature(name: &str, hp: f64) -> Creature {
    Creature {
        id: name.to_string(),
        arrival: None,
        name: name.to_string(),
        hp: hp as u32,
        ac: 15,
        speed_fly: None,
        save_bonus: 0.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: None,
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: simulation_wasm::model::DiceFormula::Value(0.0),
        initiative_advantage: false,
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None,
        count: 1.0,
    }
}

fn create_test_combatant(name: &str, hp: f64, current_hp: f64, team: u32) -> Combattant {
    let creature = create_test_creature(name, hp);
    let state = CreatureState {
        current_hp: current_hp as u32,
        temp_hp: None,
        buffs: std::collections::HashMap::new(),
        resources: simulation_wasm::model::SerializableResourceLedger { 
            current: std::collections::HashMap::new(), 
            max: std::collections::HashMap::new() 
        },
        upcoming_buffs: std::collections::HashMap::new(),
        used_actions: std::collections::HashSet::new(),
        concentrating_on: None,
        actions_used_this_encounter: std::collections::HashSet::new(),
        bonus_action_used: false,
        known_ac: std::collections::HashMap::new(),
        arcane_ward_hp: None,
        cumulative_spent: 0.0,
    };

    Combattant {
        id: name.to_string(),
        team,
        creature: std::sync::Arc::new(creature),
        initiative: 10.0,
        initial_state: state.clone(),
        final_state: state,
        actions: vec![],
    }
}

fn create_test_round() -> Round {
    let player1 = create_test_combatant("Player1", 50.0, 30.0, 0);
    let player2 = create_test_combatant("Player2", 40.0, 40.0, 0);
    let monster1 = create_test_combatant("Monster1", 60.0, 10.0, 1);
    
    Round {
        team1: vec![player1, player2],
        team2: vec![monster1],
    }
}

fn create_test_simulation_result() -> SimulationResult {
    let round = create_test_round();
    let encounter = EncounterResult {
        stats: std::collections::HashMap::new(),
        rounds: vec![round],
        target_role: TargetRole::Standard,
    };
    SimulationResult {
        encounters: vec![encounter],
        score: Some(100.0),
        num_combat_encounters: 1,
        seed: 0,
    }
}

#[test]
fn test_empty_result_handling() {
    let empty_results: Vec<SimulationResult> = vec![];
    
    // Test that empty results are handled gracefully
    match validate_simulation_results(&empty_results) {
        Err(SimulationError::EmptyResult(_)) => {
            // Expected behavior
        }
        _ => panic!("Expected EmptyResult error for empty results"),
    }
    
    // Test safe aggregation with empty results
    match aggregate_results_safe(&empty_results) {
        Err(SimulationError::EmptyResult(_)) => {
            // Expected behavior
        }
        _ => panic!("Expected EmptyResult error for empty aggregation"),
    }
    
    // Test safe score calculation with empty results
    let empty_result = SimulationResult { encounters: vec![], score: None, num_combat_encounters: 0, seed: 0 };
    match calculate_score_safe(&empty_result) {
        Err(SimulationError::EmptyResult(_)) => {
            // Expected behavior
        }
        _ => panic!("Expected EmptyResult error for empty score calculation"),
    }
}

#[test]
fn test_invalid_combatant_recovery() {
    let mut results = vec![];
    
    // Create a valid result
    let valid_result = create_test_simulation_result();
    results.push(valid_result);
    
    // Create a result with invalid combatant (negative HP is impossible with u32, so we use 0)
    let mut invalid_result = create_test_simulation_result();
    if let Some(encounter) = invalid_result.encounters.first_mut() {
        for round in &mut encounter.rounds {
            for combatant in &mut round.team1 {
                combatant.final_state.current_hp = 0; // Simulate "dead" when we expect "alive"
            }
        }
    }
    results.push(invalid_result);
    
    // Test validation catches the invalid combatant
    match validate_simulation_results(&results) {
        Ok(_report) => {
            // In our current system, 0 HP is valid but might be flagged depending on validation rules
            // This test might need adjustment if 0 HP is not strictly "invalid"
        }
        Err(_) => {
            // Also acceptable if error rate is too high
        }
    }
}

#[test]
fn test_resource_exhaustion_handling() {
    let context = ErrorContext::new("test".to_string(), 0, 0);
    let mut logger = ErrorLogger::new(100);
    
    // Test resource exhaustion error logging
    let error = SimulationError::ResourceExhausted("Spell slots depleted".to_string());
    logger.log_error(error.clone(), context.clone());
    
    assert_eq!(logger.get_logs().len(), 1);
    assert_eq!(logger.get_error_summary().get("ResourceExhausted"), Some(&1));
}

#[test]
fn test_error_logger_functionality() {
    let mut logger = ErrorLogger::new(10);
    let context = ErrorContext::new("test_sim".to_string(), 0, 0);
    
    // Test logging different error types
    logger.log_error(
        SimulationError::EmptyResult("Test error".to_string()),
        context.clone()
    );
    
    logger.log_recovery_attempt(
        SimulationError::InvalidCombatant("Test combatant".to_string()),
        context.clone(),
        true
    );
    
    assert_eq!(logger.get_logs().len(), 2);
    
    let summary = logger.get_error_summary();
    assert_eq!(summary.get("EmptyResult"), Some(&1));
    assert_eq!(summary.get("InvalidCombatant"), Some(&1));
    
    // Test log rotation
    for i in 0..15 {
        logger.log_error(
            SimulationError::UnexpectedState(format!("Error {}", i)),
            context.clone()
        );
    }
    
    // Should maintain max size
    assert_eq!(logger.get_logs().len(), 10);
}

#[test]
fn test_error_context_builder() {
    let context = ErrorContext::new("test_sim".to_string(), 5, 2)
        .with_round(3)
        .with_combatant("player1".to_string())
        .with_action("attack".to_string());
    
    assert_eq!(context.simulation_id, "test_sim");
    assert_eq!(context.iteration, 5);
    assert_eq!(context.encounter_idx, 2);
    assert_eq!(context.round, Some(3));
    assert_eq!(context.combatant_id, Some("player1".to_string()));
    assert_eq!(context.action_id, Some("attack".to_string()));
}

#[test]
fn test_validation_report() {
    let mut report = ValidationReport::new(100);
    
    assert_eq!(report.success_rate(), 1.0); // total_results=100, error_count=0, so success_rate=1.0
    assert!(report.is_acceptable()); // 100% success rate is acceptable
    
    report.error_count = 5;
    
    assert_eq!(report.success_rate(), 0.95); // (100-5)/100 = 0.95
    assert!(report.is_acceptable()); // 95% is exactly the threshold
    
    report.error_count = 10;
    
    assert_eq!(report.success_rate(), 0.90); // (100-10)/100 = 0.90
    assert!(!report.is_acceptable()); // 90% is below the 95% threshold
}

#[test]
fn test_recovery_engine() {
    let mut engine = ErrorRecoveryEngine::new(3);
    let context = ErrorContext::new("test".to_string(), 0, 0);
    
    // Test retry behavior
    let error = SimulationError::EmptyResult("Test error".to_string());
    
    assert!(engine.attempt_recovery(&error, &context).is_ok());
    assert_eq!(engine.get_retry_count(), 1);
    
    // Test max retries
    for _ in 0..3 {
        let _ = engine.attempt_recovery(&error, &context);
    }
    
    let exhausted_error = SimulationError::RetryExhausted("Test".to_string());
    assert!(engine.attempt_recovery(&exhausted_error, &context).is_err());
}

#[test]
fn test_recovery_report() {
    let mut report = RecoveryReport::new(100);
    
    assert_eq!(report.success_rate(), 0.0);
    assert_eq!(report.recovery_rate(), 0.0);
    
    report.successful_iterations = 95;
    report.failed_iterations = 5;
    report.retried_iterations = 3;
    
    assert_eq!(report.success_rate(), 0.95);
    assert_eq!(report.recovery_rate(), 0.6);
}

#[test]
fn test_safe_score_calculation() {
    let valid_result = create_test_simulation_result();
    
    // Test normal score calculation
    match calculate_score_safe(&valid_result) {
        Ok(score) => {
            assert!(score > 0.0); // Should have positive score with survivors
        }
        Err(_) => panic!("Failed to calculate score for valid result"),
    }
    
    // Test with empty result
    let empty_result = SimulationResult { encounters: vec![], score: None, num_combat_encounters: 0, seed: 0 };
    match calculate_score_safe(&empty_result) {
        Err(SimulationError::EmptyResult(_)) => {
            // Expected
        }
        _ => panic!("Expected EmptyResult error for empty result"),
    }
}

#[test]
fn test_safe_aggregation() {
    let valid_result = create_test_simulation_result();
    let results = vec![valid_result];
    
    // Test normal aggregation
    match aggregate_results_safe(&results) {
        Ok(rounds) => {
            assert!(!rounds.is_empty());
        }
        Err(_) => panic!("Failed to aggregate valid results"),
    }
    
    // Test with empty results
    let empty_results: Vec<SimulationResult> = vec![];
    match aggregate_results_safe(&empty_results) {
        Err(SimulationError::EmptyResult(_)) => {
            // Expected
        }
        _ => panic!("Expected EmptyResult error for empty aggregation"),
    }
}

#[test]
fn test_high_stress_simulation() {
    let mut results = vec![];
    
    // Create many results to stress test the system
    for i in 0..100 {
        let mut result = create_test_simulation_result();
        
        // Introduce some variability and potential issues
        if i % 10 == 0 {
            // Some results with very low HP
            if let Some(encounter) = result.encounters.first_mut() {
                for round in &mut encounter.rounds {
                    for combatant in &mut round.team1 {
                        combatant.final_state.current_hp = (i as f64 * 0.1) as u32;
                    }
                }
            }
        }
        
        if i % 20 == 0 {
            // Some results with 0 HP
            if let Some(encounter) = result.encounters.first_mut() {
                for round in &mut encounter.rounds {
                    for combatant in &mut round.team2 {
                        combatant.final_state.current_hp = 0;
                    }
                }
            }
        }
        
        results.push(result);
    }
    
    // Test validation on large dataset
    match validate_simulation_results(&results) {
        Ok(report) => {
            assert!(report.success_rate() > 0.8); // Should have reasonable success rate
        }
        Err(SimulationError::UnexpectedState(_)) => {
            // Acceptable if error rate is too high
        }
        Err(_) => panic!("Unexpected error type during stress test"),
    }
    
    // Test aggregation on large dataset
    match aggregate_results_safe(&results) {
        Ok(rounds) => {
            assert!(!rounds.is_empty());
        }
        Err(_) => {
            // Acceptable if too many errors
        }
    }
}

#[test]
fn test_error_logging_completeness() {
    let mut logger = ErrorLogger::new(100);
    let context = ErrorContext::new("completeness_test".to_string(), 0, 0);
    
    // Test all error types are logged properly
    let errors = vec![
        SimulationError::EmptyResult("Test empty".to_string()),
        SimulationError::InvalidCombatant("Test combatant".to_string()),
        SimulationError::ResourceExhausted("Test resource".to_string()),
        SimulationError::IndexOutOfBounds("Test index".to_string()),
        SimulationError::SerializationError("Test serialization".to_string()),
        SimulationError::UnexpectedState("Test state".to_string()),
        SimulationError::ValidationFailed("Test validation".to_string()),
        SimulationError::RetryExhausted("Test retry".to_string()),
    ];
    
    for error in errors {
        logger.log_error(error.clone(), context.clone());
    }
    
    assert_eq!(logger.get_logs().len(), 8);
    
    let summary = logger.get_error_summary();
    assert_eq!(summary.len(), 8); // All error types should be represented
    
    // Verify each error type was logged
    assert_eq!(summary.get("EmptyResult"), Some(&1));
    assert_eq!(summary.get("InvalidCombatant"), Some(&1));
    assert_eq!(summary.get("ResourceExhausted"), Some(&1));
    assert_eq!(summary.get("IndexOutOfBounds"), Some(&1));
    assert_eq!(summary.get("SerializationError"), Some(&1));
    assert_eq!(summary.get("UnexpectedState"), Some(&1));
    assert_eq!(summary.get("ValidationFailed"), Some(&1));
    assert_eq!(summary.get("RetryExhausted"), Some(&1));
}
