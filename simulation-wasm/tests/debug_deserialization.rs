#[cfg(test)]
mod debug_tests {
    use simulation_wasm::model::Creature;
    use serde_json;

    // Test with minimal action data similar to what TypeScript sends
    #[test]
    fn test_minimal_action_deserialization() {
        let json = r#"{
            "id": "test-1",
            "mode": "player",
            "name": "Test Fighter",
            "count": 1,
            "hp": 44,
            "ac": 18,
            "saveBonus": 3,
            "initiativeBonus": 2,
            "actions": [{
                "id": "action-1",
                "name": "Longsword",
                "actionSlot": 0,
                "type": "atk",
                "freq": "at will",
                "condition": "default",
                "dpr": "1d8+3",
                "toHit": 5,
                "target": "enemy with least HP",
                "targets": 1,
                "cost": [{"type": "Discrete", "resourceType": "Action", "amount": 1}],
                "requirements": [],
                "tags": ["Melee", "Weapon", "Attack", "Damage"]
            }],
            "triggers": []
        }"#;

        let result: Result<Creature, _> = serde_json::from_str(json);
        match &result {
            Ok(_) => println!("Deserialization succeeded"),
            Err(e) => println!("Deserialization failed: {}", e),
        }
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
    }

    // Test with triggers containing TriggerRequirement
    #[test]
    fn test_trigger_requirement_deserialization() {
        let json = r#"{
            "id": "test-2",
            "mode": "player",
            "name": "Test Wizard",
            "count": 1,
            "hp": 8,
            "ac": 12,
            "saveBonus": 0,
            "initiativeBonus": 0,
            "actions": [],
            "triggers": [],
            "initialBuffs": [{
                "displayName": "Test Buff",
                "duration": "entire encounter",
                "triggers": [{
                    "condition": "OnBeingDamaged",
                    "requirements": ["HasTempHP"],
                    "effect": {
                        "type": "SuppressBuff",
                        "buffId": "Test",
                        "duration": "1 round"
                    }
                }]
            }]
        }"#;

        let result: Result<Creature, _> = serde_json::from_str(json);
        match &result {
            Ok(_) => println!("Deserialization with triggers succeeded"),
            Err(e) => println!("Deserialization with triggers failed: {}", e),
        }
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
    }
}
