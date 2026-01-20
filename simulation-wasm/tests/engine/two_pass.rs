// Validation tests for Two-Pass Deterministic Re-simulation system
//
// These tests verify:
// 1. Seeded runs are deterministic (same seed = same results)
// 2. Re-simulated runs match original results
// 3. Decile selection accuracy (we get true percentiles)

use simulation_wasm::enums::EnemyTarget;
use simulation_wasm::model::*;
use simulation_wasm::{
    run_single_event_driven_simulation, run_single_lightweight_simulation, run_survey_pass,
    select_interesting_seeds_with_tiers,
};

fn create_test_creature(name: &str, hp: u32, ac: u32, damage_dice: &str, mode: &str) -> Creature {
    Creature {
        magic_items: vec![],
        max_arcane_ward_hp: None,
        initial_buffs: vec![],
        id: name.to_string(),
        name: name.to_string(),
        hp,
        ac,
        count: 1.0,
        mode: mode.to_string(),
        arrival: None,
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
        actions: vec![Action::Atk(AtkAction {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            dpr: DiceFormula::Expr(damage_dice.to_string()),
            target: EnemyTarget::EnemyWithMostHP,
            to_hit: DiceFormula::Value(5.0),
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        })],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    }
}

#[allow(dead_code)]
fn create_test_timeline(num_encounters: usize) -> Vec<TimelineStep> {
    (0..num_encounters)
        .map(|_| {
            TimelineStep::Combat(Encounter {
                monsters: vec![],
                players_surprised: None,
                monsters_surprised: None,
                players_precast: None,
                monsters_precast: None,
                target_role: TargetRole::Standard,
            })
        })
        .collect()
}

fn create_simple_timeline() -> Vec<TimelineStep> {
    vec![TimelineStep::Combat(Encounter {
        monsters: vec![create_test_creature("Monster", 20, 10, "1d4", "monster")],
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: TargetRole::Standard,
    })]
}

#[test]
fn test_lightweight_simulation_determinism() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let seed = 12345u64;

    // Run same seed twice
    let run1 = run_single_lightweight_simulation(&players, &timeline, seed);
    let run2 = run_single_lightweight_simulation(&players, &timeline, seed);

    // Should have identical results
    assert_eq!(run1.seed, run2.seed);
    assert_eq!(run1.encounter_scores, run2.encounter_scores);
    assert_eq!(run1.final_score, run2.final_score);
    assert_eq!(run1.has_death, run2.has_death);
}

#[test]
fn test_different_seeds_produce_different_results() {
    let players = vec![
        create_test_creature("Fighter", 50, 12, "1d20", "player"), // Very swingy damage
    ];
    // Need a monster to actually take damage and change scores
    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters: vec![create_test_creature("Monster", 100, 10, "1d20", "monster")], // High HP to survive multiple rounds
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: TargetRole::Standard,
    })];

    let mut scores = Vec::new();
    for seed in 0..10 {
        let run = run_single_lightweight_simulation(&players, &timeline, seed * 1000);
        scores.push(run.final_score);
    }

    scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
    scores.dedup();

    assert!(
        scores.len() >= 2,
        "Expected at least 2 different scores from different seeds, got {:?}",
        scores
    );
}

#[test]
fn test_survey_pass_returns_correct_count() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let iterations = 100;

    let results = run_survey_pass(players, timeline, iterations, None);

    assert_eq!(results.len(), iterations);
}

#[test]
fn test_survey_pass_seeds_are_unique() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let iterations = 100;
    let expected = 100;

    let results = run_survey_pass(players, timeline, iterations, None);

    // All seeds should be unique
    let seeds: std::collections::HashSet<_> = results.iter().map(|r| r.seed).collect();
    assert_eq!(seeds.len(), expected, "All seeds should be unique");
}

#[test]
fn test_decile_selection_includes_extremes() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let iterations = 100;

    let results = run_survey_pass(players, timeline, iterations, None);
    let interesting_seeds = select_interesting_seeds_with_tiers(&results);

    // Find the best and worst runs
    let mut sorted = results.clone();
    sorted.sort_by(|a, b| a.final_score.partial_cmp(&b.final_score).unwrap());

    let worst_seed = sorted.first().map(|r| r.seed).unwrap();
    let best_seed = sorted.last().map(|r| r.seed).unwrap();

    // Extract seed values from SelectedSeed for comparison
    let seed_values: Vec<u64> = interesting_seeds.iter().map(|s| s.seed).collect();

    // Both extremes should be included
    assert!(
        seed_values.contains(&worst_seed),
        "Worst run should be in interesting seeds"
    );
    assert!(
        seed_values.contains(&best_seed),
        "Best run should be in interesting seeds"
    );
}

#[test]
fn test_decile_selection_size_is_reasonable() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let iterations = 101; // Use 10n + 1

    let results = run_survey_pass(players, timeline, iterations, None);
    let interesting_seeds = select_interesting_seeds_with_tiers(&results);

    // Now we select 11 global deciles + 100 1% bucket medians + per-encounter extremes
    // For 1 encounter: 11 (Tier A) + 100 (Tier B) + 3 (Tier C) = ~114 with possible overlaps
    let expected_max = 150;

    assert!(
        interesting_seeds.len() <= expected_max,
        "Interesting seeds ({}) should not exceed expected max ({})",
        interesting_seeds.len(),
        expected_max
    );

    // Should have at least the 11 decile seeds
    assert!(
        interesting_seeds.len() >= 11,
        "Should have at least 11 interesting seeds"
    );
}

