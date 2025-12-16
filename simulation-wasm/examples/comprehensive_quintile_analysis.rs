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

fn run_comprehensive_quintile_analysis(scenario_file: &str) {
    println!("\n" + "=".repeat(80).as_str());
    println!("COMPREHENSIVE QUINTILE ANALYSIS: {}", scenario_file);
    println!("=".repeat(80));
    
    let (players, encounters, _) = load_scenario(scenario_file);
    let iterations = 1005; // Higher number for better quintile resolution
    
    println!("Running {} simulation iterations...", iterations);
    
    // Run simulation
    let start_time = std::time::Instant::now();
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);
    let simulation_time = start_time.elapsed();
    
    // Sort results (required for quintile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1; // Assuming 1v1 for these tests
    let analysis = run_quintile_analysis(&results, scenario_file, party_size);
    
    // Performance metrics
    println!("Performance Metrics:");
    println!("  - Simulation Time: {:?}", simulation_time);
    println!("  - Iterations per Second: {:.0}", iterations as f64 / simulation_time.as_secs_f64());
    println!("  - Total Results: {}", results.len());
    
    println!("\nQuintile Analysis Results:");
    println!("{:<12} {:<12} {:<15} {:<15} {:<15} {:<15}", 
             "Quintile", "Win Rate", "Avg HP Remaining", "Median HP", "Battle Duration", "Success Rate");
    println!("{}", "-".repeat(90));
    
    for quintile in &analysis.quintiles {
        println!("{:<12} {:<11.1}% {:<14.1} {:<14.1} {:<14.1} rounds {:<14.1}%", 
                 quintile.label,
                 quintile.win_rate,
                 quintile.avg_hp_remaining,
                 quintile.median_hp_remaining,
                 quintile.battle_duration_rounds,
                 quintile.success_rate * 100.0);
    }
    
    // Detailed analysis for each quintile
    println!("\nDetailed Quintile Breakdown:");
    for (i, quintile) in analysis.quintiles.iter().enumerate() {
        println!("\n  Quintile {}: {} (Runs {}-{})", 
                 i + 1, quintile.label, 
                 quintile.start_run + 1, 
                 quintile.end_run + 1);
        
        println!("    - Win Rate: {:.1}%", quintile.win_rate);
        println!("    - Success Rate: {:.1}%", quintile.success_rate * 100.0);
        println!("    - Average HP Remaining: {:.1}", quintile.avg_hp_remaining);
        println!("    - Median HP Remaining: {:.1}", quintile.median_hp_remaining);
        println!("    - Battle Duration: {:.1} rounds", quintile.battle_duration_rounds);
        println!("    - Risk Factor: {:?}", quintile.risk_factor);
        println!("    - Difficulty: {:?}", quintile.difficulty);
        
        // Show median run visualization
        if !quintile.median_run_visualization.is_empty() {
            println!("    - Median Run Final State:");
            for combatant in &quintile.median_run_visualization {
                let status = if combatant.is_dead { "DEAD" } else { "ALIVE" };
                println!("      * {}: {}/{} HP ({})", 
                         combatant.name, 
                         combatant.current_hp, 
                         combatant.max_hp, 
                         status);
            }
        }
    }
    
    // Error patterns and reliability analysis
    println!("\nReliability Analysis:");
    println!("  - Total Successful Runs: {}", analysis.quintiles.iter().map(|q| q.success_count).sum::<usize>());
    println!("  - Total Failed Runs: {}", analysis.quintiles.iter().map(|q| q.failure_count).sum::<usize>());
    
    let total_success_rate = analysis.quintiles.iter().map(|q| q.success_rate).sum::<f64>() / 5.0;
    println!("  - Overall Success Rate: {:.1}%", total_success_rate * 100.0);
    
    // Check for any concerning patterns
    let worst_quintile = &analysis.quintiles[0];
    let best_quintile = &analysis.quintiles[4];
    
    println!("\nPattern Analysis:");
    if worst_quintile.win_rate < 10.0 {
        println!("  ⚠️  Worst 20% shows extremely low win rate ({:.1}%) - possible balance issues", worst_quintile.win_rate);
    }
    
    if best_quintile.win_rate < 80.0 {
        println!("  ⚠️  Best 20% has low win rate ({:.1}%) - scenario may be too difficult", best_quintile.win_rate);
    }
    
    let hp_variance = best_quintile.avg_hp_remaining - worst_quintile.avg_hp_remaining;
    if hp_variance > 50.0 {
        println!("  📊 High HP variance ({:.1}) between worst and best quintiles - high randomness factor", hp_variance);
    }
    
    let duration_variance = best_quintile.battle_duration_rounds - worst_quintile.battle_duration_rounds;
    if duration_variance > 10.0 {
        println!("  ⏱️  High duration variance ({:.1} rounds) - inconsistent battle lengths", duration_variance);
    }
    
    // Summary assessment
    println!("\nSummary Assessment:");
    if total_success_rate >= 0.95 && worst_quintile.win_rate >= 20.0 {
        println!("  ✅ Scenario is well-balanced with good reliability");
    } else if total_success_rate >= 0.80 {
        println!("  ⚠️  Scenario is moderately balanced but may need tuning");
    } else {
        println!("  ❌ Scenario shows significant balance or reliability issues");
    }
}

