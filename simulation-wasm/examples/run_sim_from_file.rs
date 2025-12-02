use std::fs;
use std::env;
use serde::{Deserialize, Serialize};
use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::simulation::run_simulation_rust;

#[derive(Debug, Serialize, Deserialize)]
struct SimulationInput {
    players: Vec<Creature>,
    encounters: Vec<Encounter>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_json_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let input_content = fs::read_to_string(input_path).expect("Failed to read input file");
    
    let input: SimulationInput = serde_json::from_str(&input_content).expect("Failed to parse JSON");

    println!("Running simulation with {} players and {} encounters...", input.players.len(), input.encounters.len());

    // Run a single simulation with logging enabled
    let (results, logs) = run_simulation_rust(input.players, input.encounters, 1, true);

    println!("\n=== SIMULATION LOGS ===\n");
    for log in logs {
        println!("{}", log);
    }

    println!("\n=== RESULTS ===\n");
    if let Some(first_run) = results.first() {
        for (i, encounter_res) in first_run.iter().enumerate() {
            println!("Encounter {}: {} rounds", i + 1, encounter_res.rounds.len());
        }
    }
}
