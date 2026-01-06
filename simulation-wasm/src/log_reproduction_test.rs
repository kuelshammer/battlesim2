#[cfg(test)]
mod tests {
    use crate::events::Event;
    use serde_json::to_string;

    #[test]
    fn test_attack_event_contains_roll_details() {
        use crate::events::{RollResult, DieRoll};

        let event = Event::AttackHit {
            attacker_id: "a".to_string(),
            target_id: "b".to_string(),
            damage: 10.0,
            attack_roll: Some(RollResult {
                total: 15.0,
                rolls: vec![DieRoll { sides: 20, value: 10 }],
                modifiers: vec![("Str".to_string(), 5.0)],
                formula: "1d20+5".to_string(),
            }),
            damage_roll: None,
            target_ac: 14.0,
            range: None,
        };

        let json = to_string(&event).unwrap();
        println!("Current JSON: {}", json);

        assert!(json.contains("attack_roll"), "Attack event SHOULD contain attack_roll now");
        assert!(json.contains("modifiers"), "Attack event SHOULD contain modifiers now");
    }
}
