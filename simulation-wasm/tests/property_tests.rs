// Property-Based Tests for Simulation Invariants
//
// These tests use proptest to validate that fundamental simulation rules
// hold across thousands of randomly generated inputs.

use proptest::prelude::*;
use simulation_wasm::model::*;
use simulation_wasm::execution::ActionExecutionEngine;
use simulation_wasm::enums::EnemyTarget;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a minimal creature for testing
fn create_minimal_creature(id: String, hp: u32, ac: u32, damage_dice: &str) -> Creature {
    Creature {
        id: id.clone(),
        name: id,
        hp,
        ac,
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
        actions: vec![
            Action::Atk(AtkAction {
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
            })
        ],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    }
}

/// Create a minimal Combattant for testing
fn create_minimal_combatant(creature: Creature, team: u32, initiative: f64) -> Combattant {
    let hp = creature.hp;
    Combattant {
        id: creature.id.clone(),
        team,
        creature: std::sync::Arc::new(creature),
        initiative,
        initial_state: CreatureState { current_hp: hp, ..CreatureState::default() },
        final_state: CreatureState { current_hp: hp, ..CreatureState::default() },
        actions: vec![],
    }
}

// ============================================================================
// INVARIANT 1: Round Count Consistency
// ============================================================================

proptest! {
    /// Round counts should be consistent and bounded
    #[test]
    fn prop_round_count_bounded(
        player_hp in 50u32..100,
        monster_hp in 50u32..100,
    ) {
        let player = create_minimal_creature("player".to_string(), player_hp, 15, "1d8+3");
        let monster = create_minimal_creature("monster".to_string(), monster_hp, 12, "1d6+2");

        let combatants = vec![
            create_minimal_combatant(player, 0, 10.0),
            create_minimal_combatant(monster, 1, 5.0),
        ];

        let mut engine = ActionExecutionEngine::new(combatants);
        let result = engine.execute_encounter();

        // Assert: total_rounds should be one less than snapshots (snapshots include initial state)
        prop_assert_eq!(
            result.total_rounds + 1,
            result.round_snapshots.len() as u32,
            "total_rounds + 1 should equal round_snapshots.len()"
        );

        // Assert: Should have reasonable number of rounds (not infinite)
        prop_assert!(result.total_rounds <= 100, "Too many rounds");
    }
}

// ============================================================================
// INVARIANT 2: HP Bounds
// ============================================================================

proptest! {
    /// HP should stay within valid bounds (0 to max)
    #[test]
    fn prop_hp_within_bounds(
        player_hp in 50u32..100,
        monster_hp in 50u32..100,
    ) {
        let player = create_minimal_creature("player".to_string(), player_hp, 15, "1d8+3");
        let monster = create_minimal_creature("monster".to_string(), monster_hp, 12, "1d6+2");

        let player_max_hp = player.hp;
        let monster_max_hp = monster.hp;

        let combatants = vec![
            create_minimal_combatant(player, 0, 10.0),
            create_minimal_combatant(monster, 1, 5.0),
        ];

        let mut engine = ActionExecutionEngine::new(combatants);
        let result = engine.execute_encounter();

        // Check all combatants in all rounds
        for round in &result.round_snapshots {
            for combatant in round {
                prop_assert!(
                    combatant.current_hp <= combatant.base_combatant.creature.hp,
                    "HP {} exceeded max {} for {}",
                    combatant.current_hp,
                    combatant.base_combatant.creature.hp,
                    combatant.base_combatant.creature.name
                );
            }
        }

        // Check final states also respect bounds
        for combatant in &result.final_combatant_states {
            let max_hp = if combatant.base_combatant.creature.id == "player" {
                player_max_hp
            } else {
                monster_max_hp
            };
            prop_assert!(
                combatant.current_hp <= max_hp,
                "Final HP {} exceeded max {}",
                combatant.current_hp,
                max_hp
            );
        }
    }
}

// ============================================================================
// INVARIANT 3: Event Sequence Validity
// ============================================================================

