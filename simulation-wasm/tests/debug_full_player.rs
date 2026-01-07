#[cfg(test)]
mod debug_full_player_tests {
    use simulation_wasm::model::Creature;
    use serde_json;
    use std::error::Error;

    // Test with actual player data structure from examples.ts
    #[test]
    fn test_full_example_player() {
        let json = r#"{
            "id": "test-fighter",
            "mode": "player",
            "name": "Fighter",
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
                "condition": "Default",
                "dpr": "1d8+3",
                "toHit": 5,
                "target": "enemy with least HP",
                "targets": 1,
                "cost": [{"type": "Discrete", "resourceType": "Action", "amount": 1}],
                "requirements": [],
                "tags": ["Melee", "Weapon", "Attack", "Damage"]
            }],
            "triggers": [],
            "initialBuffs": []
        }"#;

        let result: Result<Creature, _> = serde_json::from_str(json);
        match &result {
            Ok(_) => println!("✓ Full player deserialization succeeded"),
            Err(e) => {
                println!("✗ Full player deserialization failed: {}", e);
                // Try to identify which field is causing the issue
                if let Some(inner_err) = e.source() {
                    println!("  Inner error: {}", inner_err);
                }
            }
        }
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
    }

    // Test with buff containing trigger
    #[test]
    fn test_player_with_buff_trigger() {
        let json = r#"{
            "id": "test-wizard",
            "mode": "player",
            "name": "Wizard",
            "count": 1,
            "hp": 8,
            "ac": 12,
            "saveBonus": 0,
            "initiativeBonus": 0,
            "actions": [],
            "triggers": [],
            "initialBuffs": [{
                "displayName": "Armor of Agathys",
                "duration": "entire encounter",
                "triggers": [{
                    "condition": "OnBeingHit",
                    "requirements": [],
                    "effect": {
                        "type": "DealDamage",
                        "amount": "5",
                        "damageType": "Cold"
                    }
                }]
            }]
        }"#;

        let result: Result<Creature, _> = serde_json::from_str(json);
        match &result {
            Ok(_) => println!("✓ Player with buff trigger succeeded"),
            Err(e) => {
                println!("✗ Player with buff trigger failed: {}", e);
            }
        }
        assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
    }
}
