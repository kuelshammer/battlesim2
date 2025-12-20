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

fn run_comprehensive_analysis(scenario_file: &str, expected_winner_is_player: bool) {
    println!("\n{}", "=".repeat(80));
    println!("COMPREHENSIVE ANALYSIS FOR: {}", scenario_file);
    println!("{}", "=".repeat(80));
    
    let (players, encounters, _) = load_scenario(scenario_file);
    let iterations = 1000; // Increased for better statistical significance
    
    println!("Running {} simulations...", iterations);
    
    // Run simulation
    let (mut results, _) = run_event_driven_simulation_rust(players, encounters, iterations, false);
    
    // Check for -1000000 score issue
    let mut negative_scores = 0;
    let mut total_score = 0.0;
    let mut min_score = f64::MAX;
    let mut max_score = f64::MIN;
    
    for result in &results {
        let score = simulation_wasm::aggregation::calculate_score(result);
        total_score += score;
        min_score = min_score.min(score);
        max_score = max_score.max(score);
        
        if score <= -1000000.0 {
            negative_scores += 1;
        }
    }
    
    println!("\nSCORE ANALYSIS:");
    println!("  Total simulations: {}", results.len());
    println!("  Average score: {:.2}", total_score / results.len() as f64);
    println!("  Min score: {:.2}", min_score);
    println!("  Max score: {:.2}", max_score);
    println!("  Scores <= -1000000: {} ({:.1}%)", negative_scores, (negative_scores as f64 / results.len() as f64) * 100.0);
    
    // Sort results (required for quintile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1; // Assuming 1v1 for these tests
    let analysis = run_quintile_analysis(&results, scenario_file, party_size);
    
    println!("\nENCOUNTER RATING:");
    println!("  Risk Factor: {:?}", analysis.risk_factor);
    println!("  Difficulty: {:?}", analysis.difficulty);
    println!("  Encounter Label: {:?}", analysis.encounter_label);
    println!("  Analysis: {}", analysis.analysis_summary);
    
    if !analysis.tuning_suggestions.is_empty() {
        println!("  Tuning Suggestions:");
        for suggestion in &analysis.tuning_suggestions {
            println!("    - {}", suggestion);
        }
    }
    
    println!("\nQUINTILE BREAKDOWN:");
    println!("{:<15} {:<10} {:<10} {:<15} {:<15} {:<10}", 
             "Quintile", "Win Rate", "Survivors", "HP Lost %", "Battle Duration", "Status");
    println!("{}", "-".repeat(80));
    
    for quintile in &analysis.quintiles {
        let status = match expected_winner_is_player {
            true if quintile.win_rate > 50.0 => "✓ PASS",
            false if quintile.win_rate < 50.0 => "✓ PASS", 
            _ => "✗ FAIL",
        };
        
        println!("{:<15} {:<10.1}% {:<10}/{} {:<15.1}% {:<15} {:<10}", 
                 quintile.label,
                 quintile.win_rate,
                 quintile.median_survivors,
                 quintile.party_size,
                 quintile.hp_lost_percent,
                 quintile.battle_duration_rounds,
                 status);
    }
    
    // Test validation
    let median_quintile = &analysis.quintiles[2]; // Index 2 is Median
    println!("\nREGRESSION TEST VALIDATION:");
    println!("  Expected winner: {}", if expected_winner_is_player { "Player" } else { "Monster" });
    println!("  Median Win Rate: {:.1}%", median_quintile.win_rate);
    println!("  Median Survivors: {}/{}", median_quintile.median_survivors, median_quintile.party_size);
    
    let test_passed = if expected_winner_is_player {
        median_quintile.win_rate > 50.0
    } else {
        median_quintile.win_rate < 50.0
    };
    
    println!("  Test Result: {}", if test_passed { "✓ PASSED" } else { "✗ FAILED" });
    
    // Additional analysis
    println!("\nDETAILED ANALYSIS:");
    println!("  Worst 20% Win Rate: {:.1}%", analysis.quintiles[0].win_rate);
    println!("  Best 20% Win Rate: {:.1}%", analysis.quintiles[4].win_rate);
    println!("  Win Rate Variance: {:.1}%", analysis.quintiles[4].win_rate - analysis.quintiles[0].win_rate);
    
    // Check for reliability issues
    let reliability_issues = negative_scores > 0 || 
                           (analysis.quintiles[4].win_rate - analysis.quintiles[0].win_rate) > 80.0;
    
    if reliability_issues {
        println!("  ⚠️  RELIABILITY ISSUES DETECTED:");
        if negative_scores > 0 {
            println!("    - {} simulations with scores <= -1000000", negative_scores);
        }
        if (analysis.quintiles[4].win_rate - analysis.quintiles[0].win_rate) > 80.0 {
            println!("    - High variance ({:.1}% spread) indicates unstable simulation", 
                     analysis.quintiles[4].win_rate - analysis.quintiles[0].win_rate);
        }
    } else {
        println!("  ✓ No significant reliability issues detected");
    }
}

fn main() {
    println!("BATTLESIM2 REGRESSION TEST SUITE - COMPREHENSIVE QUINTILE ANALYSIS");
    println!("=================================================================");
    
    // Run all regression tests with comprehensive analysis
    run_comprehensive_analysis("fast_init_PlayerA_wins.json", true);
    run_comprehensive_analysis("damage_vs_precision_MonsterB_wins.json", false);
    run_comprehensive_analysis("heavy_vs_consistent_PlayerA_wins.json", true);
    
    println!("\n{}", "=".repeat(80));
    println!("REGRESSION TEST SUITE COMPLETED");
    println!("{}", "=".repeat(80));
}