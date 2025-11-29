#[cfg(test)]
mod tests {
    use crate::simulation::get_targets;
    use crate::model::*;
    use crate::enums::*;
    use std::collections::{HashMap, HashSet};

    fn create_dummy_combattant(name: &str, id: &str) -> Combattant {
        Combattant {
            id: id.to_string(),
            initiative: 10.0,
            creature: Creature {
                id: id.to_string(),
                name: name.to_string(),
                count: 1.0,
                hp: 10.0,
                ac: 10.0,
                save_bonus: 0.0,
                initiative_bonus: 0.0,
                initiative_advantage: false,
                actions: vec![],
                arrival: None,
                speed_fly: None,
                con_save_bonus: None,
            },
            initial_state: CreatureState {
                current_hp: 10.0,
                temp_hp: None,
                buffs: HashMap::new(),
                remaining_uses: HashMap::new(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: None,
            },
            final_state: CreatureState {
                current_hp: 10.0,
                temp_hp: None,
                buffs: HashMap::new(),
                remaining_uses: HashMap::new(),
                upcoming_buffs: HashMap::new(),
                used_actions: HashSet::new(),
                concentrating_on: None,
            },
            actions: vec![],
        }
    }

    #[test]
    fn test_buff_targets_distinct_allies() {
        let allies = vec![
            create_dummy_combattant("Ally 1", "1"),
            create_dummy_combattant("Ally 2", "2"),
            create_dummy_combattant("Ally 3", "3"),
        ];
        let enemies = vec![];

        let buff_action = Action::Buff(BuffAction {
            id: "buff".to_string(),
            name: "Bless".to_string(),
            action_slot: 0,
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 3,
            target: AllyTarget::AllyWithLeastHP,
            buff: Buff {
                display_name: Some("Bless".to_string()),
                duration: BuffDuration::EntireEncounter,
                ac: None,
                to_hit: None,
                damage: None,
                damage_reduction: None,
                damage_multiplier: None,
                damage_taken_multiplier: None,
                dc: None,
                save: None,
                condition: None,
                magnitude: None,
                source: None,
                concentration: false,
            },
        });

        let attacker = &allies[0];
        let targets = get_targets(attacker, &buff_action, &allies, &enemies);

        assert_eq!(targets.len(), 3);
        let target_indices: HashSet<usize> = targets.iter().map(|(_, idx)| *idx).collect();
        assert_eq!(target_indices.len(), 3, "Should target 3 distinct allies");
    }

    #[test]
    fn test_atk_targets_can_repeat() {
        let allies = vec![create_dummy_combattant("Attacker", "1")];
        let enemies = vec![
            create_dummy_combattant("Enemy 1", "2"),
        ];

        let atk_action = Action::Atk(AtkAction {
            id: "atk".to_string(),
            name: "Multiattack".to_string(),
            action_slot: 0,
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets: 2,
            dpr: DiceFormula::Value(5.0),
            to_hit: DiceFormula::Value(5.0),
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        });

        let attacker = &allies[0];
        let targets = get_targets(attacker, &atk_action, &allies, &enemies);

        assert_eq!(targets.len(), 2);
        let target_indices: Vec<usize> = targets.iter().map(|(_, idx)| *idx).collect();
        assert_eq!(target_indices[0], 0);
        assert_eq!(target_indices[1], 0); // Should target the same enemy twice
    }
}
