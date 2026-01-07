use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::model::{Creature, TimelineStep, TargetRole, Encounter, DiceFormula};
use simulation_wasm::decile_analysis::run_decile_analysis;
use std::time::{Duration, Instant};

// Configuration for stress tests via environment variables
// Pre-commit: STRESS_TIME_SECS=10 STRESS_COUNT=100
// CI: STRESS_TIME_SECS=60 STRESS_COUNT=1000
fn get_stress_config() -> (u64, usize) {
    let time_secs: u64 = std::env::var("STRESS_TIME_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let count: usize = std::env::var("STRESS_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    (time_secs, count)
}

#[test]
fn test_fast_simulation_iterations() {
    let player = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 15,
        count: 1.0,
        mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
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
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    let monster = Creature {
        id: "m1".to_string(),
        name: "Monster".to_string(),
        hp: 50,
        ac: 12,
        count: 1.0,
        mode: "monster".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
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
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    let players = vec![player];

    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters: vec![monster],
        target_role: TargetRole::Standard,
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
    })];

    let iterations = 31;
    let expected = 100;
    let runs = run_event_driven_simulation_rust(players.clone(), timeline.clone(), iterations, false, None);

    assert_eq!(runs.len(), expected, "Should have exactly 100 runs (minimum enforced)");

    // Check decile analysis works with 100 runs
    let results: Vec<_> = runs.iter().map(|r| r.result.clone()).collect();
    let analysis = run_decile_analysis(&results, "Fast Test", 1, 0);

    assert_eq!(analysis.deciles.len(), 10, "Should produce 10 deciles");
    
    // Check that we have events for at least the first run
    assert!(!runs[0].events.is_empty(), "First run should have events");
}

