use simulation_wasm::execution;
use simulation_wasm::*; // Add execution module import

fn main() {
    // Simple test: Caster casts Bless on Target, then Caster dies
    // Expected: Bless should be removed from Target in aggregated results

    let caster = model::Creature {
        id: "caster-template".to_string(),
        name: "Acolyte Buff".to_string(),
        count: 1.0,
        hp: 5.0, // Very low HP so it dies quickly
        ac: 10.0,
        speed_fly: None,
        save_bonus: 0.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: Some(2.0),
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: model::DiceFormula::Value(0.0),
        initiative_advantage: false,
        actions: vec![model::Action::Buff(model::BuffAction {
            id: "bless".to_string(),
            name: "Bless".to_string(),
            action_slot: Some(1), // Bonus Action, ensure Some
            cost: vec![],         // Add new fields
            requirements: vec![], // Add new fields
            tags: vec![],         // Add new fields
            freq: model::Frequency::Static("at will".to_string()),
            condition: enums::ActionCondition::Default,
            targets: 3,
            target: enums::AllyTarget::AllyWithMostHP,
            buff: model::Buff {
                display_name: Some("Bless".to_string()),
                duration: enums::BuffDuration::EntireEncounter,
                ac: None,
                to_hit: Some(model::DiceFormula::Expr("1d4".to_string())),
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: Some(model::DiceFormula::Expr("1d4".to_string())),
                condition: None,
                magnitude: None,
                source: None, // Will be set during simulation
                concentration: true,
                triggers: vec![], // Add missing triggers field
            },
        })],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        arrival: None,
        mode: "monster".to_string(),
    };
    let target = model::Creature {
        id: "target-template".to_string(),
        name: "Player Fighter".to_string(),
        count: 1.0,
        hp: 54.0,
        ac: 18.0,
        speed_fly: None,
        save_bonus: 1.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: Some(3.0),
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: model::DiceFormula::Value(0.0),
        initiative_advantage: false,
        actions: vec![],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        arrival: None,
        mode: "monster".to_string(),
    };

    let enemy = model::Creature {
        id: "enemy-template".to_string(),
        name: "Goblin".to_string(),
        count: 1.0,
        hp: 7.0,
        ac: 15.0,
        speed_fly: None,
        save_bonus: 0.0,
        str_save_bonus: None,
        dex_save_bonus: None,
        con_save_bonus: Some(0.0),
        int_save_bonus: None,
        wis_save_bonus: None,
        cha_save_bonus: None,
        con_save_advantage: None,
        save_advantage: None,
        initiative_bonus: model::DiceFormula::Value(2.0),
        initiative_advantage: false,
        actions: vec![model::Action::Atk(model::AtkAction {
            id: "shortsword".to_string(),
            name: "Shortsword".to_string(),
            action_slot: Some(0), // Ensure Some(0)
            cost: vec![],         // Add missing fields
            requirements: vec![], // Add missing fields
            tags: vec![],         // Add missing fields
            freq: model::Frequency::Static("at will".to_string()),
            condition: enums::ActionCondition::Default,
            targets: 1,
            dpr: model::DiceFormula::Expr("1d6+2".to_string()),
            to_hit: model::DiceFormula::Value(4.0),
            target: enums::EnemyTarget::EnemyWithMostHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        })],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
        arrival: None,
        mode: "monster".to_string(),
    };

    let players = vec![caster, target];
    let encounter = model::Encounter {
        monsters: vec![enemy],
        players_surprised: None,
        monsters_surprised: None,
        short_rest: None,
        players_precast: None,
        monsters_precast: None,
    };

    println!("Running 10 simulations...");
    let (results, _events) = crate::run_event_driven_simulation_rust(players, vec![encounter], 10, false);

    println!("\nAggregating results...");
    let aggregated = aggregation::aggregate_results(&results);

    println!("\n=== Aggregated Results ===");
    for (round_idx, round) in aggregated.iter().enumerate() {
        println!("\n--- Round {} ---", round_idx + 1);

        println!("Team 1:");
        for c in &round.team1 {
            println!(
                "  {}: HP {:.1}/{:.1}, Buffs: {:?}, Concentrating: {:?}",
                c.creature.name,
                c.final_state.current_hp,
                c.creature.hp,
                c.final_state.buffs.keys().collect::<Vec<_>>(),
                c.final_state.concentrating_on
            );

            for (buff_id, buff) in &c.final_state.buffs {
                println!("    - Buff {}: source = {:?}", buff_id, buff.source);
            }
        }

        println!("Team 2:");
        for c in &round.team2 {
            println!(
                "  {}: HP {:.1}/{:.1}",
                c.creature.name, c.final_state.current_hp, c.creature.hp
            );
        }
    }
}
