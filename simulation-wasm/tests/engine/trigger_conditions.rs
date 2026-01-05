use simulation_wasm::enums::TriggerCondition;
use simulation_wasm::events::Event;

#[cfg(test)]
mod composite_trigger_tests {
    use super::*;

    #[test]
    fn test_and_condition_both_true() {
        // TriggerCondition::And with two OnHit conditions should return true only if both evaluate true
        let and_condition = TriggerCondition::And {
            conditions: vec![
                TriggerCondition::OnHit,
                TriggerCondition::OnHit,
            ],
        };

        let hit_event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
            attack_roll: None,
            damage_roll: None,
            target_ac: 15.0,
        };

        assert!(and_condition.evaluate(&hit_event));
    }

    #[test]
    fn test_and_condition_one_false() {
        // And condition with OnHit and OnMiss should return false when only hit occurs
        let and_condition = TriggerCondition::And {
            conditions: vec![
                TriggerCondition::OnHit,
                TriggerCondition::OnMiss,
            ],
        };

        let hit_event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
            attack_roll: None,
            damage_roll: None,
            target_ac: 15.0,
        };

        assert!(!and_condition.evaluate(&hit_event));
    }

    #[test]
    fn test_or_condition_hit_matches() {
        // TriggerCondition::Or with OnHit and OnMiss should return true if either matches
        let or_condition = TriggerCondition::Or {
            conditions: vec![
                TriggerCondition::OnHit,
                TriggerCondition::OnMiss,
            ],
        };

        let hit_event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
            attack_roll: None,
            damage_roll: None,
            target_ac: 15.0,
        };

        assert!(or_condition.evaluate(&hit_event));
    }

    #[test]
    fn test_or_condition_miss_matches() {
        // Or condition should return true for miss event
        let or_condition = TriggerCondition::Or {
            conditions: vec![
                TriggerCondition::OnHit,
                TriggerCondition::OnMiss,
            ],
        };

        let miss_event = Event::AttackMissed {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            attack_roll: None,
            target_ac: 15.0,
        };

        assert!(or_condition.evaluate(&miss_event));
    }

    #[test]
    fn test_or_condition_unrelated_event() {
        // Or condition should return false when neither condition matches
        let or_condition = TriggerCondition::Or {
            conditions: vec![
                TriggerCondition::OnHit,
                TriggerCondition::OnMiss,
            ],
        };

        let damage_event = Event::DamageTaken {
            target_id: "target".to_string(),
            damage: 10.0,
            damage_type: "fire".to_string(),
        };

        assert!(!or_condition.evaluate(&damage_event));
    }

    #[test]
    fn test_not_condition_with_hit() {
        // TriggerCondition::Not wrapping OnHit should return false when event is a hit
        let not_condition = TriggerCondition::Not {
            condition: Box::new(TriggerCondition::OnHit),
        };

        let hit_event = Event::AttackHit {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            damage: 10.0,
            attack_roll: None,
            damage_roll: None,
            target_ac: 15.0,
        };

        assert!(!not_condition.evaluate(&hit_event));
    }

    #[test]
    fn test_not_condition_with_miss() {
        // Not condition wrapping OnHit should return true when event is NOT a hit
        let not_condition = TriggerCondition::Not {
            condition: Box::new(TriggerCondition::OnHit),
        };

        let miss_event = Event::AttackMissed {
            attacker_id: "attacker".to_string(),
            target_id: "target".to_string(),
            attack_roll: None,
            target_ac: 15.0,
        };

        assert!(not_condition.evaluate(&miss_event));
    }
}