#[test]
fn test_heavy_load_simulation() {
    // 4 Players vs 10 Monsters
    let mut players = Vec::new();
    for i in 1..=4 {
        players.push(Creature {
            id: format!("p{}", i),
            name: format!("Player {}", i),
            hp: 100,
            ac: 18,
            count: 1.0,
            mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
            arrival: None,
            speed_fly: None,
            save_bonus: 2.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(3.0),
            initiative_advantage: false,
            actions: vec![
                simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {
                    id: "atk".to_string(),
                    name: "Attack".to_string(),
                    action_slot: None,
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![],
                    freq: simulation_wasm::model::Frequency::Static("at will".to_string()),
                    condition: simulation_wasm::model::ActionCondition::Default,
                    targets: 1,
                    dpr: DiceFormula::Expr("2d6+4".to_string()),
                    target: simulation_wasm::enums::EnemyTarget::EnemyWithMostHP,
                    to_hit: DiceFormula::Value(7.0),
                    use_saves: None,
                    half_on_save: None,
                    rider_effect: None,
                })
            ],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        });
    }

    let mut monsters = Vec::new();
    for i in 1..=10 {
        monsters.push(Creature {
            id: format!("m{}", i),
            name: format!("Monster {}", i),
            hp: 30,
            ac: 13,
            count: 1.0,
            mode: "monster".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
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
            initiative_bonus: DiceFormula::Value(1.0),
            initiative_advantage: false,
            actions: vec![
                simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {
                    id: "atk".to_string(),
                    name: "Bite".to_string(),
                    action_slot: None,
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![],
                    freq: simulation_wasm::model::Frequency::Static("at will".to_string()),
                    condition: simulation_wasm::model::ActionCondition::Default,
                    targets: 1,
                    dpr: DiceFormula::Expr("1d8+2".to_string()),
                    target: simulation_wasm::enums::EnemyTarget::EnemyWithLeastHP,
                    to_hit: DiceFormula::Value(4.0),
                    use_saves: None,
                    half_on_save: None,
                    rider_effect: None,
                })
            ],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        });
    }

    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters,
        target_role: TargetRole::Standard,
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
    })];

        // 5000 iterations is a good stress test for native (WASM would be slower but this tests the logic and memory)

        let iterations = 1000;

        let start = std::time::Instant::now();

        let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, None);

        let duration = start.elapsed();

    

        assert_eq!(runs.len(), iterations);

        println!("Stress test: {} iterations took {:?}", iterations, duration);

    }

    

    #[test]

    fn test_extreme_load_simulation() {

        // 6 Players vs 20 Monsters (Extreme combat)

        let mut players = Vec::new();

        for i in 1..=6 {

            players.push(Creature {

                id: format!("p{}", i),

                name: format!("Player {}", i),

                hp: 150,

                ac: 20,

                count: 1.0,

                mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],

                arrival: None,

                speed_fly: None,

                save_bonus: 5.0,

                str_save_bonus: None,

                dex_save_bonus: None,

                con_save_bonus: None,

                int_save_bonus: None,

                wis_save_bonus: None,

                cha_save_bonus: None,

                con_save_advantage: None,

                save_advantage: None,

                initiative_bonus: DiceFormula::Value(5.0),

                initiative_advantage: true,

                actions: vec![

                    simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {

                        id: "atk".to_string(),

                        name: "Greatsword".to_string(),

                        action_slot: None,

                        cost: vec![],

                        requirements: vec![],

                        tags: vec![],

                        freq: simulation_wasm::model::Frequency::Static("at will".to_string()),

                        condition: simulation_wasm::model::ActionCondition::Default,

                        targets: 2, // Multi-target attack

                        dpr: DiceFormula::Expr("2d6+15".to_string()),

                        target: simulation_wasm::enums::EnemyTarget::EnemyWithLeastHP,

                        to_hit: DiceFormula::Value(10.0),

                        use_saves: None,

                        half_on_save: None,

                        rider_effect: None,

                    })

                ],

                triggers: vec![],

                spell_slots: None,

                class_resources: None,

                hit_dice: None,

                con_modifier: None,

            });

        }

    

        let mut monsters = Vec::new();

        for i in 1..=20 {

            monsters.push(Creature {

                id: format!("m{}", i),

                name: format!("Minion {}", i),

                hp: 50,

                ac: 15,

                count: 1.0,

                mode: "monster".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],

                arrival: None,

                speed_fly: None,

                save_bonus: 2.0,

                str_save_bonus: None,

                dex_save_bonus: None,

                con_save_bonus: None,

                int_save_bonus: None,

                wis_save_bonus: None,

                cha_save_bonus: None,

                con_save_advantage: None,

                save_advantage: None,

                initiative_bonus: DiceFormula::Value(2.0),

                initiative_advantage: false,

                actions: vec![

                    simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {

                        id: "atk".to_string(),

                        name: "Claw".to_string(),

                        action_slot: None,

                        cost: vec![],

                        requirements: vec![],

                        tags: vec![],

                        freq: simulation_wasm::model::Frequency::Static("at will".to_string()),

                        condition: simulation_wasm::model::ActionCondition::Default,

                        targets: 1,

                        dpr: DiceFormula::Expr("2d4+2".to_string()),

                        target: simulation_wasm::enums::EnemyTarget::EnemyWithLeastHP,

                        to_hit: DiceFormula::Value(5.0),

                        use_saves: None,

                        half_on_save: None,

                        rider_effect: None,

                    })

                ],

                triggers: vec![],

                spell_slots: None,

                class_resources: None,

                hit_dice: None,

                con_modifier: None,

            });

        }

    

        let timeline = vec![TimelineStep::Combat(Encounter {

            monsters,

            target_role: TargetRole::Standard,

            players_surprised: None,

            monsters_surprised: None,

            players_precast: None,

            monsters_precast: None,

        })];

    

        let iterations = 2000;

        let start = std::time::Instant::now();

        let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, None);

        let duration = start.elapsed();

    

        assert_eq!(runs.len(), iterations);

        println!("Extreme stress test: {} iterations, 26 combatants took {:?}", iterations, duration);

        

        // Validate performance: should be under 300 seconds for 2k complex runs natively

        assert!(duration.as_secs() < 300, "Simulation too slow: {:?}", duration);

    }

    

    #[test]

    fn test_multi_encounter_stress() {

        // 4 Players vs 3 consecutive encounters

        let mut players = Vec::new();

        for i in 1..=4 {

            players.push(Creature {

                id: format!("p{}", i),

                name: format!("Player {}", i),

                hp: 80,

                ac: 17,

                count: 1.0,

                mode: "player".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],

                arrival: None,

                speed_fly: None,

                save_bonus: 2.0,

                str_save_bonus: None,

                dex_save_bonus: None,

                con_save_bonus: None,

                int_save_bonus: None,

                wis_save_bonus: None,

                cha_save_bonus: None,

                con_save_advantage: None,

                save_advantage: None,

                initiative_bonus: DiceFormula::Value(2.0),

                initiative_advantage: false,

                actions: vec![

                    simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {

                        id: "atk".to_string(),

                        name: "Attack".to_string(),

                        action_slot: None,

                        cost: vec![],

                        requirements: vec![],

                        tags: vec![],

                        freq: simulation_wasm::model::Frequency::Static("at will".to_string()),

                        condition: simulation_wasm::model::ActionCondition::Default,

                        targets: 1,

                        dpr: DiceFormula::Expr("1d10+4".to_string()),

                        target: simulation_wasm::enums::EnemyTarget::EnemyWithHighestAC,

                        to_hit: DiceFormula::Value(6.0),

                        use_saves: None,

                        half_on_save: None,

                        rider_effect: None,

                    })

                ],

                triggers: vec![],

                spell_slots: None,

                class_resources: None,

                hit_dice: None,

                con_modifier: None,

            });

        }

    

        let mut timeline = Vec::new();

        for i in 1..=3 {

            let mut monsters = Vec::new();

            for j in 1..=5 {

                monsters.push(Creature {

                    id: format!("e{}_m{}", i, j),

                    name: format!("Enemy {}-{}", i, j),

                    hp: 40,

                    ac: 14,

                    count: 1.0,

                    mode: "monster".to_string(), magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],

                    arrival: None,

                    speed_fly: None,

                    save_bonus: 1.0,

                    str_save_bonus: None,

                    dex_save_bonus: None,

                    con_save_bonus: None,

                    int_save_bonus: None,

                    wis_save_bonus: None,

                    cha_save_bonus: None,

                    con_save_advantage: None,

                    save_advantage: None,

                    initiative_bonus: DiceFormula::Value(1.0),

                    initiative_advantage: false,

                    actions: vec![

                        simulation_wasm::model::Action::Atk(simulation_wasm::model::AtkAction {

                            id: "atk".to_string(),

                            name: "Attack".to_string(),

                            action_slot: None,

                            cost: vec![],

                            requirements: vec![],

                            tags: vec![],

                            freq: simulation_wasm::model::Frequency::Static("at will".to_string()),

                            condition: simulation_wasm::model::ActionCondition::Default,

                            targets: 1,

                            dpr: DiceFormula::Expr("1d8+2".to_string()),

                            target: simulation_wasm::enums::EnemyTarget::EnemyWithLeastHP,

                            to_hit: DiceFormula::Value(4.0),

                            use_saves: None,

                            half_on_save: None,

                            rider_effect: None,

                        })

                    ],

                    triggers: vec![],

                    spell_slots: None,

                    class_resources: None,

                    hit_dice: None,

                    con_modifier: None,

                });

            }

            timeline.push(TimelineStep::Combat(Encounter {

                monsters,

                target_role: TargetRole::Standard,

                players_surprised: None,

                monsters_surprised: None,

                players_precast: None,

                monsters_precast: None,

            }));

            if i < 3 {

                timeline.push(TimelineStep::ShortRest(simulation_wasm::model::ShortRest {

                    id: format!("rest_{}", i),

                }));

            }

        }

    

        let iterations = 500;

        let start = std::time::Instant::now();

        let runs = run_event_driven_simulation_rust(players, timeline, iterations, false, None);

        let duration = start.elapsed();

    

        assert_eq!(runs.len(), iterations);

        println!("Multi-encounter stress test: {} iterations took {:?}", iterations, duration);

    }


