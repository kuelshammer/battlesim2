use simulation_wasm::run_single_event_driven_simulation;
use simulation_wasm::run_survey_pass;
use simulation_wasm::rng;
use crate::common::load_scenario;

#[test]
fn test_two_pass_reproducibility() {
    let (players, timeline) = load_scenario("fast_init_PlayerA_wins.json");
    let base_seed = 12345;
    let iterations = 100; // Minimum enforced
    let expected = 100;

    // 1. Run Survey Pass (Phase 1)
    let survey_runs = run_survey_pass(players.clone(), timeline.clone(), iterations, Some(base_seed));

    assert_eq!(survey_runs.len(), expected, "Should have 100 survey runs (minimum enforced)");

    // 2. Pick specific indices to verify
    // We verify start, middle, and end to ensure RNG offset is correct
    let indices_to_verify = vec![0, 10, 25, 49];

    for &index in &indices_to_verify {
        let lightweight_run = &survey_runs[index];
        let expected_seed = base_seed + index as u64;

        assert_eq!(lightweight_run.seed, expected_seed, "Seed mismatch at index {}", index);

        // 3. Re-run with Full Simulation (Phase 3 equivalent for single run)
        rng::seed_rng(expected_seed);
        
        let (full_result, _events) = run_single_event_driven_simulation(&players, &timeline, false);
        
        // Clear RNG after to be clean
        rng::clear_rng();

        // 4. Compare Metrics
        let full_score = full_result.score.expect("Full simulation should return a score");
        
        // Floating point comparison with epsilon
        let score_diff = (lightweight_run.final_score - full_score).abs();
        assert!(score_diff < 1e-10, 
            "Score mismatch at index {}: Lightweight={}, Full={}", 
            index, lightweight_run.final_score, full_score);

        // B. Survivors
        let full_survivors = full_result.encounters.last()
            .map(|enc| enc.rounds.last().unwrap().team1.iter().filter(|c| c.final_state.current_hp > 0).count())
            .unwrap_or(0);
        
        assert_eq!(lightweight_run.total_survivors, full_survivors,
            "Survivor count mismatch at index {}", index);

        assert!(!full_result.encounters.is_empty(), "Full result should have encounters");
    }
}

#[test]
fn test_reproducibility_complex_mechanics() {
    // Using a scenario with more variables (damage vs precision) to test deeper mechanics
    let (players, timeline) = load_scenario("damage_vs_precision_MonsterB_wins.json");
    let base_seed = 999;
    let iterations = 100; // Minimum enforced
    let expected = 100;

    let survey_runs = run_survey_pass(players.clone(), timeline.clone(), iterations, Some(base_seed));
    assert_eq!(survey_runs.len(), expected);

    for (index, lightweight_run) in survey_runs.iter().enumerate() {
        let expected_seed = base_seed + index as u64;
        
        rng::seed_rng(expected_seed);
        let (full_result, _) = run_single_event_driven_simulation(&players, &timeline, false);
        rng::clear_rng();

        let full_score = full_result.score.unwrap();
        
        assert!( (lightweight_run.final_score - full_score).abs() < 1e-10,
            "Complex Scenario Score mismatch at index {}: LW={}, Full={}",
            index, lightweight_run.final_score, full_score);
    }
}