use simulation_wasm::model::{Buff, Creature, DiceFormula};
use simulation_wasm::simulation::run_encounter;
use simulation_wasm::context::TurnContext;
use std::collections::HashMap;

#[test]
fn test_arcane_ward_absorbs_all_damage() {
    // Setup creature with max_arcane_ward_hp = 10
    let mut player = Creature {
        name: "Abjurer".to_string(),
        hp: 50,
        ac: 15,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: Some(10),
    };

    let monster = Creature {
        name: "Goblin".to_string(),
        hp: 20,
        ac: 12,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Run encounter with small damage (5 damage)
    let results = run_encounter(&vec![player], &vec![monster], 1);

    // Verify: Ward should be at 5, player HP should be at 50 (unchanged)
    let final_state = &results[0].player_states[0].final_state;
    assert_eq!(final_state.arcane_ward_hp, Some(5), "Ward should absorb 5 damage, leaving 5");
    assert_eq!(final_state.current_hp, 50, "Player HP should be unchanged");
}

#[test]
fn test_arcane_ward_partially_absorbs() {
    // Setup creature with max_arcane_ward_hp = 10
    let player = Creature {
        name: "Abjurer".to_string(),
        hp: 50,
        ac: 15,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: Some(10),
    };

    let monster = Creature {
        name: "Orc".to_string(),
        hp: 30,
        ac: 13,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Run encounter with large damage (15 damage)
    let results = run_encounter(&vec![player], &vec![monster], 1);

    // Verify: Ward depleted (None), player HP reduced by 5
    let final_state = &results[0].player_states[0].final_state;
    assert_eq!(final_state.arcane_ward_hp, None, "Ward should be depleted");
    assert_eq!(final_state.current_hp, 45, "Player HP should be reduced by 5 (overflow damage)");
}

#[test]
fn test_arcane_ward_depleted_single_hit() {
    // Setup creature with max_arcane_ward_hp = 10
    let player = Creature {
        name: "Abjurer".to_string(),
        hp: 50,
        ac: 15,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: Some(10),
    };

    let monster = Creature {
        name: "Troll".to_string(),
        hp: 50,
        ac: 14,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Run encounter with damage > ward (20 damage)
    let results = run_encounter(&vec![player], &vec![monster], 1);

    // Verify: Ward depleted, remaining 10 damage goes to HP
    let final_state = &results[0].player_states[0].final_state;
    assert_eq!(final_state.arcane_ward_hp, None, "Ward should be completely depleted");
    assert_eq!(final_state.current_hp, 40, "Player HP should be reduced by overflow (20 - 10 = 10)");
}

#[test]
fn test_no_ward_direct_damage() {
    // Setup creature WITHOUT max_arcane_ward_hp
    let player = Creature {
        name: "Fighter".to_string(),
        hp: 50,
        ac: 16,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: None, // No ward
    };

    let monster = Creature {
        name: "Goblin".to_string(),
        hp: 20,
        ac: 12,
        actions: vec![],
        initial_buffs: vec![],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Run encounter with 10 damage
    let results = run_encounter(&vec![player], &vec![monster], 1);

    // Verify: All damage goes directly to HP
    let final_state = &results[0].player_states[0].final_state;
    assert_eq!(final_state.arcane_ward_hp, None, "No ward should exist");
    assert!(final_state.current_hp < 50, "Player HP should be reduced by full damage");
}
