use simulation_wasm::enums::{AttackRange, BuffDuration, TriggerCondition, TriggerEffect};
use simulation_wasm::events::Event;

#[test]
fn test_armor_of_agathys_trigger_conditions() {
    // Verify Armor of Agathys trigger condition structure
    // Should use OnBeingHit AND AttackWasMelee

    let on_being_hit = TriggerCondition::OnBeingHit;
    let attack_was_melee = TriggerCondition::AttackWasMelee;

    // Create composite trigger
    let composite_trigger = TriggerCondition::And {
        conditions: vec![on_being_hit, attack_was_melee],
    };

    // Test that it evaluates correctly for melee AttackHit events
    let melee_hit_event = Event::AttackHit {
        attacker_id: "goblin".to_string(),
        target_id: "warlock".to_string(),
        damage: 5.0,
        attack_roll: None,
        damage_roll: None,
        target_ac: 13.0,
        range: Some(AttackRange::Melee),
    };

    assert!(
        composite_trigger.evaluate(&melee_hit_event),
        "And condition should match melee hit"
    );

    // Test that it doesn't match ranged attacks
    let ranged_hit_event = Event::AttackHit {
        attacker_id: "goblin".to_string(),
        target_id: "warlock".to_string(),
        damage: 5.0,
        attack_roll: None,
        damage_roll: None,
        target_ac: 13.0,
        range: Some(AttackRange::Ranged),
    };

    assert!(
        !composite_trigger.evaluate(&ranged_hit_event),
        "And condition should NOT match ranged hit"
    );

    // Test that it doesn't match attacks without range set
    let unknown_range_hit = Event::AttackHit {
        attacker_id: "goblin".to_string(),
        target_id: "warlock".to_string(),
        damage: 5.0,
        attack_roll: None,
        damage_roll: None,
        target_ac: 13.0,
        range: None,
    };

    assert!(
        !composite_trigger.evaluate(&unknown_range_hit),
        "And condition should NOT match attack with unknown range"
    );
}

#[test]
fn test_cloak_of_displacement_trigger_on_damage() {
    // Verify OnBeingDamaged trigger condition for Cloak of Displacement

    let trigger = TriggerCondition::OnBeingDamaged;

    let damage_event = Event::DamageTaken {
        target_id: "player".to_string(),
        damage: 10.0,
        damage_type: "Physical".to_string(),
    };

    assert!(
        trigger.evaluate(&damage_event),
        "OnBeingDamaged should match DamageTaken event"
    );

    let attack_hit_event = Event::AttackHit {
        attacker_id: "enemy".to_string(),
        target_id: "player".to_string(),
        damage: 10.0,
        attack_roll: None,
        damage_roll: None,
        target_ac: 15.0,
        range: None,
    };

    assert!(
        !trigger.evaluate(&attack_hit_event),
        "OnBeingDamaged should NOT match AttackHit event"
    );
}

#[test]
fn test_suppress_buff_effect_structure() {
    // Verify SuppressBuff effect structure for Cloak of Displacement
    let suppress_effect = TriggerEffect::SuppressBuff {
        buff_id: "Cloak of Displacement".to_string(),
        duration: BuffDuration::OneRound,
    };

    // Verify the effect can be created and matched
    match &suppress_effect {
        TriggerEffect::SuppressBuff { buff_id, duration } => {
            assert_eq!(buff_id, "Cloak of Displacement");
            assert_eq!(duration, &BuffDuration::OneRound);
        }
        _ => panic!("Expected SuppressBuff effect"),
    }
}

#[test]
fn test_trigger_condition_variants() {
    // Test all new trigger condition variants

    // OnCastSpell
    let cast_spell = TriggerCondition::OnCastSpell;
    let spell_event = Event::CastSpell {
        caster_id: "wizard".to_string(),
        spell_name: "Fireball".to_string(),
        target_ids: vec!["enemy1".to_string()],
        spell_level: 3,
    };
    assert!(cast_spell.evaluate(&spell_event));

    // OnSaveFailed
    let save_failed = TriggerCondition::OnSaveFailed;
    let save_fail_event = Event::SaveResult {
        creature_id: "player".to_string(),
        save_type: "DEX".to_string(),
        succeeded: false,
        roll_total: 8,
    };
    assert!(save_failed.evaluate(&save_fail_event));

    // OnSaveSucceeded
    let save_success = TriggerCondition::OnSaveSucceeded;
    let save_success_event = Event::SaveResult {
        creature_id: "player".to_string(),
        save_type: "DEX".to_string(),
        succeeded: true,
        roll_total: 15,
    };
    assert!(save_success.evaluate(&save_success_event));
}

#[test]
fn test_creature_condition_variants() {
    // Test new CreatureCondition variants
    use simulation_wasm::enums::CreatureCondition;

    // Verify variants exist
    let _ = CreatureCondition::IsProne;
    let _ = CreatureCondition::IsHidden;
    let _ = CreatureCondition::IsConcentrating;
    let _ = CreatureCondition::IsSurprised;
}

#[test]
fn test_roll_manipulation_effects() {
    // Test roll manipulation TriggerEffect variants

    let add_to_roll = TriggerEffect::AddToRoll {
        amount: "1d4".to_string(),
        roll_type: "attack".to_string(),
    };
    match &add_to_roll {
        TriggerEffect::AddToRoll { amount, roll_type } => {
            assert_eq!(amount, "1d4");
            assert_eq!(roll_type, "attack");
        }
        _ => panic!("Expected AddToRoll effect"),
    }

    let force_self_reroll = TriggerEffect::ForceSelfReroll {
        roll_type: "save".to_string(),
        must_use_second: true,
    };
    match &force_self_reroll {
        TriggerEffect::ForceSelfReroll {
            roll_type,
            must_use_second,
        } => {
            assert_eq!(roll_type, "save");
            assert_eq!(must_use_second, &true);
        }
        _ => panic!("Expected ForceSelfReroll effect"),
    }

    let force_target_reroll = TriggerEffect::ForceTargetReroll {
        roll_type: "abilityCheck".to_string(),
        must_use_second: false,
    };
    match &force_target_reroll {
        TriggerEffect::ForceTargetReroll {
            roll_type,
            must_use_second,
        } => {
            assert_eq!(roll_type, "abilityCheck");
            assert_eq!(must_use_second, &false);
        }
        _ => panic!("Expected ForceTargetReroll effect"),
    }
}

#[test]
fn test_interrupt_system_effects() {
    // Test interrupt system TriggerEffect variants

    let interrupt = TriggerEffect::InterruptAction {
        action_id: "enemy-attack".to_string(),
    };
    match &interrupt {
        TriggerEffect::InterruptAction { action_id } => {
            assert_eq!(action_id, "enemy-attack");
        }
        _ => panic!("Expected InterruptAction effect"),
    }

    let grant_immediate = TriggerEffect::GrantImmediateAction {
        action_id: "opportunity-attack".to_string(),
        action_slot: "reaction".to_string(),
    };
    match &grant_immediate {
        TriggerEffect::GrantImmediateAction {
            action_id,
            action_slot,
        } => {
            assert_eq!(action_id, "opportunity-attack");
            assert_eq!(action_slot, "reaction");
        }
        _ => panic!("Expected GrantImmediateAction effect"),
    }
}