proptest! {
    /// Event IDs should reference valid combatants
    #[test]
    fn prop_event_ids_valid(
        player_hp in 50u32..100,
        monster_hp in 50u32..100,
    ) {
        let player = create_minimal_creature("player".to_string(), player_hp, 15, "1d8+3");
        let monster = create_minimal_creature("monster".to_string(), monster_hp, 12, "1d6+2");

        let combatants = vec![
            create_minimal_combatant(player, 0, 10.0),
            create_minimal_combatant(monster, 1, 5.0),
        ];

        let valid_ids: std::collections::HashSet<String> = combatants
            .iter()
            .map(|c| c.id.clone())
            .collect();

        let mut engine = ActionExecutionEngine::new(combatants);
        let result = engine.execute_encounter();

        // Check all event references
        for event in &result.event_history {
            let valid = match event {
                simulation_wasm::events::Event::TurnStarted { unit_id, .. } => {
                    valid_ids.contains(unit_id)
                }
                simulation_wasm::events::Event::ActionStarted { actor_id, .. } => {
                    valid_ids.contains(actor_id)
                }
                simulation_wasm::events::Event::AttackHit { attacker_id, target_id, .. } => {
                    valid_ids.contains(attacker_id) &&
                    valid_ids.contains(target_id)
                }
                simulation_wasm::events::Event::AttackMissed { attacker_id, target_id, .. } => {
                    valid_ids.contains(attacker_id) &&
                    valid_ids.contains(target_id)
                }
                simulation_wasm::events::Event::DamageTaken { target_id, .. } => {
                    valid_ids.contains(target_id)
                }
                simulation_wasm::events::Event::HealingApplied { target_id, .. } => {
                    valid_ids.contains(target_id)
                }
                simulation_wasm::events::Event::UnitDied { unit_id, .. } => {
                    valid_ids.contains(unit_id)
                }
                _ => true,
            };

            prop_assert!(valid, "Invalid ID in event: {:?}", event);
        }
    }
}

// ============================================================================
// INVARIANT 4: Combatant Count Consistency
// ============================================================================

proptest! {
    /// Combatant count should remain constant across rounds
    #[test]
    fn prop_combatant_count_consistent(
        player_hp in 50u32..100,
        monster_hp in 50u32..100,
    ) {
        let player = create_minimal_creature("player".to_string(), player_hp, 15, "1d8+3");
        let monster = create_minimal_creature("monster".to_string(), monster_hp, 12, "1d6+2");

        let expected_count = 2;
        let combatants = vec![
            create_minimal_combatant(player, 0, 10.0),
            create_minimal_combatant(monster, 1, 5.0),
        ];

        let mut engine = ActionExecutionEngine::new(combatants);
        let result = engine.execute_encounter();

        // Check each round has same number of combatants
        for (i, round) in result.round_snapshots.iter().enumerate() {
            prop_assert_eq!(
                round.len(),
                expected_count,
                "Round {} has {} combatants, expected {}",
                i,
                round.len(),
                expected_count
            );
        }
    }
}

// ============================================================================
// INVARIANT 5: Action Economy Bounds
// ============================================================================

proptest! {
    /// Total actions executed should be reasonably bounded
    #[test]
    fn prop_actions_bounded(
        player_hp in 50u32..100,
        monster_hp in 50u32..100,
    ) {
        let player = create_minimal_creature("player".to_string(), player_hp, 15, "1d8+3");
        let monster = create_minimal_creature("monster".to_string(), monster_hp, 12, "1d6+2");

        let combatants = vec![
            create_minimal_combatant(player, 0, 10.0),
            create_minimal_combatant(monster, 1, 5.0),
        ];

        let combatant_count = combatants.len();

        let mut engine = ActionExecutionEngine::new(combatants);
        let result = engine.execute_encounter();

        // Count ActionStarted events
        let action_count = result.event_history.iter()
            .filter(|e| matches!(e, simulation_wasm::events::Event::ActionStarted { .. }))
            .count();

        // Upper bound: 2 actions per combatant per round + margin
        let max_expected = (combatant_count * 2) * (result.total_rounds as usize + 1);

        prop_assert!(
            action_count <= max_expected * 2, // Allow 2x margin for reactions/etc
            "Too many actions: {} (max ~{})",
            action_count,
            max_expected * 2
        );
    }
}
