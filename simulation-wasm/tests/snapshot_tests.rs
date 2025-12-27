// Snapshot Tests for Simulation Regression
//
// These tests use insta to snapshot simulation results and detect regressions.
// When simulation logic changes, snapshots will fail and need review.

use simulation_wasm::model::{Creature, TimelineStep};
use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::decile_analysis::run_decile_analysis;
use std::fs;
use std::path::PathBuf;
use serde::Serialize;

/// Load a scenario from the tests/scenarios directory
fn load_scenario(filename: &str) -> (Vec<Creature>, Vec<TimelineStep>) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/scenarios");
    path.push(filename);

    let content = fs::read_to_string(&path).expect(&format!("Failed to read scenario file: {:?}", path));
    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let players: Vec<Creature> =
        serde_json::from_value(data["players"].clone()).expect("Failed to parse players");

    let timeline: Vec<TimelineStep> = if let Some(t) = data.get("timeline") {
        serde_json::from_value(t.clone()).expect("Failed to parse timeline")
    } else {
        let encounters: Vec<simulation_wasm::model::Encounter> =
            serde_json::from_value(data["encounters"].clone()).expect("Failed to parse encounters");
        encounters.into_iter().map(TimelineStep::Combat).collect()
    };

    (players, timeline)
}

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
    let (players, timeline) = load_scenario("fast_init_PlayerA_wins.json");

    // Run 101 iterations for decile analysis
    let runs = run_event_driven_simulation_rust(players, timeline, 101, false);
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    // Sort results (required for decile analysis)
    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    // Analyze
    let party_size = 1;
    let analysis = run_decile_analysis(&results, "fast_init_PlayerA_wins.json", party_size);

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
    let (players, timeline) = load_scenario("damage_vs_precision_MonsterB_wins.json");

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false);
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1;
    let analysis = run_decile_analysis(&results, "damage_vs_precision_MonsterB_wins.json", party_size);

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
    let (players, timeline) = load_scenario("heavy_vs_consistent_PlayerA_wins.json");

    let runs = run_event_driven_simulation_rust(players, timeline, 101, false);
    let mut results: Vec<_> = runs.into_iter().map(|r| r.result).collect();

    results.sort_by(|a, b| simulation_wasm::aggregation::calculate_score(a)
        .partial_cmp(&simulation_wasm::aggregation::calculate_score(b))
        .unwrap_or(std::cmp::Ordering::Equal));

    let party_size = 1;
    let analysis = run_decile_analysis(&results, "heavy_vs_consistent_PlayerA_wins.json", party_size);

    let median = &analysis.deciles[4];
    let data = SnapshotData {
        description: "Burst Damage (Player) beats Low Dmg/High Hit (Monster)",
        median_win_rate: median.win_rate,
        median_survivors: median.median_survivors as u32,
        party_size: median.party_size as u32,
    };
    insta::assert_json_snapshot!("heavy_vs_consistent", data);
}
