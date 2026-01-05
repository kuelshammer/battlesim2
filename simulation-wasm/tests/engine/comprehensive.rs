use simulation_wasm::model::{Creature, Combattant, CreatureState, Action, AtkAction, ActionTrigger, Frequency, DiceFormula, TemplateAction, TemplateOptions};
use simulation_wasm::execution::{ActionExecutionEngine};
use simulation_wasm::enums::{TriggerCondition, ActionCondition, EnemyTarget, AllyTarget};
use simulation_wasm::events::Event;
use std::sync::Arc;

fn create_simple_creature(id: &str, name: &str, hp: u32, ac: u32, team: u32) -> Combattant {
    let creature = Creature {
        id: id.to_string(),
        name: name.to_string(),
        count: 1.0,
        hp,
        ac,
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
        actions: Vec::new(),
        triggers: Vec::new(),
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        magic_items: vec![],
        max_arcane_ward_hp: None,
        arrival: None,
        mode: if team == 0 { "player".to_string() } else { "monster".to_string() },
    };

    let arc_creature = Arc::new(creature);

    Combattant {
        id: id.to_string(),
        team,
        creature: Arc::clone(&arc_creature),
        initiative: 10.0,
        initial_state: CreatureState {
            current_hp: hp,
            ..CreatureState::default()
        },
        final_state: CreatureState {
            current_hp: hp,
            ..CreatureState::default()
        },
        actions: Vec::new(),
    }
}

/// Helper to fully refresh a combattant after updating its creature
fn finalize_combattant(mut c: Combattant) -> Combattant {
    let hp = c.creature.hp;
    c.initial_state = CreatureState {
        current_hp: hp,
        ..CreatureState::default()
    };
    c.final_state = CreatureState {
        current_hp: hp,
        ..CreatureState::default()
    };
    c
}

#[test]
fn test_shield_spell_reaction() {
    // 1. Setup Attacker (Strong Orc)
    let mut attacker = create_simple_creature("orc", "Orc", 30, 13, 1);
    let atk = AtkAction {
        id: "club".to_string(),
        name: "Club".to_string(),
        action_slot: Some(0),
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Value(0.0), // Roll 12 + 0 = 12. Wizard AC is 10. Hit! But 12 < 10 + 5. Shield should trigger.
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None,
        half_on_save: None,
        rider_effect: None,
    };
    attacker.creature = Arc::new({
        let mut c = (*attacker.creature).clone();
        c.actions.push(Action::Atk(atk));
        c
    });
    attacker = finalize_combattant(attacker);

    // 2. Setup Defender (Wizard with Shield)
    let mut defender = create_simple_creature("wizard", "Wizard", 20, 10, 0);
    
    // Shield Trigger: OnBeingAttacked -> +5 AC
    let shield_trigger = ActionTrigger {
        id: "shield_reaction".to_string(),
        condition: TriggerCondition::OnBeingAttacked,
        action: Action::Template(TemplateAction {
            id: "shield".to_string(),
            name: "Shield".to_string(),
            action_slot: Some(3), // Reaction
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            template_options: TemplateOptions {
                template_name: "Shield".to_string(),
                target: Some(simulation_wasm::enums::TargetType::Ally(AllyTarget::Self_)),
                save_dc: None,
                amount: None,
            },
        }),
        cost: Some(4), // Reaction slot
    };

    defender.creature = Arc::new({
        let mut c = (*defender.creature).clone();
        c.triggers.push(shield_trigger);
        c
    });
    defender = finalize_combattant(defender);

    let mut engine = ActionExecutionEngine::new(vec![attacker, defender], true);
    
    // We need deterministic RNG for this test
    simulation_wasm::rng::seed_rng(42); 
    
    let result = engine.execute_encounter();
    
    // Check if Shield was used (search events)
    let shield_used = result.event_history.iter().any(|e| {
        if let Event::ActionStarted { action_id, .. } = e {
            action_id == "shield"
        } else {
            false
        }
    });
    
    assert!(shield_used, "Shield spell should have been triggered");
}

