use simulation_wasm::rng;
use simulation_wasm::dice;
use simulation_wasm::model::DiceFormula;
use simulation_wasm::action_resolver::ActionResolver;
use simulation_wasm::context::TurnContext;
use simulation_wasm::model::{Creature, AtkAction, Frequency, ActionCondition, Combattant, CreatureState};
use simulation_wasm::enums::EnemyTarget;
use std::sync::Arc;

#[test]
fn test_mock_rng_d20() {
    rng::clear_forced_rolls();
    rng::force_d20_rolls(vec![20, 1, 10]);
    
    assert_eq!(rng::roll_d20(), 20);
    assert_eq!(rng::roll_d20(), 1);
    assert_eq!(rng::roll_d20(), 10);
    
    // Should fall back to random (or seeded) after queue is empty
    let next = rng::roll_d20();
    assert!(next >= 1 && next <= 20);
    
    rng::clear_forced_rolls();
}

#[test]
fn test_dice_multiplication_on_crit() {
    // Expr multiplication (1d6 * 2 = 2d6)
    rng::clear_forced_rolls();
    
    let formula = DiceFormula::Expr("1d6".to_string());
    
    let res = dice::evaluate_detailed(&formula, 2);
    assert_eq!(res.rolls.len(), 2);
    
    // Value does NOT multiply (7.0 stay 7.0) - per 5e rules only dice double
    let formula_val = DiceFormula::Value(7.0);
    let res_val = dice::evaluate(&formula_val, 2);
    assert_eq!(res_val, 7.0);
}

#[test]
fn test_critical_hit_logic() {
    let player_creature = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 10,
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
    
    let monster_creature = Creature {
        id: "m1".to_string(),
        name: "Monster".to_string(),
        hp: 100,
        ac: 30, // High AC to ensure only crits hit
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

    let p1 = Combattant { team: 0,
        id: "p1".to_string(),
        creature: Arc::new(player_creature),
        initiative: 10.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };
    
    let m1 = Combattant { team: 1,
        id: "m1".to_string(),
        creature: Arc::new(monster_creature),
        initiative: 5.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };

    let mut context = TurnContext::new(vec![p1, m1], vec![], None, "Arena".to_string(), true);
    let resolver = ActionResolver::new();
    
    let attack = AtkAction {
        id: "atk".to_string(),
        name: "Power Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Expr("10".to_string()),
        to_hit: DiceFormula::Value(0.0), // No bonus
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    // 1. Force a CRIT (20)
    rng::force_d20_rolls(vec![20]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    
    // Check if it hit despite high AC (30)
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "Crit should hit");
    
    let attack_dice = AtkAction {
        id: "atk_dice".to_string(),
        name: "Dice Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Expr("1d1+10".to_string()),
        to_hit: DiceFormula::Value(0.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    rng::force_d20_rolls(vec![20]);
    let events = resolver.resolve_attack(&attack_dice, &mut context, "p1");
    
    // 2d1 + 10 = 12
    let dmg_event = events.iter().find(|e| matches!(e, simulation_wasm::events::Event::DamageTaken { .. }));
    if let Some(simulation_wasm::events::Event::DamageTaken { damage, .. }) = dmg_event {
        assert_eq!(*damage, 12.0, "Crit damage should only double dice (1d1->2d1), not constant (+10)");
    }

    // 2. Force a MISS (1)
    // Reset context HP
    context.combatants.get_mut("m1").unwrap().current_hp = 100;
    rng::force_d20_rolls(vec![1]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    
    // Check if it missed despite high bonus (if we had one) or low AC
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackMissed { .. })), "Natural 1 should miss");
    
    rng::clear_forced_rolls();
}

#[test]
fn test_complex_to_hit_formula() {
    let player_creature = Creature {
        id: "p1".to_string(),
        name: "Cleric".to_string(),
        hp: 100,
        ac: 10,
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
    
    let monster_creature = Creature {
        id: "m1".to_string(),
        name: "Target".to_string(),
        hp: 100,
        ac: 18, // AC 18
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

    let p1 = Combattant { team: 0,
        id: "p1".to_string(),
        creature: Arc::new(player_creature),
        initiative: 10.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };
    
    let m1 = Combattant { team: 1,
        id: "m1".to_string(),
        creature: Arc::new(monster_creature),
        initiative: 5.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };

    let mut context = TurnContext::new(vec![p1, m1], vec![], None, "Arena".to_string(), true);
    let resolver = ActionResolver::new();
    
    // Complex formula: +7 base + 1d4 bless. Total range: 8-11.
    // If d20 is 10, total is 18-21. Should hit AC 18.
    let attack = AtkAction {
        id: "atk".to_string(),
        name: "Blessed Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Expr("+7 + 1d4[Bless]".to_string()),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    // Force d20=10, d4=1. Total = 10 + 7 + 1 = 18. Exact hit.
    rng::force_roll(20, 10);
    rng::force_roll(4, 1);
    
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "Exact hit with formula should succeed");

    // Force d20=10, d4=1 against AC 19. Should miss.
    context.combatants.get_mut("m1").unwrap().base_combatant.creature = Arc::new(Creature {
        ac: 19,
        ..(*context.combatants.get("m1").unwrap().base_combatant.creature).clone()
    });
    
    rng::force_roll(20, 10);
    rng::force_roll(4, 1);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackMissed { .. })), "Formula missing AC should fail");

    rng::clear_forced_rolls();
}

#[test]
fn test_advantage_disadvantage_scenarios() {
    let player_creature = Creature {
        id: "p1".to_string(),
        name: "Player".to_string(),
        hp: 100,
        ac: 10,
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
    
    let monster_creature = Creature {
        id: "m1".to_string(),
        name: "Monster".to_string(),
        hp: 100,
        ac: 15,
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

    let p1 = Combattant { team: 0,
        id: "p1".to_string(),
        creature: Arc::new(player_creature),
        initiative: 10.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };
    
    let m1 = Combattant { team: 1,
        id: "m1".to_string(),
        creature: Arc::new(monster_creature),
        initiative: 5.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };

    let resolver = ActionResolver::new();
    let attack = AtkAction {
        id: "atk".to_string(),
        name: "Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Value(5.0), // +5 bonus
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    // 1. DIS(20, 7) + 5 bonus = 12. vs AC 15. Should MISS.
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "dis".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithDisadvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![20, 7]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackMissed { .. })), "dis(20, 7) + 5 = 12 should miss AC 15");

    // 2. ADV(11, 20) + 5 bonus = CRIT. Should HIT.
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "adv".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithAdvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![11, 20]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "adv(11, 20) should be a crit hit");

    // 3. DIS(17, 1) + 5 bonus = 1. Should MISS (Natural 1).
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "dis".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithDisadvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![17, 1]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackMissed { .. })), "dis(17, 1) should miss (Natural 1)");

    // 4. ADV(12, 1) + 5 bonus = 17. vs AC 15. Should HIT.
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "adv".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithAdvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![12, 1]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "adv(12, 1) + 5 = 17 should hit AC 15");

    rng::clear_forced_rolls();
}

