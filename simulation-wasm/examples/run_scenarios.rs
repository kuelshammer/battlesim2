use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::run_event_driven_simulation_rust;
use std::fs;
use std::path::Path;

fn main() {
    let scenarios_dir = Path::new("tests/scenarios");
    if !scenarios_dir.exists() {
        eprintln!("Scenarios directory not found: {:?}", scenarios_dir);
        return;
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    println!("Running regression scenarios from {:?}...", scenarios_dir);
    println!("---------------------------------------------------");

    for entry in fs::read_dir(scenarios_dir).expect("Failed to read scenarios directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path.file_name().unwrap().to_string_lossy();
            println!("Testing scenario: {}", file_name);

            let content = fs::read_to_string(&path).expect("Failed to read file");
            let data: serde_json::Value =
                serde_json::from_str(&content).expect("Failed to parse JSON");

            let players: Vec<Creature> =
                serde_json::from_value(data["players"].clone()).expect("Failed to parse players");
            let encounters: Vec<Encounter> = serde_json::from_value(data["encounters"].clone())
                .expect("Failed to parse encounters");

            // Run simulation (1005 iterations for quintile statistical significance)
            let iterations = 1005;
            let (results, _) = run_event_driven_simulation_rust(
                players.clone(),
                encounters.clone(),
                iterations,
                false,
            );

            // Results are already sorted by score (ascending) by run_event_driven_simulation_rust

            // Extract Middle Quintile (indices 402 to 602 inclusive, size ~201)
            let start_idx = iterations * 2 / 5; // 402
            let end_idx = iterations * 3 / 5; // 603

            let middle_quintile = &results[start_idx..end_idx];
            let quintile_size = middle_quintile.len();

            // Calculate Win Rate for Team A in Middle Quintile
            let mut wins = 0;
            let mut total_rounds = 0;

            for result in middle_quintile {
                // result is Vec<EncounterResult>
                if let Some(encounter) = result.last() {
                    if let Some(last_round) = encounter.rounds.last() {
                        let team1_alive = last_round
                            .team1
                            .iter()
                            .any(|c| c.final_state.current_hp > 0.0);
                        let team2_alive = last_round
                            .team2
                            .iter()
                            .any(|c| c.final_state.current_hp > 0.0);

                        // Check if Team A won
                        if team1_alive && !team2_alive {
                            wins += 1;
                        }
                        total_rounds += encounter.rounds.len();
                    }
                }
            }

            let win_rate = (wins as f64 / quintile_size as f64) * 100.0;
            let avg_rounds = total_rounds as f64 / quintile_size as f64;

            println!(
                "  Middle Quintile Results ({} runs): {:.1}% Win Rate, {:.1} Avg Rounds",
                quintile_size, win_rate, avg_rounds
            );

            // User requirement: In the middle 20% quintile, player A is winning more often than Monster B (> 50%)
            if win_rate > 50.0 {
                println!("  ✅ PASS: Team A win rate > 50% in middle quintile");
                success_count += 1;
            } else {
                println!("  ❌ FAIL: Team A win rate <= 50% in middle quintile");
                fail_count += 1;
            }
            println!("---------------------------------------------------");
        }
    }

    println!("Summary: {} Passed, {} Failed", success_count, fail_count);
    if fail_count > 0 {
        std::process::exit(1);
    }
}