#[test]
fn test_bless_buff_accuracy() {
    let mut attacker = create_simple_creature("p1", "Player", 30, 10, 0);
    
    // Add Bless Template action
    let bless = Action::Template(TemplateAction {
        id: "bless_action".to_string(),
        name: "Bless".to_string(),
        action_slot: Some(0),
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        template_options: TemplateOptions {
            template_name: "Bless".to_string(),
            target: Some(simulation_wasm::enums::TargetType::Ally(AllyTarget::Self_)),
            save_dc: None,
            amount: None,
        },
    });
    
    // Add basic attack
    let atk = AtkAction {
        id: "hit".to_string(),
        name: "Hit".to_string(),
        action_slot: Some(0),
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(10.0),
        to_hit: DiceFormula::Value(0.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None, half_on_save: None, rider_effect: None,
    };
    
    attacker.creature = Arc::new({
        let mut c = (*attacker.creature).clone();
        c.actions.push(bless);
        c.actions.push(Action::Atk(atk));
        c
    });
    attacker = finalize_combattant(attacker);

    let monster = create_simple_creature("m1", "Monster", 100, 15, 1); // 15 AC

    let mut engine = ActionExecutionEngine::new(vec![attacker, monster], true);
    simulation_wasm::rng::seed_rng(42); 
    
    let result = engine.execute_encounter();
    
    // Verify Bless was applied
    let bless_applied = result.event_history.iter().any(|e| {
        matches!(e, Event::BuffApplied { buff_id, .. } if buff_id == "Bless")
    });
    assert!(bless_applied, "Bless should be applied");
    
    // Check if attack events show the Bless bonus
    let attack_with_bless = result.event_history.iter().any(|e| {
        if let Event::AttackHit { attack_roll: Some(roll), .. } = e {
            roll.modifiers.iter().any(|(m, _)| m == "Bless")
        } else if let Event::AttackMissed { attack_roll: Some(roll), .. } = e {
            roll.modifiers.iter().any(|(m, _)| m == "Bless")
        } else {
            false
        }
    });
    assert!(attack_with_bless, "Attack should include Bless modifier");
}

#[test]
fn test_tpk_detection() {
    let player = create_simple_creature("p1", "Player", 5, 10, 0);
    let mut monster = create_simple_creature("m1", "Monster", 50, 20, 1);
    
    // Monster kills player in one hit
    let atk = AtkAction {
        id: "kill".to_string(),
        name: "Kill".to_string(),
        action_slot: Some(0),
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 1,
        dpr: DiceFormula::Value(100.0),
        to_hit: DiceFormula::Value(20.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None, half_on_save: None, rider_effect: None,
    };
    monster.creature = Arc::new({
        let mut c = (*monster.creature).clone();
        c.actions.push(Action::Atk(atk));
        c
    });
    monster = finalize_combattant(monster);

    let mut engine = ActionExecutionEngine::new(vec![player, monster], true);
    let result = engine.execute_encounter();
    
    assert_eq!(result.winner, Some("Monsters".to_string()));
}

#[test]
fn test_draw_max_rounds() {
    let player = create_simple_creature("p1", "Player", 100, 30, 0);
    let monster = create_simple_creature("m1", "Monster", 100, 30, 1);
    
    let mut engine = ActionExecutionEngine::new(vec![player, monster], true);
    let result = engine.execute_encounter();
    
    assert_eq!(result.winner, None);
    assert_eq!(result.total_rounds, 50); 
}

#[test]
fn test_multiattack_switching() {
    let mut attacker = create_simple_creature("orc", "Orc", 50, 10, 1);
    let atk = AtkAction {
        id: "multi".to_string(),
        name: "Multiattack".to_string(),
        action_slot: Some(0),
        cost: vec![],
        requirements: vec![],
        tags: vec![],
        freq: Frequency::Static("at will".to_string()),
        condition: ActionCondition::Default,
        targets: 2,
        dpr: DiceFormula::Value(20.0),
        to_hit: DiceFormula::Value(20.0),
        target: EnemyTarget::EnemyWithLeastHP,
        use_saves: None, half_on_save: None, rider_effect: None,
    };
    attacker.creature = Arc::new({
        let mut c = (*attacker.creature).clone();
        c.actions.push(Action::Atk(atk));
        c
    });
    attacker = finalize_combattant(attacker);

    let p1 = create_simple_creature("p1", "P1", 10, 10, 0);
    let p2 = create_simple_creature("p2", "P2", 10, 10, 0);

    let mut engine = ActionExecutionEngine::new(vec![attacker, p1, p2], true);
    simulation_wasm::rng::seed_rng(42);
    
    let result = engine.execute_encounter();
    
    let p1_state = result.final_combatant_states.iter().find(|s| s.id == "p1").unwrap();
    let p2_state = result.final_combatant_states.iter().find(|s| s.id == "p2").unwrap();
    
    assert_eq!(p1_state.current_hp, 0, "P1 should be killed by first attack");
    assert_eq!(p2_state.current_hp, 0, "P2 should be killed by second attack");
}
