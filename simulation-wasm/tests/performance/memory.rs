// WASM OOM Reproduction Test
//
// This test reproduces the out-of-memory error that occurs when running
// simulations with high iteration counts (2511) in WASM.
//
// Error: RuntimeError: unreachable at __rg_oom
// Root cause: Storing all 2511 results in Vec<SimulationResult> causes O(N) memory usage

use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::model::{Creature, TimelineStep, TargetRole, DiceFormula, Encounter};
use std::mem::size_of;

// Helper to calculate deep size of heap-allocated data
fn calculate_deep_size<T: ?Sized>(_: &T) -> usize {
    // Placeholder - in reality, we'd need to recursively traverse all allocations
    // For now, we'll use theoretical calculations based on structure sizes
    0
}

// Helper to create a simple test scenario
fn create_test_scenario() -> (Vec<Creature>, Vec<TimelineStep>) {
    let players = vec![Creature {
        id: "fighter".to_string(),
        name: "Fighter".to_string(),
        hp: 20,
        ac: 10,
        count: 1.0,
        actions: vec![],
        triggers: vec![],
        arrival: None,
        mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
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
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    }];

    let encounter = Encounter {
        monsters: vec![],
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: TargetRole::Standard,
    };

    let timeline = vec![TimelineStep::Combat(encounter)];

    (players, timeline)
}

#[test]
fn measure_structure_sizes() {
    use simulation_wasm::model::{SimulationResult, Combattant, SimulationRun, EncounterResult};
    use simulation_wasm::context::CombattantState;

    println!("=== Memory Footprint Analysis ===");
    println!("Size of Creature: {} bytes", size_of::<Creature>());
    println!("Size of Vec<Creature>: {} bytes", size_of::<Vec<Creature>>());
    println!("Size of TimelineStep: {} bytes", size_of::<TimelineStep>());
    println!("Size of Vec<TimelineStep>: {} bytes", size_of::<Vec<TimelineStep>>());
    println!("Size of Combattant: {} bytes", size_of::<Combattant>());
    println!("Size of CombattantState: {} bytes", size_of::<CombattantState>());
    println!("Size of EncounterResult: {} bytes", size_of::<EncounterResult>());
    println!("Size of SimulationResult: {} bytes", size_of::<SimulationResult>());
    println!("Size of SimulationRun: {} bytes", size_of::<SimulationRun>());
    println!("Size of Vec<SimulationRun>: {} bytes (empty)", size_of::<Vec<SimulationRun>>());
    println!("====================================");

    // Rough memory usage per iteration calculation:
    // - SimulationRun contains SimulationResult + Vec<Event>
    // - SimulationResult contains Vec<EncounterResult> containing Vec<Combattant>
    // - Each Combattant contains a full Creature clone
    // - Empty encounter with 1 player: ~1-2 KB per iteration minimum
    // With 5 combatants in a real encounter: ~5-10 KB per iteration
}

