use simulation_wasm::aggregation::calculate_score;
use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::run_event_driven_simulation_rust;
use std::fs;

fn main() {
    // Load test data (Andrew vs Bernd)
    let content = fs::read_to_string("tests/scenarios/crit_test_PlayerA_wins.json")
        .expect("Failed to read test file");
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
    let encounters: Vec<Encounter> =
        serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");

    // Run 1005 iterations
    let iterations = 1005;
    println!("Running {} iterations...", iterations);
    let (results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);

    assert_eq!(
        results.len(),
        iterations,
        "Should have {} results",
        iterations
    );

    // Verify sorting
    println!("Verifying result sorting...");
    let mut prev_score = f64::NEG_INFINITY;
    let mut sorted = true;

    for (i, result) in results.iter().enumerate() {
        let score = calculate_score(result);
        if score < prev_score {
            println!(
                "❌ Result {} is out of order: score {} < prev {}",
                i, score, prev_score
            );
            sorted = false;
            break;
        }
        prev_score = score;
    }

    if sorted {
        println!("✅ Results are correctly sorted by score.");
    } else {
        println!("❌ Results are NOT sorted.");
        std::process::exit(1);
    }

    // Verify Score Formula (10 * HP - Monster HP)
    // Pick median result
    let median_result = &results[iterations / 2];
    let score = calculate_score(median_result);

    if let Some(encounter) = median_result.last() {
        if let Some(round) = encounter.rounds.last() {
            let player_hp: f64 = round.team1.iter().map(|c| c.final_state.current_hp).sum();
            let monster_hp: f64 = round.team2.iter().map(|c| c.final_state.current_hp).sum();
            let expected_score = 10.0 * player_hp - monster_hp;

            println!("Median Result Check:");
            println!("  Player HP: {}", player_hp);
            println!("  Monster HP: {}", monster_hp);
            println!("  Calculated Score: {}", score);
            println!("  Expected Score: {}", expected_score);

            if (score - expected_score).abs() < 0.001 {
                println!("✅ Score formula matches (10 * PlayerHP - MonsterHP)");
            } else {
                println!("❌ Score formula mismatch!");
                std::process::exit(1);
            }
        }
    }
}