fn main() {
    println!("BATTLESIM2 COMPREHENSIVE REGRESSION TEST & QUINTILE ANALYSIS");
    println!("=========================================================");
    
    let scenarios = [
        "fast_init_PlayerA_wins.json",
        "damage_vs_precision_MonsterB_wins.json", 
        "heavy_vs_consistent_PlayerA_wins.json",
        "coin_flip_working.json",
        "volatile_crit_test.json"
    ];
    
    for scenario in &scenarios {
        run_comprehensive_quintile_analysis(scenario);
    }
    
    println!("\n" + "=".repeat(80).as_str());
    println!("REGRESSION TEST SUMMARY");
    println!("=".repeat(80));
    
    // Run the actual regression tests
    println!("\nRunning standard regression tests...");
    
    // Test 1: Fast Initiative Dominance
    println!("\n1. Fast Initiative Dominance Test:");
    run_comprehensive_quintile_analysis("fast_init_PlayerA_wins.json");
    let (players, encounters, _) = load_scenario("fast_init_PlayerA_wins.json");
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, 505, false);
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));
    let analysis = run_quintile_analysis(&results, "fast_init_PlayerA_wins.json", 1);
    let median_quintile = &analysis.quintiles[2];
    
    if median_quintile.win_rate > 50.0 {
        println!("  ✅ PASS: Player wins as expected (Median win rate: {:.1}%)", median_quintile.win_rate);
    } else {
        println!("  ❌ FAIL: Expected player win but got {:.1}% win rate", median_quintile.win_rate);
    }
    
    // Test 2: Damage vs Precision
    println!("\n2. Damage vs Precision Test:");
    let (players, encounters, _) = load_scenario("damage_vs_precision_MonsterB_wins.json");
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, 505, false);
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));
    let analysis = run_quintile_analysis(&results, "damage_vs_precision_MonsterB_wins.json", 1);
    let median_quintile = &analysis.quintiles[2];
    
    if median_quintile.win_rate < 50.0 {
        println!("  ✅ PASS: Monster wins as expected (Median win rate: {:.1}%)", median_quintile.win_rate);
    } else {
        println!("  ❌ FAIL: Expected monster win but got {:.1}% win rate", median_quintile.win_rate);
    }
    
    // Test 3: Heavy vs Consistent
    println!("\n3. Heavy vs Consistent Test:");
    let (players, encounters, _) = load_scenario("heavy_vs_consistent_PlayerA_wins.json");
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, 505, false);
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));
    let analysis = run_quintile_analysis(&results, "heavy_vs_consistent_PlayerA_wins.json", 1);
    let median_quintile = &analysis.quintiles[2];
    
    if median_quintile.win_rate > 50.0 {
        println!("  ✅ PASS: Player wins as expected (Median win rate: {:.1}%)", median_quintile.win_rate);
    } else {
        println!("  ❌ FAIL: Expected player win but got {:.1}% win rate", median_quintile.win_rate);
    }
    
    println!("\n" + "=".repeat(80).as_str());
    println!("FINAL SUMMARY");
    println!("=".repeat(80));
    println!("All regression tests completed with detailed quintile analysis.");
    println!("Each scenario was evaluated across 5 quintiles to ensure reliability and balance.");
}