#[test]
fn estimate_memory_for_iterations() {
    let (players, timeline) = create_test_scenario();

    // Run 100 iterations to get a baseline (minimum enforced)
    let iterations = 100;
    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, Some(42));

    // Calculate theoretical memory based on structure sizes and content
    use simulation_wasm::model::{SimulationRun, SimulationResult, EncounterResult};
    use simulation_wasm::context::CombattantState;

    let mut total_estimated_bytes = 0;
    for run in &runs {
        // Vec structure overhead for the Vec itself
        total_estimated_bytes += size_of::<SimulationRun>();

        // SimulationResult contains: Vec<EncounterResult>
        let result_bytes = size_of::<SimulationResult>();

        // EncounterResult contains: HashMap<String, EncounterStats> + Vec<Round>
        // Each EncounterResult has stats + rounds with combattants
        for encounter_res in &run.result.encounters {
            total_estimated_bytes += size_of::<EncounterResult>();
            // HashMap overhead (bucket array + entries)
            total_estimated_bytes += encounter_res.stats.len() * (size_of::<String>() + 64); // rough estimate
            // Rounds with combattants
            for round in &encounter_res.rounds {
                total_estimated_bytes += size_of::<Vec<simulation_wasm::model::Combattant>>() * 2; // two teams
                total_estimated_bytes += round.team1.len() * size_of::<simulation_wasm::model::Combattant>();
                total_estimated_bytes += round.team2.len() * size_of::<simulation_wasm::model::Combattant>();
            }
        }

        // Events Vec
        total_estimated_bytes += run.events.len() * 64; // rough estimate per event
    }

    let per_iteration = total_estimated_bytes / iterations;

    println!("=== Deep Memory Estimation ===");
    println!("Total estimated for {} iterations: {} bytes", iterations, total_estimated_bytes);
    println!("Estimated per iteration: {} bytes", per_iteration);
    println!("==============================");

    // Estimate for different iteration counts
    for est_iterations in [31, 100, 500, 1000, 2511] {
        let estimated_mb = (per_iteration * est_iterations) as f64 / (1024.0 * 1024.0);
        println!("{} iterations: ~{:.2} MB", est_iterations, estimated_mb);
    }

    // Typical WASM heap limit is 16MB to 128MB depending on browser
    // If we exceed ~80% of the heap, allocation failures may occur
}

#[test]
fn test_small_iteration_count() {
    let (players, timeline) = create_test_scenario();

    // Requested 31, but should get 100 (enforced minimum)
    let requested = 31;
    let expected = 100;
    let runs = run_event_driven_simulation_rust(players, timeline, requested, false, Some(42));

    assert_eq!(runs.len(), expected);
    println!("✓ Requested {} iterations, received {} (minimum enforced)", requested, runs.len());
}

#[test]
fn test_medium_iteration_count() {
    let (players, timeline) = create_test_scenario();

    // Medium count (500) - should also work
    let iterations = 500;
    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, Some(42));

    assert_eq!(runs.len(), iterations);
    println!("✓ {} iterations completed successfully", iterations);
}

#[test]
#[ignore] // Ignored by default - run with: cargo test test_high_iteration_count_oom_trigger -- --ignored
fn test_high_iteration_count_oom_trigger() {
    let (players, timeline) = create_test_scenario();

    // Precise iterations (2511) - this may trigger OOM in WASM
    let iterations = 2511;

    println!("Attempting {} iterations - this may OOM in WASM environment...", iterations);

    let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, Some(42));

    assert_eq!(runs.len(), iterations);
    println!("✓ {} iterations completed successfully (OOM may have been fixed!)", iterations);
}

#[test]
fn demonstrate_memory_growth() {
    let (players, timeline) = create_test_scenario();

    // Measure at different scales to show linear O(N) growth
    println!("=== Memory Growth Demonstration ===");

    for iterations in [10, 50, 100, 500] {
        let actual_expected = iterations.max(100);
        let start = std::time::Instant::now();
        let runs = run_event_driven_simulation_rust(
            players.clone(),
            timeline.clone(),
            iterations,
            false,
            Some(42),
        );
        let elapsed = start.elapsed();

        // Theoretical size calculation
        let mut total_size = 0;
        for run in &runs {
            total_size += size_of_val(&run);
            // Rough estimate of heap allocations
            for enc in &run.result.encounters {
                total_size += enc.stats.len() * 128; // HashMap entry estimate
                for round in &enc.rounds {
                    total_size += (round.team1.len() + round.team2.len()) * size_of::<simulation_wasm::model::Combattant>();
                }
            }
            total_size += run.events.len() * 64; // Event estimate
        }

        println!(
            "{} iterations: {} bytes estimated, {} ms (~{} bytes/iter, ~{:.2} MB)",
            iterations,
            total_size,
            elapsed.as_millis(),
            total_size / actual_expected,
            total_size as f64 / (1024.0 * 1024.0)
        );
    }

    println!("================================");
    println!("Note: Actual heap usage is higher due to:");
    println!("  - String allocations (id, name, etc.)");
    println!("  - Vec overhead and reallocations");
    println!("  - HashMap bucket arrays");
    println!("  - Creature clones in each Combattant");
}
