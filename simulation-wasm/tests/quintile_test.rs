use simulation_wasm::aggregation::calculate_score;
use simulation_wasm::model::{Creature, SimulationResult, DiceFormula};
use simulation_wasm::quintile_analysis::run_quintile_analysis;

// Helper to create a dummy SimulationResult for testing
fn create_dummy_simulation_result(_score: f64, player_hp: f64, monster_hp: f64, rounds: usize) -> SimulationResult {
    // Simplified SimulationResult for testing purposes
    // In a real scenario, this would come from a full simulation run
    let mut result = Vec::new();
    let mut encounter_result = simulation_wasm::model::EncounterResult {
        stats: std::collections::HashMap::new(),
        rounds: Vec::new(),
    };

    // Create a dummy final round to extract visualization from
    let mut team1 = Vec::new();
    let mut team2 = Vec::new();

    team1.push(simulation_wasm::model::Combattant {
        id: "PlayerA".to_string(),
        creature: Creature {
            id: "PlayerA".to_string(),
            arrival: None,
            name: "PlayerA".to_string(),
            hp: 100.0, // Max HP
            ac: 15.0,
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
            initiative_bonus: DiceFormula::Value(2.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            mode: "player".to_string(),
            count: 1.0,
        },
        initiative: 0.0,
        initial_state: simulation_wasm::model::CreatureState {
            current_hp: 100.0,
            temp_hp: None,
            buffs: std::collections::HashMap::new(),
            resources: simulation_wasm::model::SerializableResourceLedger { current: std::collections::HashMap::new(), max: std::collections::HashMap::new() },
            upcoming_buffs: std::collections::HashMap::new(),
            used_actions: std::collections::HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: std::collections::HashSet::new(),
            bonus_action_used: false,
            known_ac: std::collections::HashMap::new(),
            arcane_ward_hp: None,
        },
        final_state: simulation_wasm::model::CreatureState {
            current_hp: player_hp,
            temp_hp: None,
            buffs: std::collections::HashMap::new(),
            resources: simulation_wasm::model::SerializableResourceLedger { current: std::collections::HashMap::new(), max: std::collections::HashMap::new() },
            upcoming_buffs: std::collections::HashMap::new(),
            used_actions: std::collections::HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: std::collections::HashSet::new(),
            bonus_action_used: false,
            known_ac: std::collections::HashMap::new(),
            arcane_ward_hp: None,
        },
        actions: Vec::new(),
    });

    team2.push(simulation_wasm::model::Combattant {
        id: "MonsterB".to_string(),
        creature: Creature {
            id: "MonsterB".to_string(),
            arrival: None,
            name: "MonsterB".to_string(),
            hp: 100.0, // Max HP
            ac: 13.0,
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
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: Vec::new(),
            triggers: Vec::new(),
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            mode: "monster".to_string(),
            count: 1.0,
        },
        initiative: 0.0,
        initial_state: simulation_wasm::model::CreatureState {
            current_hp: 100.0,
            temp_hp: None,
            buffs: std::collections::HashMap::new(),
            resources: simulation_wasm::model::SerializableResourceLedger { current: std::collections::HashMap::new(), max: std::collections::HashMap::new() },
            upcoming_buffs: std::collections::HashMap::new(),
            used_actions: std::collections::HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: std::collections::HashSet::new(),
            bonus_action_used: false,
            known_ac: std::collections::HashMap::new(),
            arcane_ward_hp: None,
        },
        final_state: simulation_wasm::model::CreatureState {
            current_hp: monster_hp,
            temp_hp: None,
            buffs: std::collections::HashMap::new(),
            resources: simulation_wasm::model::SerializableResourceLedger { current: std::collections::HashMap::new(), max: std::collections::HashMap::new() },
            upcoming_buffs: std::collections::HashMap::new(),
            used_actions: std::collections::HashSet::new(),
            concentrating_on: None,
            actions_used_this_encounter: std::collections::HashSet::new(),
            bonus_action_used: false,
            known_ac: std::collections::HashMap::new(),
            arcane_ward_hp: None,
        },
        actions: Vec::new(),
    });

    for _ in 0..rounds {
        encounter_result.rounds.push(simulation_wasm::model::Round {
            team1: team1.clone(),
            team2: team2.clone(),
        });
    }


    result.push(encounter_result);
    result
}

#[test]
fn test_quintile_analysis_median_run_visualization() {
    let scenario_name = "TestScenario";
    let party_size = 1; // PlayerA

    // Create 1005 dummy simulation results with varying scores
    // Sort order: worst to best.
    // Score calculation: (survivors * 10000) + remaining_party_hp - (remaining_monster_hp)
    let mut results: Vec<SimulationResult> = (0..1005)
        .map(|i| {
            let score_base = i as f64; // Simple score for sorting
            let player_hp;
            let monster_hp;
            let rounds;

            if i < 201 { // Worst 20% (Losses)
                player_hp = 0.0;
                monster_hp = 100.0 - (score_base % 10.0); // Monster has some HP left
                rounds = 5 + (i % 5);
            } else if i < 402 { // Below Average (Some losses, some pyrrhic wins)
                player_hp = if i % 2 == 0 { 0.0 } else { 10.0 + (score_base % 5.0) };
                monster_hp = if i % 2 == 0 { 100.0 - (score_base % 10.0) } else { 0.0 };
                rounds = 10 + (i % 5);
            } else if i < 603 { // Median (Wins, some damage)
                player_hp = 20.0 + (score_base % 10.0);
                monster_hp = 0.0;
                rounds = 15 + (i % 5);
            } else if i < 804 { // Above Average (Good wins)
                player_hp = 50.0 + (score_base % 10.0);
                monster_hp = 0.0;
                rounds = 20 + (i % 5);
            } else { // Best 20% (Excellent wins)
                player_hp = 80.0 + (score_base % 10.0);
                monster_hp = 0.0;
                rounds = 25 + (i % 5);
            }
            create_dummy_simulation_result(score_base, player_hp, monster_hp, rounds)
        })
        .collect();

    // The run_quintile_analysis function expects sorted results.
    // The create_dummy_simulation_result uses 'score_base' for a simple sorting key.
    // The actual calculate_score is more complex, but for this test, we want a controlled sort.
    results.sort_by(|a, b| calculate_score(a).partial_cmp(&calculate_score(b)).unwrap_or(std::cmp::Ordering::Equal));

    let output = run_quintile_analysis(&results, scenario_name, party_size);

    assert_eq!(output.quintiles.len(), 5, "Should have 5 quintiles");

    // Test Quintile 1 (Worst 20%) - Median is 101st run (index 100)
    let q1 = &output.quintiles[0];
    assert_eq!(q1.quintile, 1);
    assert_eq!(q1.label, "Worst 20%");
    assert!(!q1.median_run_visualization.is_empty(), "Q1 median run visualization should not be empty");
    
    // Check PlayerA in Q1 median run
    let player_a_q1 = q1.median_run_visualization.iter().find(|c| c.name == "PlayerA").expect("PlayerA not found in Q1 visualization");
    assert_eq!(player_a_q1.name, "PlayerA");
    assert_eq!(player_a_q1.max_hp, 100.0);
    assert!(player_a_q1.is_dead, "PlayerA should be dead in Q1 median run");
    
    // Check MonsterB in Q1 median run
    let monster_b_q1 = q1.median_run_visualization.iter().find(|c| c.name == "MonsterB").expect("MonsterB not found in Q1 visualization");
    assert_eq!(monster_b_q1.name, "MonsterB");
    assert_eq!(monster_b_q1.max_hp, 100.0);
    assert!(!monster_b_q1.is_dead, "MonsterB should be alive in Q1 median run");


    // Test Quintile 3 (Median) - Median is 101st run (index 502 overall, 100 within its slice)
    let q3 = &output.quintiles[2];
    assert_eq!(q3.quintile, 3);
    assert_eq!(q3.label, "Median");
    assert!(!q3.median_run_visualization.is_empty(), "Q3 median run visualization should not be empty");

    // Check PlayerA in Q3 median run
    let player_a_q3 = q3.median_run_visualization.iter().find(|c| c.name == "PlayerA").expect("PlayerA not found in Q3 visualization");
    assert_eq!(player_a_q3.name, "PlayerA");
    assert_eq!(player_a_q3.max_hp, 100.0);
    assert!(!player_a_q3.is_dead, "PlayerA should be alive in Q3 median run");
    
    // Check MonsterB in Q3 median run
    let monster_b_q3 = q3.median_run_visualization.iter().find(|c| c.name == "MonsterB").expect("MonsterB not found in Q3 visualization");
    assert_eq!(monster_b_q3.name, "MonsterB");
    assert_eq!(monster_b_q3.max_hp, 100.0);
    assert!(monster_b_q3.is_dead, "MonsterB should be dead in Q3 median run");


    // Ensure all quintiles have visualization data
    for (i, q_stats) in output.quintiles.iter().enumerate() {
        assert!(!q_stats.median_run_visualization.is_empty(), "Quintile {} visualization should not be empty", i + 1);
        assert!(q_stats.battle_duration_rounds > 0, "Quintile {} battle duration should be greater than 0", i + 1);
        
        // Verify max_hp for all combatants in all quintiles
        for combatant_viz in &q_stats.median_run_visualization {
            assert_eq!(combatant_viz.max_hp, 100.0, "Combatant {} in Quintile {} has incorrect max_hp", combatant_viz.name, i + 1);
        }
    }
}
