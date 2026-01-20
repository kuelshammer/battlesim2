use crate::common::load_scenario;
use simulation_wasm::decile_analysis::run_decile_analysis;
use simulation_wasm::run_event_driven_simulation_rust;

fn run_regression_test(scenario_file: &str, expected_winner_is_player: bool) {
    println!("Running regression test for: {}", scenario_file);
    let (players, timeline) = load_scenario(scenario_file);
    let sr_count = timeline
        .iter()
        .filter(|s| matches!(s, simulation_wasm::model::TimelineStep::ShortRest(_)))
        .count();

    // With deterministic RNG, we can use fewer iterations
    // Each run produces the same result when seeded
    let iterations = 251; // Increased from 101 for better stability

    // Run simulation once with fixed seed for deterministic results
    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    // Sort results (required for decile analysis)
    results.sort_by(|a, b| {
        simulation_wasm::aggregation::calculate_score(a)
            .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Analyze
    let party_size = 1; // Assuming 1v1 for these tests
    let analysis = run_decile_analysis(&results, scenario_file, party_size, sr_count);
    let median_decile = &analysis.deciles[4]; // Index 4 is Median (50th percentile)

    println!("  Win Rate: {:.1}%", median_decile.win_rate);
    println!(
        "  Survivors: {}/{}",
        median_decile.median_survivors, median_decile.party_size
    );

    // With deterministic RNG, we can use the original 50% threshold
    const THRESHOLD: f64 = 50.0;

    if expected_winner_is_player {
        assert!(
            median_decile.win_rate > THRESHOLD,
            "Regression Failed: Player should win in '{}', but win rate is {:.1}%",
            scenario_file,
            median_decile.win_rate
        );
    } else {
        assert!(
            median_decile.win_rate < THRESHOLD,
            "Regression Failed: Monster should win in '{}', but win rate is {:.1}%",
            scenario_file,
            median_decile.win_rate
        );
    }
}

#[test]
fn test_mechanics_initiative_dominance() {
    // Fast Initiative should guarantee a win in a balanced mirror match
    run_regression_test("fast_init_PlayerA_wins.json", true);
}

#[test]
fn test_mechanics_damage_vs_precision() {
    // MonsterB has slightly higher hit chance which wins over slightly higher damage in this seed
    run_regression_test("damage_vs_precision_MonsterB_wins.json", false);
}

#[test]
fn test_mechanics_heavy_vs_consistent() {
    // Current balance: Burst Damage (Player) beats Low Dmg/High Hit (Monster)
    run_regression_test("heavy_vs_consistent_PlayerA_wins.json", true);
}
