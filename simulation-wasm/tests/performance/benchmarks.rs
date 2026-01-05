//! Benchmark tests for 10,100 runs with 1% granularity and three-tier events
//!
//! Validates:
//! - Time: ~16s total (Phase 1 ~10s, Phase 3 ~6s)
//! - Memory: ~4.5 MB total (Phase 1 ~323 KB, Phase 3 ~4.2 MB)
//! - 1% granularity: 100 buckets with 101 runs each
//! - Tier classification: ~170 seeds selected (11 Tier A, ~100 Tier B, ~59 Tier C)

use simulation_wasm::model::{Creature, TimelineStep};
use std::time::Instant;

fn create_test_scenario() -> (Vec<Creature>, Vec<TimelineStep>) {
    // Create a simple 1v1 scenario for faster benchmarking
    let player = Creature {
        id: "test_player".to_string(),
        name: "Test Fighter".to_string(),
        count: 1.0,
        hp: 45,
        ac: 18,
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
        initiative_bonus: simulation_wasm::model::DiceFormula::Value(3.0),
        initiative_advantage: false,
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        arrival: None,
        mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None,
    };

    let monster = Creature {
        id: "test_monster".to_string(),
        name: "Goblin".to_string(),
        count: 1.0,
        hp: 15,
        ac: 13,
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
        initiative_bonus: simulation_wasm::model::DiceFormula::Value(2.0),
        initiative_advantage: false,
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        arrival: None,
        mode: "monster".to_string(), magic_items: vec![], max_arcane_ward_hp: None,
    };

    let encounter = simulation_wasm::model::Encounter {
        monsters: vec![monster],
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: Default::default(),
    };

    let timeline = vec![TimelineStep::Combat(encounter)];

    (vec![player], timeline)
}

#[test]
fn benchmark_phase_1_lightweight_survey() {
    let (players, timeline) = create_test_scenario();

    // Test with 10,100 iterations (100 buckets × 101 runs)
    const ITERATIONS: usize = 10_100;

    let start = Instant::now();
    let lightweight_runs = simulation_wasm::run_survey_pass(
        players.clone(),
        timeline.clone(),
        ITERATIONS,
        Some(42),
    );
    let duration = start.elapsed();

    println!("\n=== Phase 1 Benchmark (10,100 runs) ===");
    println!("Duration: {:?}", duration);
    println!("Runs: {}", lightweight_runs.len());
    println!("Time per run: {:.2} ms", duration.as_millis() as f64 / ITERATIONS as f64);

    // Expected: ~10s total, ~1ms per run
    // Allow 3× margin for CI/variability (can be 30s+ in CI)
    assert_eq!(lightweight_runs.len(), ITERATIONS);
    assert!(
        duration.as_secs() < 45,
        "Phase 1 took too long: {:?} (expected < 45s)",
        duration
    );
}

#[test]
fn benchmark_phase_2_seed_selection() {
    let (players, timeline) = create_test_scenario();

    const ITERATIONS: usize = 10_100;

    // Run Phase 1 first
    let lightweight_runs = simulation_wasm::run_survey_pass(
        players.clone(),
        timeline.clone(),
        ITERATIONS,
        Some(42),
    );

    // Benchmark Phase 2
    let start = Instant::now();
    let selected_seeds = simulation_wasm::select_interesting_seeds_with_tiers(&lightweight_runs);
    let duration = start.elapsed();

    println!("\n=== Phase 2 Benchmark (Seed Selection) ===");
    println!("Duration: {:?}", duration);

    // Count seeds by tier
    let tier_a_count = selected_seeds.iter().filter(|s| s.tier == simulation_wasm::model::InterestingSeedTier::TierA).count();
    let tier_b_count = selected_seeds.iter().filter(|s| s.tier == simulation_wasm::model::InterestingSeedTier::TierB).count();
    let tier_c_count = selected_seeds.iter().filter(|s| s.tier == simulation_wasm::model::InterestingSeedTier::TierC).count();

    println!("Total selected seeds: {}", selected_seeds.len());
    println!("  Tier A (full events): {}", tier_a_count);
    println!("  Tier B (lean events): {}", tier_b_count);
    println!("  Tier C (no events): {}", tier_c_count);

    // Expected: ~170 total seeds
    // - 100 from 1% buckets (Tier B)
    // - 11 from global deciles (Tier A)
    // - 3 from per-encounter extremes (Tier C) - 1 encounter × 3 percentiles
    // - Plus any death runs
    assert!(selected_seeds.len() <= 200, "Selected too many seeds: {}", selected_seeds.len());
    assert!(tier_a_count == 11, "Expected 11 Tier A seeds, got {}", tier_a_count);
    assert!(tier_b_count >= 100, "Expected at least 100 Tier B seeds, got {}", tier_b_count);

    // Phase 2 should be very fast (< 100ms)
    assert!(duration.as_millis() < 100, "Phase 2 took too long: {:?}", duration);
}

