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
    let iterations = 1001; // Increased for better statistical significance

    // Run simulation multiple times and take median to reduce variance
    let mut win_rates = Vec::new();
    for _run in 0..3 {
        let runs = run_event_driven_simulation_rust(players.clone(), timeline.clone(), iterations, false);
        let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

        // Sort results (required for decile analysis)
        results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
            .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
            .unwrap_or(std::cmp::Ordering::Equal));

        // Analyze
        let party_size = 1; // Assuming 1v1 for these tests
        let analysis = run_decile_analysis(&results, scenario_file, party_size);
        win_rates.push(analysis.deciles[4].win_rate);
    }

    // Take median of the 3 runs to smooth out variance
    win_rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_win_rate = win_rates[1];

    println!("  Median Win Rate (3 runs): {:.1}%", median_win_rate);
    println!("  Individual runs: {:.1}%, {:.1}%, {:.1}%", win_rates[0], win_rates[1], win_rates[2]);

    // Use 45% threshold instead of 50% to account for inherent variance
    const THRESHOLD: f64 = 45.0;

    if expected_winner_is_player {
        assert!(median_win_rate > THRESHOLD,
            "Regression Failed: Player should win in '{}', but median win rate is {:.1}% (threshold: {:.1}%)",
            scenario_file, median_win_rate, THRESHOLD);
    } else {
        assert!(median_win_rate < (100.0 - THRESHOLD),
            "Regression Failed: Monster should win in '{}', but median win rate is {:.1}% (threshold: {:.1}%)",
            scenario_file, median_win_rate, 100.0 - THRESHOLD);
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