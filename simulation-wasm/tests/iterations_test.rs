use simulation_wasm::run_event_driven_simulation_rust;
use simulation_wasm::model::{Creature, TimelineStep, TargetRole, Encounter, DiceFormula};
use simulation_wasm::decile_analysis::run_decile_analysis;

#[test]
fn test_fast_simulation_iterations() {
    let player = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 15,
        count: 1.0,
        mode: "player".to_string(),
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
        mode: "monster".to_string(),
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
    let runs = run_event_driven_simulation_rust(players.clone(), timeline.clone(), iterations, false);

    assert_eq!(runs.len(), iterations, "Should have exactly 31 runs");

    // Check decile analysis works with 31 runs
    let results: Vec<_> = runs.iter().map(|r| r.result.clone()).collect();
    let analysis = run_decile_analysis(&results, "Fast Test", 1);

    assert_eq!(analysis.deciles.len(), 10, "Should produce 10 deciles");
    
    // Check that we have events for at least the first run
    assert!(!runs[0].events.is_empty(), "First run should have events");
}