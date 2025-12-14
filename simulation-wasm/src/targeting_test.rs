#[cfg(test)]
mod tests {
    use crate::enums::*;
    use crate::model::*;
    use crate::resources::{ActionCost, ResourceType};
    use crate::targeting::estimate_dpr;
    use std::collections::HashMap;

    fn create_dummy_action(
        name: &str,
        damage: &str,
        targets: i32,
        to_hit: &str,
        cost_type: ResourceType,
    ) -> Action {
        Action::Atk(AtkAction {
            id: name.to_string(),
            name: name.to_string(),
            action_slot: None,
            cost: vec![ActionCost::Discrete {
                resource_type: cost_type,
                resource_val: None,
                amount: 1.0,
            }],
            requirements: vec![],
            tags: vec![],
            freq: Frequency::Static("at will".to_string()),
            condition: ActionCondition::Default,
            targets,
            dpr: DiceFormula::Expr(damage.to_string()),
            to_hit: DiceFormula::Expr(to_hit.to_string()),
            target: EnemyTarget::EnemyWithLeastHP,
            use_saves: None,
            half_on_save: None,
            rider_effect: None,
        })
    }

    fn create_dummy_combattant(name: &str, actions: Vec<Action>) -> Combattant {
        let creature = Creature {
            id: name.to_string(),
            name: name.to_string(),
            count: 1.0,
            hp: 100.0,
            ac: 15.0,
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions,
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
            arrival: None,
            mode: "monster".to_string(),
        };

        Combattant {
            id: name.to_string(),
            creature: creature.clone(),
            initiative: 10.0,
            initial_state: CreatureState::default(),
            final_state: CreatureState::default(),
            actions: vec![],
        }
    }

    #[test]
    fn test_estimate_dpr_multiattack() {
        // Creature A: 1 attack, 10 dmg
        let action_a = create_dummy_action("Big Hit", "10", 1, "5", ResourceType::Action);
        let creature_a = create_dummy_combattant("Creature A", vec![action_a]);

        // Creature B: 3 attacks, 5 dmg each (Total 15)
        let action_b = create_dummy_action("Multi Hit", "5", 3, "5", ResourceType::Action);
        let creature_b = create_dummy_combattant("Creature B", vec![action_b]);

        let dpr_a = estimate_dpr(&creature_a);
        let dpr_b = estimate_dpr(&creature_b);

        println!("DPR A: {}", dpr_a);
        println!("DPR B: {}", dpr_b);

        // Current buggy implementation will likely say A (10) > B (5) because it ignores targets count
        // Correct behavior should be B (15) > A (10)
        assert!(
            dpr_b > dpr_a,
            "Creature B with 3x5 damage should have higher DPR than Creature A with 1x10 damage"
        );
    }

    #[test]
    fn test_estimate_dpr_hit_chance() {
        // Creature A: 10 dmg, +10 to hit (High chance)
        let action_a = create_dummy_action("Accurate", "10", 1, "10", ResourceType::Action);
        let creature_a = create_dummy_combattant("Creature A", vec![action_a]);

        // Creature B: 12 dmg, +0 to hit (Low chance)
        let action_b = create_dummy_action("Inaccurate", "12", 1, "0", ResourceType::Action);
        let creature_b = create_dummy_combattant("Creature B", vec![action_b]);

        let dpr_a = estimate_dpr(&creature_a);
        let dpr_b = estimate_dpr(&creature_b);

        // Against AC 15:
        // A: +10 needs 5+ (80% chance). 10 * 0.8 = 8.0
        // B: +0 needs 15+ (30% chance). 12 * 0.3 = 3.6
        // Current implementation ignores hit chance, so it sees 12 > 10.

        assert!(
            dpr_a > dpr_b,
            "Accurate creature should have higher effective DPR"
        );
    }
}
