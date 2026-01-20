#[cfg(test)]
mod tests {
    use crate::creature_adjustment::detect_role;
    use crate::model::*;

    fn create_test_creature(name: &str, hp: u32, ac: u32, count: f64) -> Creature {
        Creature {
            id: name.to_string(),
            name: name.to_string(),
            hp,
            ac,
            count,
            arrival: None,
            mode: "monster".to_string(),
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
            initial_buffs: vec![],
            magic_items: vec![],
            max_arcane_ward_hp: None,
        }
    }

    #[test]
    fn test_role_detection_minion() {
        let minion = create_test_creature("Goblin", 7, 13, 4.0);
        // HP (7) < 20% of Party DPR (50) = 10. And count >= 4.
        let role = detect_role(&minion, 100.0, 50.0);
        assert_eq!(role, MonsterRole::Minion);
    }

    #[test]
    fn test_role_detection_boss() {
        let boss = create_test_creature("Dragon", 200, 18, 1.0);
        // HP (200) > 50% of Encounter Total HP (300).
        let role = detect_role(&boss, 300.0, 50.0);
        assert_eq!(role, MonsterRole::Boss);
    }

    #[test]
    fn test_role_detection_brute() {
        let brute = create_test_creature("Ogre", 60, 11, 1.0);
        // Melee only (no actions), low AC (11).
        let role = detect_role(&brute, 200.0, 50.0);
        assert_eq!(role, MonsterRole::Brute);
    }

    #[test]
    fn test_adjust_template_action() {
        use crate::creature_adjustment::adjust_damage;
        let mut dragon = create_test_creature("Black Dragon", 200, 19, 1.0);
        dragon.actions.push(Action::Template(TemplateAction {
            id: "breath".to_string(),
            name: "Acid Breath".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            template_options: TemplateOptions {
                template_name: "Line".to_string(),
                target: None,
                save_dc: Some(18.0),
                amount: Some(DiceFormula::Value(54.0)),
            },
        }));

        adjust_damage(&mut dragon, 0.10); // +10%

        let action = &dragon.actions[0];
        if let Action::Template(t) = action {
            // Currently this will FAIL because adjust_damage ignores Template
            assert!(t.template_options.amount.is_some());
            if let Some(DiceFormula::Value(v)) = t.template_options.amount {
                assert!(
                    v > 54.0,
                    "Expected damage {} to be greater than 54.0 after adjustment",
                    v
                );
            }
        }
    }

    #[test]
    fn test_adjust_template_dc() {
        use crate::creature_adjustment::adjust_dc;
        let mut dragon = create_test_creature("Black Dragon", 200, 19, 1.0);
        dragon.actions.push(Action::Template(TemplateAction {
            id: "breath".to_string(),
            name: "Acid Breath".to_string(),
            action_slot: None,
            cost: vec![],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 1,
            template_options: TemplateOptions {
                template_name: "Line".to_string(),
                target: None,
                save_dc: Some(18.0),
                amount: Some(DiceFormula::Value(54.0)),
            },
        }));

        adjust_dc(&mut dragon, -2.0); // Nerf DC by 2

        let action = &dragon.actions[0];
        if let Action::Template(t) = action {
            assert_eq!(t.template_options.save_dc, Some(16.0));
        }
    }
}
