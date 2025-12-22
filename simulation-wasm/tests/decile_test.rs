use simulation_wasm::decile_analysis::run_decile_analysis;
use simulation_wasm::model::{Creature, Encounter, SimulationResult};
use std::collections::HashMap;

#[test]
fn test_decile_analysis_median_run_visualization() {
    let mut players: Vec<Creature> = Vec::new();

    // Create a dummy result set
    let scenario_name = "Test Scenario";
    let party_size = 1;

    // The run_decile_analysis function expects sorted results.
    // We create dummy results with varying scores.
    let mut results: Vec<SimulationResult> = Vec::new();
    for i in 0..100 {
        results.push(SimulationResult {
            encounters: vec![],
            score: Some(i as f64),
        });
    }

    let output = run_decile_analysis(&results, scenario_name, party_size);

    assert_eq!(output.deciles.len(), 10, "Should have 10 deciles");
    
    let q1 = &output.deciles[0];
    assert_eq!(q1.decile, 1);
    
    let q5 = &output.deciles[4];
    assert_eq!(q5.decile, 5);
}