#[cfg(test)]
mod tests {
    use crate::model::*;
    use crate::creature_adjustment::detect_role;

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
}