use simulation_wasm::model::{Buff, Creature, DiceFormula};
use simulation_wasm::context::TurnContext;

#[test]
fn test_magic_item_buff_applied() {
    // Create a wizard with Cloak of Protection (+1 AC, +1 saves)
    let wizard = Creature {
        name: "Wizard".to_string(),
        hp: 30,
        ac: 12, // Base AC
        arrival: 0,
        cha_save_bonus: None,
        class_resources: None,
        spell_slots: None,
        hit_dice: None,
        con_modifier: None,
        actions: vec![],
        initial_buffs: vec![
            // This buff comes from magic item flattening
            Buff {
                display_name: Some("Cloak of Protection".to_string()),
                duration: simulation_wasm::enums::BuffDuration::EntireEncounter,
                ac: Some(DiceFormula::Value(1.0)),
                to_hit: Some(DiceFormula::Value(1.0)),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(DiceFormula::Value(1.0)),
                condition: None,
                magnitude: None,
                source: Some("Cloak of Protection".to_string()),
                concentration: false,
                triggers: vec![],
                suppressed_until: None,
            }
        ],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Create a context and apply the creature state
    let mut ctx = TurnContext::new(10);
    let creature_id = "wizard-1".to_string();

    // Initialize creature state (this applies initial_buffs)
    let state = ctx.create_creature_state(&creature_id, &wizard);

    // Verify the buff was applied
    assert!(
        state.buffs.contains_key("Cloak of Protection"),
        "Wizard should have Cloak of Protection buff from magicItems"
    );

    // Verify the buff has correct properties
    let cloak_buff = state.buffs.get("Cloak of Protection").unwrap();
    assert_eq!(
        cloak_buff.ac,
        Some(DiceFormula::Value(1.0)),
        "Cloak should grant +1 AC"
    );
    assert_eq!(
        cloak_buff.save,
        Some(DiceFormula::Value(1.0)),
        "Cloak should grant +1 saves"
    );
}

#[test]
fn test_multiple_magic_items_stack() {
    // Test stacking multiple magic items
    let character = Creature {
        name: "Paladin".to_string(),
        hp: 45,
        ac: 16, // Base AC
        arrival: 0,
        cha_save_bonus: None,
        class_resources: None,
        spell_slots: None,
        hit_dice: None,
        con_modifier: None,
        actions: vec![],
        initial_buffs: vec![
            // Cloak of Protection: +1 AC, +1 saves
            Buff {
                display_name: Some("Cloak of Protection".to_string()),
                duration: simulation_wasm::enums::BuffDuration::EntireEncounter,
                ac: Some(DiceFormula::Value(1.0)),
                to_hit: Some(DiceFormula::Value(1.0)),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(DiceFormula::Value(1.0)),
                condition: None,
                magnitude: None,
                source: Some("Cloak of Protection".to_string()),
                concentration: false,
                triggers: vec![],
                suppressed_until: None,
            },
            // Ring of Protection: +1 AC, +1 saves
            Buff {
                display_name: Some("Ring of Protection".to_string()),
                duration: simulation_wasm::enums::BuffDuration::EntireEncounter,
                ac: Some(DiceFormula::Value(1.0)),
                to_hit: Some(DiceFormula::Value(1.0)),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(DiceFormula::Value(1.0)),
                condition: None,
                magnitude: None,
                source: Some("Ring of Protection".to_string()),
                concentration: false,
                triggers: vec![],
                suppressed_until: None,
            },
        ],
        magic_items: vec![],
        max_arcane_ward_hp: None,
    };

    // Create a context and apply the creature state
    let mut ctx = TurnContext::new(10);
    let creature_id = "paladin-1".to_string();

    let state = ctx.create_creature_state(&creature_id, &character);

    // Verify both buffs are present
    assert_eq!(
        state.buffs.len(),
        2,
        "Should have both Cloak and Ring buffs"
    );

    // Calculate total AC bonus from buffs
    let total_ac_bonus: f64 = state.buffs.values()
        .filter_map(|b| b.ac.as_ref().map(|f| match f {
            DiceFormula::Value(v) => *v,
            _ => 0.0,
        }))
        .sum();

    assert_eq!(total_ac_bonus, 2.0, "Should have +2 AC from both items");
}