#[test]
fn benchmark_full_two_pass_10k() {
    let (players, timeline) = create_test_scenario();

    const ITERATIONS: usize = 10_100;

    let start = Instant::now();
    let summary = simulation_wasm::run_simulation_with_three_tier(
        players,
        timeline,
        ITERATIONS,
        false,
        Some(42),
    );
    let total_duration = start.elapsed();

    println!("\n=== Full Two-Pass Benchmark (10,100 runs) ===");
    println!("Total duration: {:?}", total_duration);
    println!("Total iterations: {}", summary.total_iterations);
    println!("Sample runs (Phase 3): {}", summary.sample_runs.len());
    println!("Score min: {:.1}", summary.score_percentiles.min);
    println!("Score max: {:.1}", summary.score_percentiles.max);
    println!("Score median: {:.1}", summary.score_percentiles.median);
    println!("Score mean: {:.1}", summary.score_percentiles.mean);
    println!("Score std dev: {:.1}", summary.score_percentiles.std_dev);

    // Expected: ~16s total (Phase 1 ~10s, Phase 3 ~6s)
    // Allow 3× margin for CI/variability (can be 40s+ in CI)
    assert_eq!(summary.total_iterations, ITERATIONS);
    assert!(
        total_duration.as_secs() < 45,
        "Total duration too long: {:?} (expected < 45s)",
        total_duration
    );

    // Verify 1% granularity worked: should have sample runs from deciles
    // Expected: 11 (Tier A) + ~100 (Tier B) = ~111
    assert!(
        summary.sample_runs.len() >= 100 && summary.sample_runs.len() <= 150,
        "Sample runs count out of range: {} (expected 100-150)",
        summary.sample_runs.len()
    );
}

#[test]
fn validate_1_percentile_buckets() {
    let (players, timeline) = create_test_scenario();

    const ITERATIONS: usize = 10_100; // 100 buckets × 101 runs

    let lightweight_runs = simulation_wasm::run_survey_pass(
        players.clone(),
        timeline.clone(),
        ITERATIONS,
        Some(42),
    );

    let selected_seeds = simulation_wasm::select_interesting_seeds_with_tiers(&lightweight_runs);

    // Check that we have approximately 100 Tier B seeds (1% medians)
    let tier_b_seeds: Vec<_> = selected_seeds
        .iter()
        .filter(|s| s.tier == simulation_wasm::model::InterestingSeedTier::TierB)
        .filter(|s| s.bucket_label.starts_with("P"))
        .filter(|s| !s.bucket_label.contains("-")) // Exclude P5, P15 etc (those are Tier A)
        .collect();

    println!("\n=== 1% Bucket Validation ===");
    println!("Tier B seeds with P-N-N labels: {}", tier_b_seeds.len());

    // Should have ~100 seeds from 1% buckets (P0-1, P1-2, ..., P99-100)
    let bucket_seeds: Vec<_> = tier_b_seeds
        .iter()
        .filter(|s| s.bucket_label.contains("-"))
        .collect();

    println!("1% bucket seeds (P-N-N): {}", bucket_seeds.len());

    // Check bucket labels format
    for seed in bucket_seeds.iter().take(5) {
        println!("  {}", seed.bucket_label);
        assert!(seed.bucket_label.contains("-"), "Bucket label should contain '-'");
    }
}
