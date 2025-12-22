use simulation_wasm::model::{Creature, Encounter, TimelineStep};
use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::decile_analysis::run_decile_analysis;
use std::fs;
use std::path::PathBuf;

fn load_scenario(filename: &str) -> (Vec<Creature>, Vec<TimelineStep>, String) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios");
    path.push(filename);

    let content = fs::read_to_string(&path).expect(&format!("Failed to read scenario file: {:?}", path));
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
    
    let timeline: Vec<TimelineStep> = if let Some(t) = data.get("timeline") {
        serde_json::from_value(t.clone()).expect("Failed to parse timeline")
    } else {
        let encounters: Vec<Encounter> = serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");
        encounters.into_iter().map(TimelineStep::Combat).collect()
    };
    
    (players, timeline, filename.to_string())
}

fn run_regression_test(scenario_file: &str, expected_winner_is_player: bool) {
    println!("Running regression test for: {}", scenario_file);
    let (players, timeline, _) = load_scenario(scenario_file);
    let iterations = 505; // Sufficient for decile analysis
    
    // Run simulation
    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false);
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();
    
    // Sort results (required for decile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1; // Assuming 1v1 for these tests
    let analysis = run_decile_analysis(&results, scenario_file, party_size);
    let median_decile = &analysis.deciles[4]; // Index 4 is Median (50th percentile)

    println!("  Median Win Rate: {:.1}%", median_decile.win_rate);
    println!("  Median Survivors: {}/{}", median_decile.median_survivors, median_decile.party_size);

    if expected_winner_is_player {
        assert!(median_decile.win_rate > 50.0, 
            "Regression Failed: Player should win in '{}', but median win rate is {:.1}%", 
            scenario_file, median_decile.win_rate);
    } else {
        assert!(median_decile.win_rate < 50.0, 
            "Regression Failed: Monster should win in '{}', but median win rate is {:.1}%", 
            scenario_file, median_decile.win_rate);
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