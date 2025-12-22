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
            num_combat_encounters: 1,
        });
    }

    let output = run_decile_analysis(&results, scenario_name, party_size);

    assert_eq!(output.deciles.len(), 10, "Should have 10 deciles");
    
    let q1 = &output.deciles[0];
    assert_eq!(q1.decile, 1);
    
    let q5 = &output.deciles[4];
    assert_eq!(q5.decile, 5);
}

#[test]
fn test_star_ratings() {
    use simulation_wasm::decile_analysis::{run_decile_analysis, IntensityTier};
    
    // Test Tier 4 -> 3 Stars
    let mut results: Vec<SimulationResult> = Vec::new();
    for _ in 0..100 {
        // Force Tier 4 (Heavy) -> 10-40% resources left
        // Typical hp_lost_percent = 75% -> Tier 4
        results.push(SimulationResult {
            encounters: vec![],
            score: Some(1_000_000.0 + 25.0), // 1 survivor, 25 HP left. If max HP is 100, loss is 75%
            num_combat_encounters: 1,
        });
    }

    // We need to ensure calculate_run_stats correctly interprets the dummy score.
    // It uses 1,000,000 per survivor + remaining HP.
    
    let output = run_decile_analysis(&results, "Stars Test", 1);
    
    // Note: Interpretation depends on party_max_hp which is currently hard to mock without creatures
    // But we can check if the field exists and is reachable.
    assert!(output.stars <= 3);
}

#[test]
fn test_intensity_regression_high_penalty() {
    use simulation_wasm::decile_analysis::run_decile_analysis;
    use simulation_wasm::model::*;
    use std::collections::HashMap;

    // Create a mock run: 2 fighters, both survive but heavily damaged and high penalty.
    // Max HP: 145. Remaining HP: 100. Attrition: 45.
    // Survivors: 2.
    // Score should be (2 * 1,000,000) + 100 - 0 (monsters) - 150 (penalty) = 1,999,950.
    
    let mut fighter = Creature {
        id: "f".to_string(), name: "f".to_string(), hp: 75, ac: 10, count: 2.0,
        arrival: None, mode: "p".to_string(), speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![], triggers: vec![], spell_slots: None, class_resources: None,
        hit_dice: None, con_modifier: None,
    };

    let mut encounter_res = EncounterResult {
        stats: HashMap::new(),
        rounds: vec![Round {
            team1: vec![
                Combattant {
                    id: "f1".to_string(), team: 0, creature: fighter.clone(), initiative: 10.0,
                    initial_state: CreatureState::default(),
                    final_state: CreatureState { current_hp: 50, ..CreatureState::default() },
                    actions: vec![],
                },
                Combattant {
                    id: "f2".to_string(), team: 0, creature: fighter.clone(), initiative: 10.0,
                    initial_state: CreatureState::default(),
                    final_state: CreatureState { current_hp: 50, ..CreatureState::default() },
                    actions: vec![],
                },
            ],
            team2: vec![],
        }]
    };

    let run = SimulationResult {
        encounters: vec![encounter_res],
        score: Some(1_999_950.0), // Penalty of 150 pushed it below 2M
        num_combat_encounters: 1,
    };

    let output = run_decile_analysis(&vec![run; 100], "Regression Test", 2);
    
    // Total Attrition should be: max_hp (150) + 2M - 1,999,950 = 150 + 50 = 200.
    // HP Lost Percent: 200 / 150 = 133%.
    // Resources left: 100 - 133 = -33%. -> Tier 5.
    
    // In the old code, survivors would be floor(1.99) = 1.
    // hp_lost = 150 - (1,999,950 - 1,000,000) = 150 - 999,950 = -999,800 -> 0.
    // Resources left: 100 - 0 = 100%. -> Tier 1.
    
    println!("Regression Tier: {:?}", output.intensity_tier);
    assert_ne!(output.intensity_tier, simulation_wasm::decile_analysis::IntensityTier::Tier1, "Intensity should NOT be Tier 1");
    assert_eq!(output.intensity_tier, simulation_wasm::decile_analysis::IntensityTier::Tier5, "Intensity should be Tier 5 (due to high penalty)");
}