#[test]
fn test_re_simulation_matches_lightweight_scores() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let seed = 54321u64;

    // Run lightweight version
    let lightweight = run_single_lightweight_simulation(&players, &timeline, seed);

    // Run full version with same seed
    simulation_wasm::rng::seed_rng(seed);
    let (full_result, _events) = run_single_event_driven_simulation(&players, &timeline, false);
    let full_score = simulation_wasm::aggregation::calculate_score(&full_result);
    simulation_wasm::rng::clear_rng();

    // Scores should match exactly
    assert_eq!(
        lightweight.final_score, full_score,
        "Lightweight score ({}) should match full simulation score ({})",
        lightweight.final_score, full_score
    );
}

#[test]
fn test_encounter_scores_cumulative() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    // Create timeline with 3 encounters
    let timeline: Vec<TimelineStep> = (0..3)
        .map(|_| {
            TimelineStep::Combat(Encounter {
                monsters: vec![],
                players_surprised: None,
                monsters_surprised: None,
                players_precast: None,
                monsters_precast: None,
                target_role: TargetRole::Standard,
            })
        })
        .collect();

    let result = run_single_lightweight_simulation(&players, &timeline, 99999);

    // Should have 3 encounter scores
    assert_eq!(result.encounter_scores.len(), 3);

    // Each score should be non-negative (survivors)
    for score in &result.encounter_scores {
        assert!(*score >= 0.0, "Encounter scores should be non-negative");
    }
}

#[test]
fn test_lightweight_run_tracks_deaths() {
    // Create a scenario where deaths are highly likely
    let players = vec![
        create_test_creature("WeakFighter", 5, 5, "1d1", "player"), // Very low HP/AC
    ];

    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters: vec![create_test_creature(
            "DeadlyMonster",
            200,
            20,
            "20d20",
            "monster",
        )], // Absolute unit
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
        target_role: TargetRole::Standard,
    })];

    // Run multiple times to find a death scenario
    let mut found_death = false;
    for seed in 0..50 {
        let result = run_single_lightweight_simulation(&players, &timeline, seed + 999);
        if result.has_death {
            found_death = true;
            break;
        }
    }

    assert!(
        found_death,
        "Should find at least one death in 50 runs with absolute unit monster"
    );
}

#[test]
fn test_two_pass_consistency() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();
    let iterations = 100;
    let expected = 100;
    let base_seed = Some(77777);

    // Run survey pass
    let lightweight_runs =
        run_survey_pass(players.clone(), timeline.clone(), iterations, base_seed);
    assert_eq!(lightweight_runs.len(), expected);

    // Select interesting seeds
    let interesting_seeds = select_interesting_seeds_with_tiers(&lightweight_runs);

    // For each interesting seed, verify full re-simulation matches
    for selected_seed in interesting_seeds {
        let seed = selected_seed.seed;
        let lightweight = lightweight_runs.iter().find(|r| r.seed == seed).expect(
            "Seed from select_interesting_seeds_with_tiers should exist in lightweight_runs",
        );

        simulation_wasm::rng::seed_rng(seed);
        let (full_result, _events) = run_single_event_driven_simulation(&players, &timeline, false);
        let full_score = simulation_wasm::aggregation::calculate_score(&full_result);
        simulation_wasm::rng::clear_rng();

        assert_eq!(
            lightweight.final_score, full_score,
            "Seed {}: lightweight score {} should match full score {}",
            seed, lightweight.final_score, full_score
        );
    }
}

#[test]
fn test_lightweight_run_serialization() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d8+3", "player")];
    let timeline = create_simple_timeline();

    let run = run_single_lightweight_simulation(&players, &timeline, 42);

    // Should be serializable
    let serialized = serde_json::to_string(&run).expect("LightweightRun should be serializable");
    let deserialized: LightweightRun =
        serde_json::from_str(&serialized).expect("LightweightRun should be deserializable");

    assert_eq!(run.seed, deserialized.seed);
    assert_eq!(run.encounter_scores, deserialized.encounter_scores);
    assert_eq!(run.final_score, deserialized.final_score);
    assert_eq!(run.has_death, deserialized.has_death);
}

#[test]
fn test_decile_approximation_accuracy() {
    let players = vec![create_test_creature("Fighter", 50, 15, "1d10+5", "player")];
    let timeline = create_simple_timeline();
    let iterations = 200;
    let seed = 123456;

    // 1. Run with rolling stats (Two-Pass)
    let summary = simulation_wasm::run_simulation_with_rolling_stats(
        players.clone(),
        timeline.clone(),
        iterations,
        false,
        Some(seed),
    );

    // 2. Run full simulation (One-Pass) manually for comparison
    let mut full_scores = Vec::new();
    for i in 0..iterations {
        simulation_wasm::rng::seed_rng(seed + i as u64);
        let (res, _) =
            simulation_wasm::run_single_event_driven_simulation(&players, &timeline, false);
        full_scores.push(simulation_wasm::aggregation::calculate_score(&res));
    }
    simulation_wasm::rng::clear_rng();
    full_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let true_median = full_scores[iterations / 2];
    let true_p25 = full_scores[iterations / 4];
    let true_p75 = full_scores[iterations * 3 / 4];

    // Approximation should be within 2.0 (due to 100-bucket quantization for scores 0-100)
    let tolerance = 2.0;

    assert!(
        (summary.score_percentiles.median - true_median).abs() <= tolerance,
        "Median approximation {} should be close to true median {}",
        summary.score_percentiles.median,
        true_median
    );

    assert!(
        (summary.score_percentiles.p25 - true_p25).abs() <= tolerance,
        "P25 approximation {} should be close to true p25 {}",
        summary.score_percentiles.p25,
        true_p25
    );

    assert!(
        (summary.score_percentiles.p75 - true_p75).abs() <= tolerance,
        "P75 approximation {} should be close to true p75 {}",
        summary.score_percentiles.p75,
        true_p75
    );
}
