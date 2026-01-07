use simulation_wasm::model::*;
use simulation_wasm::enums::*;
use simulation_wasm::execution::engine::ActionExecutionEngine;
use simulation_wasm::events::Event;
use simulation_wasm::resources::{ActionTag, ActionCost, ResourceType};
use simulation_wasm::rng;
use std::sync::Arc;

#[test]
fn test_counterspell_interrupt_flow() {
    // Wizard A: Fireball caster
    let wizard_a = Creature {
        id: "wizard_a".to_string(),
        name: "Wizard A".to_string(),
        hp: 100,
        ac: 10,
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
        actions: vec![
            Action::Atk(AtkAction {
                id: "fireball".to_string(),
                name: "Fireball".to_string(),
                action_slot: Some(0),
                cost: vec![],
                requirements: vec![],
                tags: vec![ActionTag::Spell, ActionTag::AoE],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(28.0),
                to_hit: DiceFormula::Value(0.0),
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: Some(true),
                half_on_save: Some(true),
                rider_effect: None,
            }),
        ],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    // Wizard B: Counterspeller
    let wizard_b = Creature {
        id: "wizard_b".to_string(),
        name: "Wizard B".to_string(),
        hp: 100,
        ac: 10,
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
        triggers: vec![
            ActionTrigger {
                id: "counterspell_trigger".to_string(),
                condition: TriggerCondition::OnCastSpell,
                action: Action::Template(TemplateAction {
                    id: "counterspell".to_string(),
                    name: "Counterspell".to_string(),
                    action_slot: Some(4), // Reaction
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![ActionTag::Spell],
                    freq: Frequency::Static("at will".to_string()),
                    condition: ActionCondition::Default,
                    targets: 1,
                    template_options: TemplateOptions {
                        template_name: "Counterspell".to_string(),
                        target: None,
                        save_dc: None,
                        amount: None,
                    },
                }),
                cost: Some(4),
            },
        ],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    };

    // In a real implementation, Counterspell template would use TriggerEffect::InterruptAction.
    // Since we don't have a specific Counterspell template logic yet that does this, 
    // let's define a custom action that uses InterruptAction.
    
    // So: Wizard B has a Buff "Counterspell Stance" which has a trigger:
    // OnCastSpell -> InterruptAction.
    
    let counterspeller_with_buff = Creature {
        initial_buffs: vec![
            Buff {
                display_name: Some("Counterspell Stance".to_string()),
                duration: BuffDuration::EntireEncounter,
                ac: None, to_hit: None, damage: None, damage_reduction: None,
                damage_multiplier: None, damage_taken_multiplier: None,
                dc: None, save: None, condition: None, magnitude: None,
                source: None, concentration: false, suppressed_until: None,
                triggers: vec![
                    EffectTrigger {
                        condition: TriggerCondition::OnCastSpell,
                        requirements: vec![],
                        effect: TriggerEffect::InterruptAction {
                            action_id: "any".to_string(),
                        },
                    }
                ],
            }
        ],
        ..wizard_b.clone()
    };

    let mut counterspeller_state = CreatureState { current_hp: 100, ..CreatureState::default() };
    counterspeller_state.buffs.insert(
        "Counterspell Stance".to_string(),
        Buff {
            display_name: Some("Counterspell Stance".to_string()),
            duration: BuffDuration::EntireEncounter,
            ac: None, to_hit: None, damage: None, damage_reduction: None,
            damage_multiplier: None, damage_taken_multiplier: None,
            dc: None, save: None, condition: None, magnitude: None,
            source: None, concentration: false, suppressed_until: None,
            triggers: vec![
                EffectTrigger {
                    condition: TriggerCondition::OnCastSpell,
                    requirements: vec![],
                    effect: TriggerEffect::InterruptAction {
                        action_id: "any".to_string(),
                    },
                }
            ],
        }
    );

    let mut engine = ActionExecutionEngine::new(
        vec![
            Combattant { team: 0,
                id: "wizard_a".to_string(),
                creature: Arc::new(wizard_a),
                initiative: 20.0,
                initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                actions: vec![],
            },
            Combattant { team: 1,
                id: "counterspeller".to_string(),
                creature: Arc::new(counterspeller_with_buff),
                initiative: 10.0,
                initial_state: counterspeller_state.clone(),
                final_state: counterspeller_state,
                actions: vec![],
            },
        ],
        true,
    );

    // Wizard A casts Fireball
    let fireball = engine.context.combatants.get("wizard_a").unwrap().base_combatant.creature.actions[0].clone();
    let result = engine.execute_action_with_reactions("wizard_a", fireball, None);
    engine.context.process_events();

    // Verify results
    assert!(result.success);
    assert!(result.events_generated.iter().any(|e| matches!(e, Event::CastSpell { .. })), "CastSpell event should be emitted");
    
    // Check if any DamageTaken events happened (they shouldn't if interrupted)
    let damage_events: Vec<_> = result.events_generated.iter()
        .filter(|e| matches!(e, Event::DamageTaken { .. }))
        .collect();
    
    assert!(damage_events.is_empty(), "Fireball should have been interrupted and dealt no damage");
}

#[test]
fn test_opportunity_attack_on_movement() {
    // Fighter moves using a movement action (resource consumption)
    // Enemy has a trigger on OnEnemyMoved -> Opportunity Attack (GrantImmediateAction)
    
    let fighter = Creature {
        id: "fighter".to_string(),
        name: "Fighter".to_string(),
        hp: 100, ac: 18, count: 1.0, mode: "player".to_string(),
        magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
        arrival: None, speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![
            Action::Atk(AtkAction {
                id: "move".to_string(),
                name: "Move".to_string(),
                action_slot: Some(0),
                cost: vec![
                    ActionCost::Discrete {
                        resource_type: ResourceType::Movement,
                        resource_val: None,
                        amount: 30.0,
                    }
                ],
                requirements: vec![],
                tags: vec![ActionTag::Movement],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(0.0),
                to_hit: DiceFormula::Value(0.0),
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None, half_on_save: None, rider_effect: None,
            }),
        ],
        triggers: vec![],
        spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None,
    };

    let enemy = Creature {
        id: "enemy".to_string(),
        name: "Enemy".to_string(),
        hp: 100, ac: 10, count: 1.0, mode: "monster".to_string(),
        magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
        arrival: None, speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![
            Action::Atk(AtkAction {
                id: "ao_attack".to_string(),
                name: "Opportunity Attack".to_string(),
                action_slot: Some(4), // Reaction
                cost: vec![],
                requirements: vec![],
                tags: vec![ActionTag::Melee],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(10.0),
                to_hit: DiceFormula::Value(100.0), // Always hit
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None, half_on_save: None, rider_effect: None,
            }),
        ],
        triggers: vec![
            ActionTrigger {
                id: "oa_trigger".to_string(),
                condition: TriggerCondition::OnEnemyMoved,
                action: Action::Atk(AtkAction {
                    id: "ao_attack".to_string(),
                    name: "Opportunity Attack".to_string(),
                    action_slot: Some(4),
                    cost: vec![],
                    requirements: vec![],
                    tags: vec![ActionTag::Melee],
                    freq: Frequency::Static("at will".to_string()),
                    condition: ActionCondition::Default,
                    targets: 1,
                    dpr: DiceFormula::Value(10.0),
                    to_hit: DiceFormula::Value(100.0),
                    target: EnemyTarget::EnemyWithLeastHP,
                    use_saves: None, half_on_save: None, rider_effect: None,
                }),
                cost: Some(4),
            }
        ],
        spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None,
    };

    let mut engine = ActionExecutionEngine::new(
        vec![
            Combattant { team: 0,
                id: "fighter".to_string(),
                creature: Arc::new(fighter),
                initiative: 20.0,
                initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                actions: vec![],
            },
            Combattant { team: 1,
                id: "enemy".to_string(),
                creature: Arc::new(enemy),
                initiative: 10.0,
                initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                actions: vec![],
            },
        ],
        true,
    );

    // Fighter moves
    let move_action = engine.context.combatants.get("fighter").unwrap().base_combatant.creature.actions[0].clone();
    let result = engine.execute_action_with_reactions("fighter", move_action, None);
    engine.context.process_events();

    println!("OA test result success: {}", result.success);
    println!("OA test events: {:?}", result.events_generated);
    let all_events = engine.context.event_bus.get_all_events();
    println!("OA all events in bus: {:?}", all_events);

    assert!(result.success);
    
    // Check for UnitMoved event in the event bus
    assert!(all_events.iter().any(|e| matches!(e, Event::UnitMoved { .. })), "UnitMoved event should be emitted in the event bus");
    
    // Check for Opportunity Attack hit
    let oa_hit = result.reactions_triggered.iter()
        .flat_map(|r| &r.events_generated)
        .any(|e| matches!(e, Event::AttackHit { attacker_id, .. } if attacker_id == "enemy"));
    
    assert!(oa_hit, "Opportunity attack should have triggered and hit");
}

#[test]
fn test_bardic_inspiration_modification() {
    // Ally has a buff "Bardic Inspiration" with trigger OnMiss -> AddToRoll
    
    let ally = Creature {
        id: "ally".to_string(),
        name: "Ally".to_string(),
        hp: 100, ac: 10, count: 1.0, mode: "player".to_string(),
        magic_items: vec![], max_arcane_ward_hp: None,
        initial_buffs: vec![
            Buff {
                display_name: Some("Bardic Inspiration".to_string()),
                duration: BuffDuration::EntireEncounter,
                ac: None, to_hit: None, damage: None, damage_reduction: None,
                damage_multiplier: None, damage_taken_multiplier: None,
                dc: None, save: None, condition: None, magnitude: None,
                source: None, concentration: false, suppressed_until: None,
                triggers: vec![
                    EffectTrigger {
                        condition: TriggerCondition::OnMiss,
                        requirements: vec![],
                        effect: TriggerEffect::AddToRoll {
                            amount: "10".to_string(), // Fixed 10 for testing
                            roll_type: "attack".to_string(),
                        },
                    }
                ],
            }
        ],
        arrival: None, speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![
            Action::Atk(AtkAction {
                id: "attack".to_string(),
                name: "Attack".to_string(),
                action_slot: Some(0),
                cost: vec![], requirements: vec![], tags: vec![],
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Value(10.0),
                to_hit: DiceFormula::Value(0.0), // Low bonus
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None, half_on_save: None, rider_effect: None,
            }),
        ],
        triggers: vec![],
        spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None,
    };

    let enemy = Creature {
        id: "enemy".to_string(),
        name: "Enemy".to_string(),
        hp: 100, ac: 15, count: 1.0, mode: "monster".to_string(),
        magic_items: vec![], max_arcane_ward_hp: None, initial_buffs: vec![],
        arrival: None, speed_fly: None, save_bonus: 0.0,
        str_save_bonus: None, dex_save_bonus: None, con_save_bonus: None,
        int_save_bonus: None, wis_save_bonus: None, cha_save_bonus: None,
        con_save_advantage: None, save_advantage: None,
        initiative_bonus: DiceFormula::Value(0.0), initiative_advantage: false,
        actions: vec![], triggers: vec![],
        spell_slots: None, class_resources: None, hit_dice: None, con_modifier: None,
    };

    let mut initial_state = CreatureState { current_hp: 100, ..CreatureState::default() };
    initial_state.buffs.insert(
        "Bardic Inspiration".to_string(),
        Buff {
            display_name: Some("Bardic Inspiration".to_string()),
            duration: BuffDuration::EntireEncounter,
            ac: None, to_hit: None, damage: None, damage_reduction: None,
            damage_multiplier: None, damage_taken_multiplier: None,
            dc: None, save: None, condition: None, magnitude: None,
            source: None, concentration: false, suppressed_until: None,
            triggers: vec![
                EffectTrigger {
                    condition: TriggerCondition::OnMiss,
                    requirements: vec![],
                    effect: TriggerEffect::AddToRoll {
                        amount: "10".to_string(),
                        roll_type: "attack".to_string(),
                    },
                }
            ],
        }
    );

    let mut engine = ActionExecutionEngine::new(
        vec![
            Combattant { team: 0,
                id: "ally".to_string(),
                creature: Arc::new(ally),
                initiative: 20.0,
                initial_state: initial_state.clone(),
                final_state: initial_state,
                actions: vec![],
            },
            Combattant { team: 1,
                id: "enemy".to_string(),
                creature: Arc::new(enemy),
                initiative: 10.0,
                initial_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                final_state: CreatureState { current_hp: 100, ..CreatureState::default() },
                actions: vec![],
            },
        ],
        true,
    );

    // Force a roll that misses baseline but hits with inspiration (roll 8 + 0 bonus < 15 AC, but 8 + 10 > 15)
    rng::force_d20_rolls(vec![8]);
    
    let attack = engine.context.combatants.get("ally").unwrap().base_combatant.creature.actions[0].clone();
    let result = engine.execute_action_with_reactions("ally", attack, None);
    engine.context.process_events();

    println!("Bardic test result success: {}", result.success);
    println!("Bardic test events: {:?}", result.events_generated);
    let all_events = engine.context.event_bus.get_all_events();
    println!("Bardic all events in bus: {:?}", all_events);

    assert!(result.success);
    
    // Check for AttackHit event (should have missed then inspired then hit)
    let hit_event = result.events_generated.iter()
        .find(|e| matches!(e, Event::AttackHit { attacker_id, .. } if attacker_id == "ally"));
    
    assert!(hit_event.is_some(), "Attack should have hit after Bardic Inspiration");
    
    if let Some(Event::AttackHit { attack_roll, .. }) = hit_event {
        assert_eq!(attack_roll.as_ref().unwrap().total, 18.0); // 8 roll + 10 inspiration
    }
}
