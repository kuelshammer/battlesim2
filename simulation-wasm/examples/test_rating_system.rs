use simulation_wasm::quintile_analysis::run_quintile_analysis;
use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::run_event_driven_simulation_rust;
use std::fs;

fn main() {
    // Test scenarios to validate rating system
    let test_scenarios = vec![
        ("crit_test_PlayerA_wins.json", "Glass Cannon vs Tank"),
        ("damage_vs_precision_PlayerA_wins.json", "Balanced Duel"),
        ("heavy_vs_consistent_MonsterB_wins.json", "Power vs Reliability"),
    ];

    for (scenario_file, description) in test_scenarios {
        println!("\n=== Testing: {} ===", description);
        println!("Scenario file: {}", scenario_file);
        
        // Load test data
        let content = fs::read_to_string(&format!("tests/scenarios/{}", scenario_file))
            .expect("Failed to read test file");
        let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

        let players: Vec<Creature> =
            serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
        let encounters: Vec<Encounter> =
            serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");

        // Run 1005 iterations for quintile analysis
        let iterations = 1005;
        println!("Running {} iterations...", iterations);
        let (results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);

        // Calculate party size from first result
        let party_size = if let Some(first_result) = results.first() {
            if let Some(first_encounter) = first_result.first() {
                if let Some(first_round) = first_encounter.rounds.first() {
                    first_round.team1.len()
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        // Run quintile analysis with rating system
        let analysis = run_quintile_analysis(&results, scenario_file, party_size);
        
        // Display results
        println!("🏷️  Encounter Label: {:?}", analysis.encounter_label);
        println!("⚠️  Risk Factor: {:?}", analysis.risk_factor);
        println!("📊 Difficulty: {:?}", analysis.difficulty);
        println!("📝 Analysis: {}", analysis.analysis_summary);
        
        if !analysis.tuning_suggestions.is_empty() {
            println!("💡 Suggestions:");
            for suggestion in &analysis.tuning_suggestions {
                println!("  • {}", suggestion);
            }
        }
        
        // Show quintile breakdown
        println!("\nQuintile Breakdown:");
        for quintile in &analysis.quintiles {
            println!("  {}: {} survivors, {:.1}% win rate, {:.1}% HP lost", 
                quintile.label, 
                quintile.median_survivors,
                quintile.win_rate, 
                quintile.hp_lost_percent);
        }
    }
}