// ============================================================================
// NEW DUAL-TEST APPROACH: Throughput + Latency
// ============================================================================

/// Throughput test: Run for fixed time, count completions
/// Measures: simulations/second (sustained performance)
#[test]
fn stress_throughput_time_boxed() {
    let (time_secs, _) = get_stress_config();

    // Simple test scenario: 1 player vs 5 monsters
    let player = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 15,
        count: 1.0,
        mode: "player".to_string(),
        magic_items: vec![],
        max_arcane_ward_hp: None,
        initial_buffs: vec![],
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
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    let mut monsters = Vec::new();
    for i in 0..5 {
        monsters.push(Creature {
            id: format!("m{}", i),
            name: format!("Monster {}", i),
            hp: 50,
            ac: 12,
            count: 1.0,
            mode: "monster".to_string(),
            magic_items: vec![],
            max_arcane_ward_hp: None,
            initial_buffs: vec![],
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        });
    }

    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters,
        target_role: TargetRole::Standard,
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
    })];

    let duration_target = Duration::from_secs(time_secs);
    let start = Instant::now();
    let mut count = 0;

    while start.elapsed() < duration_target {
        run_event_driven_simulation_rust(vec![player.clone()], timeline.clone(), 31, false, None);
        count += 1;
    }

    let elapsed = start.elapsed();
    let sims_per_sec = (count * 31) as f64 / elapsed.as_secs_f64();

    println!("\n=== STRESS: THROUGHPUT ({}s time-box) ===", time_secs);
    println!("Duration: {:?}", elapsed);
    println!("Iterations: {}", count);
    println!("Total simulations: {}", count * 31);
    println!("Rate: {:.2} sims/second", sims_per_sec);
    println!("===========================================\n");

    // Floor assertion: at least 1 simulation should complete (catches infinite loops)
    assert!(count > 0, "No simulations completed in {}s - possible infinite loop!", time_secs);
}

