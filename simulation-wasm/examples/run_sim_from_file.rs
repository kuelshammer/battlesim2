use std::fs;
use std::env;
use serde::{Deserialize, Serialize};
use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::simulation::run_simulation_rust;
use simulation_wasm::run_event_driven_simulation_rust;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct SimulationInput {
    players: Vec<Creature>,
    encounters: Vec<Encounter>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_json_file> [--event-driven]", args[0]);
        eprintln!("  --event-driven: Use event-driven simulation (default: legacy)");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let use_event_driven = args.get(2).map(|s| s == "--event-driven").unwrap_or(false);
    
    let input_content = fs::read_to_string(input_path).expect("Failed to read input file");
    
    let input: SimulationInput = serde_json::from_str(&input_content).expect("Failed to parse JSON");

    println!("Running {} simulation with {} players and {} encounters...", 
        if use_event_driven { "event-driven" } else { "legacy" },
        input.players.len(), 
        input.encounters.len()
    );

    // Run a single simulation with logging enabled
    let (results, logs): (_, Vec<String>) = if use_event_driven {
        let (res, events) = run_event_driven_simulation_rust(input.players, input.encounters, 1, true);
        let string_logs = events.into_iter()
                                .map(|event| serde_json::to_string(&event).unwrap_or_default())
                                .collect();
        (res, string_logs)
    } else {
        run_simulation_rust(input.players, input.encounters, 1, true)
    };

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
