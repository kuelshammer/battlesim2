#[cfg(test)]
mod tests {
    use crate::intensity_calculation::*;
    use crate::resources::*;
    use std::collections::HashMap;

    #[test]
    fn test_ehp_weighting() {
        let mut current = HashMap::new();
        let mut rules = HashMap::new();

        // 10 HP
        current.insert("HP".to_string(), 10.0);
        
        // 1 Hit Die (d8)
        current.insert("HitDice(d8)".to_string(), 1.0);
        
        // 1 Lvl 1 Spell Slot
        current.insert("SpellSlot(1)".to_string(), 1.0);
        
        // 1 Action Surge (SR)
        current.insert("ClassResource(Action Surge)".to_string(), 1.0);
        rules.insert("ClassResource(Action Surge)".to_string(), ResetType::ShortRest);

        let ehp = calculate_ehp_points(&current, &rules);
        
        // HP: 10 * 1 = 10
        // HD: 1 * 8 = 8
        // Slot: 1 * 15 * 1^1.5 = 15
        // SR: 1 * 15 = 15
        // Total: 48
        assert_eq!(ehp, 48.0);
    }

    #[test]
    fn test_spell_slot_scaling() {
        let mut current = HashMap::new();
        let rules = HashMap::new();

        // 1 Lvl 3 Spell Slot
        current.insert("SpellSlot(3)".to_string(), 1.0);
        
        let ehp = calculate_ehp_points(&current, &rules);
        
        // 15 * 3^1.5 = 15 * 5.196 = 77.94
        assert!((ehp - 77.94).abs() < 0.01);
    }
}