/// Latency test: Run fixed count, measure time
/// Measures: milliseconds/simulation (per-unit performance)
#[test]
fn stress_latency_count_boxed() {
    let (_, target_sims) = get_stress_config();

    // Same scenario as throughput test for consistency
    let player = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 15,
        count: 1.0,
        mode: "player".to_string(),
        magic_items: vec![],
        max_arcane_ward_hp: None,
        initial_buffs: vec![],
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
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    let mut monsters = Vec::new();
    for i in 0..5 {
        monsters.push(Creature {
            id: format!("m{}", i),
            name: format!("Monster {}", i),
            hp: 50,
            ac: 12,
            count: 1.0,
            mode: "monster".to_string(),
            magic_items: vec![],
            max_arcane_ward_hp: None,
            initial_buffs: vec![],
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
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        });
    }

    let timeline = vec![TimelineStep::Combat(Encounter {
        monsters,
        target_role: TargetRole::Standard,
        players_surprised: None,
        monsters_surprised: None,
        players_precast: None,
        monsters_precast: None,
    })];

    let iterations_per_sim = 31;
    let start = Instant::now();

    for _ in 0..target_sims {
        run_event_driven_simulation_rust(vec![player.clone()], timeline.clone(), iterations_per_sim, false, None);
    }

    let elapsed = start.elapsed();
    let ms_per_sim = elapsed.as_millis() as f64 / target_sims as f64;

    println!("\n=== STRESS: LATENCY ({} sims count-box) ===", target_sims);
    println!("Simulations: {}", target_sims);
    println!("Duration: {:?}", elapsed);
    println!("Average: {:.2} ms/simulation", ms_per_sim);
    println!("Rate: {:.2} sims/second", 1000.0 / ms_per_sim);
    println!("=============================================\n");

    // Floor assertion: should complete in reasonable time (catches catastrophic slowdown)
    // Debug mode is slow: 100 sims ~110s, 1000 sims ~1100s
    // Use very generous floor to catch only infinite loops or catastrophic failures
    let max_acceptable = Duration::from_secs((target_sims as u64 * 2).max(120));
    assert!(elapsed < max_acceptable,
        "{} sims took {:?} - exceeds max acceptable {:?} (possible infinite loop?)",
        target_sims, elapsed, max_acceptable);
}