#[test]
fn test_triple_advantage_logic() {
    let player_creature = Creature {
        id: "p1".to_string(),
        name: "Elf".to_string(),
        hp: 100,
        ac: 10,
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
    
    let monster_creature = Creature {
        id: "m1".to_string(),
        name: "Monster".to_string(),
        hp: 100,
        ac: 25,
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

    let p1 = Combattant { team: 0,
        id: "p1".to_string(),
        creature: Arc::new(player_creature),
        initiative: 10.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };
    
    let m1 = Combattant { team: 1,
        id: "m1".to_string(),
        creature: Arc::new(monster_creature),
        initiative: 5.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };

    let resolver = ActionResolver::new();
    let attack = AtkAction {
        id: "atk".to_string(),
        name: "Triple Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Value(0.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    // Force Rolls: 1, 1, 20. Triple advantage should pick 20.
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "triple".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithTripleAdvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![1, 1, 20]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "Triple advantage should pick 20 from [1, 1, 20]");
    
    rng::clear_forced_rolls();
}

#[test]
fn test_advantage_disadvantage_cancellation() {
    let p1_creature = Creature {
        id: "p1".to_string(),
        name: "Attacker".to_string(),
        hp: 100,
        ac: 10,
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
    
    let m1_creature = Creature {
        id: "m1".to_string(),
        name: "Defender".to_string(),
        hp: 100,
        ac: 15,
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

    let p1 = Combattant { team: 0,
        id: "p1".to_string(),
        creature: Arc::new(p1_creature),
        initiative: 10.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };
    
    let m1 = Combattant { team: 1,
        id: "m1".to_string(),
        creature: Arc::new(m1_creature),
        initiative: 5.0,
        initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
        actions: vec![],
    };

    let resolver = ActionResolver::new();
    let attack = AtkAction {
        id: "atk".to_string(),
        name: "Attack".to_string(),
        action_slot: None,
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Value(0.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };

    // SETUP: P1 has Advantage, M1 imposes Disadvantage
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    
    // Attacker has Advantage
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "adv".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithAdvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    
    // Defender grants Disadvantage
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "dis_source".to_string(),
        source_id: "m1".to_string(),
        target_id: "m1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::IsAttackedWithDisadvantage),
        remaining_duration: 10,
        conditions: vec![],
    });

    // Force Rolls: 20, 1. 
    // If they cancel out, it's a normal roll -> picks the FIRST roll (20) -> HIT (Crit).
    rng::force_d20_rolls(vec![20, 1]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackHit { .. })), "Cancellation should result in normal roll, picking first roll (20)");

    // Force Rolls: 1, 20. 
    // If they cancel out, it's a normal roll -> picks the FIRST roll (1) -> MISS.
    let mut context = TurnContext::new(vec![p1.clone(), m1.clone()], vec![], None, "Arena".to_string(), true);
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "adv".to_string(),
        source_id: "p1".to_string(),
        target_id: "p1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::AttacksWithAdvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    context.apply_effect(simulation_wasm::context::ActiveEffect {
        id: "dis_source".to_string(),
        source_id: "m1".to_string(),
        target_id: "m1".to_string(),
        effect_type: simulation_wasm::context::EffectType::Condition(simulation_wasm::enums::CreatureCondition::IsAttackedWithDisadvantage),
        remaining_duration: 10,
        conditions: vec![],
    });
    rng::force_d20_rolls(vec![1, 20]);
    let events = resolver.resolve_attack(&attack, &mut context, "p1");
    assert!(events.iter().any(|e| matches!(e, simulation_wasm::events::Event::AttackMissed { .. })), "Cancellation should result in normal roll, picking first roll (1)");

    rng::clear_forced_rolls();
}
