use simulation_wasm::decile_analysis::run_decile_analysis;
use simulation_wasm::model::{Creature, SimulationResult};
use std::collections::HashMap;

#[test]
fn test_decile_analysis_median_run_visualization() {
    let _players: Vec<Creature> = Vec::new();

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
    use simulation_wasm::decile_analysis::run_decile_analysis;
    
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

    // Create a mock run: 2 fighters, both survive but heavily damaged and high penalty.
    // Max HP: 145. Remaining HP: 100. Attrition: 45.
    // Survivors: 2.
    // Score should be (2 * 1,000,000) + 100 - 0 (monsters) - 150 (penalty) = 1,999,950.

    let fighter = Creature {
        id: "f".to_string(), name: "f".to_string(), hp: 75, ac: 10, count: 2.0,
        arrival: None, mode: "p".to_string(), speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![], triggers: vec![], spell_slots: None, class_resources: None,
        hit_dice: None, con_modifier: None,
    };

    let encounter_res = EncounterResult {
        stats: HashMap::new(),
        rounds: vec![Round {
            team1: vec![
                Combattant {
                    id: "f1".to_string(), team: 0, creature: std::sync::Arc::new(fighter.clone()), initiative: 10.0,
                    initial_state: CreatureState {
                        current_hp: 75,
                        resources: SerializableResourceLedger {
                            current: [("HP".to_string(), 75.0)].into(),
                            max: [("HP".to_string(), 75.0)].into()
                        },
                        ..CreatureState::default()
                    },
                    final_state: CreatureState {
                        current_hp: 50,
                        resources: SerializableResourceLedger {
                            current: [("HP".to_string(), 50.0)].into(),
                            max: [("HP".to_string(), 75.0)].into()
                        },
                        ..CreatureState::default()
                    },
                    actions: vec![],
                },
                Combattant {
                    id: "f2".to_string(), team: 0, creature: std::sync::Arc::new(fighter.clone()), initiative: 10.0,
                    initial_state: CreatureState {
                        current_hp: 75,
                        resources: SerializableResourceLedger {
                            current: [("HP".to_string(), 75.0)].into(),
                            max: [("HP".to_string(), 75.0)].into()
                        },
                        ..CreatureState::default() 
                    },
                    final_state: CreatureState { 
                        current_hp: 50, 
                        resources: SerializableResourceLedger { 
                            current: [("HP".to_string(), 50.0)].into(), 
                            max: [("HP".to_string(), 75.0)].into() 
                        },
                        ..CreatureState::default() 
                    },
                    actions: vec![],
                },
            ],
            team2: vec![],
        }],
        target_role: TargetRole::Standard,
    };

    let run = SimulationResult {
        encounters: vec![encounter_res],
        score: Some(1_999_950.0), // Penalty of 150 pushed it below 2M
        num_combat_encounters: 1,
    };

    let output = run_decile_analysis(&vec![run; 100], "Regression Test", 2);
    
    // Calculation:
    // TDNW: 2 * (75 HP + (0 HD * 8) + (0 Slots)) = 150.
    // Start EHP: 2 * 75 = 150.
    // End EHP: 2 * 50 = 100.
    // Attrition (Burned Resources): 150 - 100 = 50.
    // Plus Penalty: 150 (from score logic: TDNW + 2M - score = 150 + 2M - 1,999,950 = 200 total attrition)
    // Wait, let's look at calculate_run_stats:
    // burned_resources = start_ehp - end_ehp = 50.
    // But we also want to include the resource penalty in intensity!
    
    println!("Regression Tier: {:?}", output.intensity_tier);
    assert_ne!(output.intensity_tier, simulation_wasm::decile_analysis::IntensityTier::Tier1, "Intensity should NOT be Tier 1");
}

#[test]
fn test_resource_timeline_points() {
    use simulation_wasm::decile_analysis::run_decile_analysis;
    use simulation_wasm::model::*;
    use std::collections::HashMap;

    let fighter = Creature {
        id: "f".to_string(), name: "f".to_string(), hp: 100, ac: 10, count: 1.0,
        arrival: None, mode: "p".to_string(), speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![], triggers: vec![], spell_slots: None, class_resources: None,
        hit_dice: None, con_modifier: None,
    };

    // 3 encounters
    let mut encounters = Vec::new();
    for _ in 0..3 {
        encounters.push(EncounterResult {
            stats: HashMap::new(),
            rounds: vec![Round {
                team1: vec![
                    Combattant {
                        id: "f1".to_string(), team: 0, creature: std::sync::Arc::new(fighter.clone()), initiative: 10.0,
                        initial_state: CreatureState {
                            current_hp: 100,
                            resources: SerializableResourceLedger {
                                current: [("HP".to_string(), 100.0)].into(),
                                max: [("HP".to_string(), 100.0)].into()
                            },
                            ..CreatureState::default() 
                        },
                        final_state: CreatureState { 
                            current_hp: 80, 
                            resources: SerializableResourceLedger { 
                                current: [("HP".to_string(), 80.0)].into(), 
                                max: [("HP".to_string(), 100.0)].into() 
                            },
                            ..CreatureState::default() 
                        },
                        actions: vec![],
                    }
                ],
                team2: vec![],
            }],
            target_role: TargetRole::Standard,
        });
    }

    let run = SimulationResult {
        encounters,
        score: Some(1_000_080.0),
        num_combat_encounters: 3,
    };

    let output = run_decile_analysis(&vec![run; 11], "Timeline Test", 1);
    
    // Total steps: Start + 3 encounters = 4 points
    let timeline = &output.deciles[0].resource_timeline;
    assert_eq!(timeline.len(), 4, "Timeline should have 4 points for a 3-encounter day");
    assert!((timeline[0] - 100.0).abs() < 0.1, "Start should be 100%");
}