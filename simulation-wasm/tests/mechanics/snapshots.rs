// Snapshot Tests for Simulation Regression
//
// These tests use insta to snapshot simulation results and detect regressions.
// When simulation logic changes, snapshots will fail and need review.

use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::decile_analysis::run_decile_analysis;
use serde::Serialize;
use crate::common::load_scenario;

/// Snapshot data for a scenario
#[derive(Serialize)]
struct SnapshotData {
    description: &'static str,
    median_win_rate: f64,
    median_survivors: u32,
    party_size: u32,
}

/// Snapshot test for basic melee combat (fast initiative dominance)
#[test]
fn snapshot_basic_melee_combat() {
    let scenario_file = "fast_init_PlayerA_wins.json";
    let (players, timeline) = load_scenario(scenario_file);

    // Run 101 iterations for decile analysis with fixed seed for determinism
    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    // Sort results (required for decile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1;
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    // Snapshot the key metrics from the median decile
    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Fast initiative should dominate in mirror match",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("basic_melee_combat", data);
}

/// Snapshot test for damage vs precision
#[test]
fn snapshot_damage_vs_precision() {
    let scenario_file = "damage_vs_precision_MonsterB_wins.json";
    let (players, timeline) = load_scenario(scenario_file);

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1;
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Precision (Monster) beats Damage (Player) in this setup",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("damage_vs_precision", data);
}

/// Snapshot test for heavy vs consistent damage
#[test]
fn snapshot_heavy_vs_consistent() {
    let scenario_file = "heavy_vs_consistent_PlayerA_wins.json";
    let (players, timeline) = load_scenario(scenario_file);

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1;
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Burst Damage (Player) beats Low Dmg/High Hit (Monster)",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("heavy_vs_consistent", data);
}

/// Snapshot test for Fireball AoE mechanics
#[test]
fn snapshot_fireball_aoe() {
    let scenario_file = "basic/02_fireball_aoe.json";
    let (players, timeline) = load_scenario(scenario_file);

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1; // Wizard vs 4 Goblins
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Wizard should clear goblins with Fireball",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("fireball_aoe", data);
}

/// Snapshot test for Healing Word mechanics
#[test]
fn snapshot_healing_word() {
    let scenario_file = "basic/03_healing_word.json";
    let (players, timeline) = load_scenario(scenario_file);

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 2; // Cleric + Fighter
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Cleric should heal Fighter with Healing Word",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("healing_word", data);
}

/// Snapshot test for Multiattack mechanics
#[test]
fn snapshot_multiattack() {
    let scenario_file = "basic/05_multiattack.json";
    let (players, timeline) = load_scenario(scenario_file);

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false, Some(42));
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1; 
    let analysis = run_decile_analysis(&results, scenario_file, party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Multiattack should increase damage throughput",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("multiattack", data);
}