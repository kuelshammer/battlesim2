#[cfg(test)]
mod tests {
    use crate::simulation::run_single_simulation;
    use crate::model::*;
    use crate::enums::*;

    #[test]
    fn test_bless_buff_display_and_bonuses() {
        println!("Testing: Bless buff should display name and give to_hit bonuses");

        // Create a Paladin with properly configured Bless
        let bless_action = Action::Buff(BuffAction {
            id: "bless".to_string(),
            name: "Bless".to_string(),
            action_slot: 1, // Bonus Action
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            buff: Buff {
                display_name: Some("Bless".to_string()), // This should fix "Buff Unknown"
                duration: BuffDuration::Concentration(60), // 1 minute concentration
                ac: None,
                to_hit: Some(DiceFormula::Dice(1, 4, 0)), // +1d4 to attack rolls
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None, // Will be set to Paladin ID
                concentration: true,
            },
            target: AllyTarget::AllyWithLeastHP,
        });

        let paladin = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "paladin_template".to_string(),
            name: "Paladin".to_string(),
            hp: 50.0,
            ac: 18.0,
            initiative_bonus: 2.0,
            initiative_advantage: false,
            save_bonus: 3.0,
            con_save_bonus: Some(3.0),
            count: 1.0,
            speed_fly: None,
            arrival: None,
            actions: vec![bless_action],
            triggers: vec![],
        };

        // Create a target that needs Bless
        let fighter = Creature {
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
            id: "fighter_target".to_string(),
            name: "Fighter".to_string(),
            hp: 40.0,
            ac: 16.0,
            initiative_bonus: 0.0,
            initiative_advantage: false,
            save_bonus: 2.0,
            con_save_bonus: None,
            count: 1.0,
            speed_fly: None,
            arrival: None,
            actions: vec![Action::Atk(AtkAction {
                id: "attack".to_string(),
                name: "Greatsword".to_string(),
                action_slot: 0,
                freq: Frequency::Static("at will".to_string()),
                condition: ActionCondition::Default,
                targets: 1,
                dpr: DiceFormula::Dice(2, 6, 0), // 2d6+5
                to_hit: DiceFormula::Dice(1, 20, 5), // +5 to hit
                target: EnemyTarget::EnemyWithLeastHP,
                use_saves: None,
                half_on_save: None,
                rider_effect: None,
            })],
            triggers: vec![],
        };

        let encounter = Encounter {
            monsters: vec![],
            short_rest: Some(false),
            players_surprised: None,
            monsters_surprised: None,
        };

        let players = vec![paladin, fighter];
        let encounters = vec![encounter];

        // Run simulation with logging enabled
        let (_result, log) = run_single_simulation(&players, &encounters, true);

        // Check log for proper buff name and bonuses
        let log_text = log.join("\n");
        println!("Generated Log:\n{}", log_text);

        // Verify buff name is displayed correctly
        assert!(log_text.contains("Bless"), "Log should contain 'Bless' not 'Unknown'");

        // Verify attack roll gets bonus from buff
        assert!(log_text.contains("+1d4 (buffs)"), "Log should show +1d4 bonus from Bless");

        println!("✓ Bless buff display and bonuses work correctly!");
    }
}