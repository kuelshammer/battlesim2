use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::quintile_analysis::run_quintile_analysis;
use std::fs;
use std::path::PathBuf;

fn load_scenario(filename: &str) -> (Vec<Creature>, Vec<Encounter>, String) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios");
    path.push(filename);

    let content = fs::read_to_string(&path).expect(&format!("Failed to read scenario file: {:?}", path));
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
    let encounters: Vec<Encounter> =
        serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");
    
    (players, encounters, filename.to_string())
}

fn run_regression_test(scenario_file: &str, expected_winner_is_player: bool) {
    println!("Running regression test for: {}", scenario_file);
    let (players, encounters, _) = load_scenario(scenario_file);
    let iterations = 505; // Sufficient for quintile analysis
    
    // Run simulation
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);
    
    // Sort results (required for quintile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1; // Assuming 1v1 for these tests
    let analysis = run_quintile_analysis(&results, scenario_file, party_size);
    let median_quintile = &analysis.quintiles[2]; // Index 2 is Median

    println!("  Median Win Rate: {:.1}%", median_quintile.win_rate);
    println!("  Median Survivors: {}/{}", median_quintile.median_survivors, median_quintile.party_size);

    if expected_winner_is_player {
        assert!(median_quintile.win_rate > 50.0, 
            "Regression Failed: Player should win in '{}', but median win rate is {:.1}%", 
            scenario_file, median_quintile.win_rate);
    } else {
        assert!(median_quintile.win_rate < 50.0, 
            "Regression Failed: Monster should win in '{}', but median win rate is {:.1}%", 
            scenario_file, median_quintile.win_rate);
    }
}

#[test]
fn test_mechanics_initiative_dominance() {
    // Fast Initiative should guarantee a win in a balanced mirror match
    run_regression_test("fast_init_PlayerA_wins.json", true);
}

#[test]
fn test_mechanics_damage_vs_precision() {
    // Current balance: Precision (Monster) beats Damage (Player) in this specific setup
    run_regression_test("damage_vs_precision_MonsterB_wins.json", false);
}

#[test]
fn test_mechanics_heavy_vs_consistent() {
    // Current balance: Burst Damage (Player) beats Low Dmg/High Hit (Monster)
    run_regression_test("heavy_vs_consistent_PlayerA_wins.json", true);
}
