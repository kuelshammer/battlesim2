use simulation_wasm::storage_manager::StorageManager;
use simulation_wasm::model::{Creature, Encounter};
use simulation_wasm::storage::StorageConfig;

fn create_simple_creature(name: &str, hp: f64, ac: f64) -> Creature {
    Creature {
        id: name.to_string(),
        arrival: None,
        mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None,
        name: name.to_string(),
        count: 1.0,
        hp,
        ac,
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
        initiative_bonus: simulation_wasm::model::DiceFormula::Value(0.0),
        initiative_advantage: false,
        actions: vec![], // Keep empty for simplicity
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    }
}

fn create_simple_encounter() -> Encounter {
    Encounter {
        monsters: vec![],
        monsters_precast: None,
        monsters_surprised: None,
        players_precast: None,
        players_surprised: None,
        short_rest: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Dual-Slot Storage System Demo ===");
    
    // Create storage manager with custom config
    let temp_dir = std::env::temp_dir().join("storage_demo");
    let config = StorageConfig {
        storage_directory: temp_dir.to_string_lossy().to_string(),
        enable_compression: true,
        max_age_seconds: 3600,
        max_storage_bytes: 10 * 1024 * 1024, // 10MB
    };
    
    let mut storage_manager = StorageManager::new(config)?;
    
    println!("Storage manager initialized successfully!");
    
    // Create test scenario
    let players = vec![
        create_simple_creature("Fighter", 45.0, 18.0),
        create_simple_creature("Rogue", 35.0, 16.0),
    ];
    
    let encounters = vec![create_simple_encounter()];
    let iterations = 10;
    
    println!("\nRunning simulation with {} iterations...", iterations);
    
    // First run - should execute simulation
    let start = std::time::Instant::now();
    let results1 = storage_manager.run_simulation_with_cache(
        &players,
        &encounters,
        iterations,
        |players, encounters, iterations| {
            // Use existing simulation engine
            Ok(simulation_wasm::run_event_driven_simulation_rust(
                players.to_vec(),
                encounters.to_vec(),
                iterations,
                false,
            ).0)
        },
    )?;
    let first_run_time = start.elapsed();
    
    println!("First run completed in {:?}", first_run_time);
    println!("Generated {} simulation results", results1.len());
    
    // Second run - should use cache
    let start = std::time::Instant::now();
    let results2 = storage_manager.run_simulation_with_cache(
        &players,
        &encounters,
        iterations,
        |_, _, _| {
            panic!("This should not be called - cache should be used!");
        },
    )?;
    let second_run_time = start.elapsed();
    
    println!("Second run completed in {:?}", second_run_time);
    println!("Cache hit! Results length: {}", results2.len());
    
    // Show performance improvement
    let speedup = if second_run_time.as_millis() > 0 {
        first_run_time.as_millis() as f64 / second_run_time.as_millis() as f64
    } else {
        f64::INFINITY
    };
    
    if speedup.is_finite() {
        println!("Performance improvement: {:.2}x faster", speedup);
    } else {
        println!("Cache provided instant results!");
    }
    
    // Display storage statistics
    let stats = storage_manager.get_storage_stats();
    println!("\n=== Storage Statistics ===");
    println!("Primary slot occupied: {}", stats.primary_slot_occupied);
    println!("Secondary slot occupied: {}", stats.secondary_slot_occupied);
    println!("Primary age: {} seconds", stats.primary_age_seconds);
    println!("Secondary age: {} seconds", stats.secondary_age_seconds);
    println!("Total cached simulations: {}", stats.total_cached_simulations);
    
    // Test with different parameters - should run new simulation
    println!("\nTesting with different parameters...");
    let different_players = vec![create_simple_creature("Barbarian", 60.0, 16.0)];
    
    let start = std::time::Instant::now();
    let results3 = storage_manager.run_simulation_with_cache(
        &different_players,
        &encounters,
        iterations,
        |players, encounters, iterations| {
            Ok(simulation_wasm::run_event_driven_simulation_rust(
                players.to_vec(),
                encounters.to_vec(),
                iterations,
                false,
            ).0)
        },
    )?;
    let third_run_time = start.elapsed();
    
    println!("Different parameters run completed in {:?}", third_run_time);
    println!("Generated {} new simulation results", results3.len());
    
    // Updated storage statistics
    let stats = storage_manager.get_storage_stats();
    println!("\n=== Updated Storage Statistics ===");
    println!("Primary slot occupied: {}", stats.primary_slot_occupied);
    println!("Secondary slot occupied: {}", stats.secondary_slot_occupied);
    println!("Total cached simulations: {}", stats.total_cached_simulations);
    
    // Test cache clearing
    println!("\nTesting cache clearing...");
    storage_manager.clear_cache()?;
    let cleared_stats = storage_manager.get_storage_stats();
    println!("After clearing - Primary slot occupied: {}", cleared_stats.primary_slot_occupied);
    println!("After clearing - Secondary slot occupied: {}", cleared_stats.secondary_slot_occupied);
    
    println!("\n=== Demo completed successfully! ===");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    Ok(())
}