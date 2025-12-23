use simulation_wasm::intensity_calculation::{calculate_vitality, calculate_power};
use simulation_wasm::resources::ResetType;
use std::collections::HashMap;

#[test]
fn test_vitality_calculation() {
    let mut current = HashMap::new();
    let mut max = HashMap::new();
    
    // Creature with 100 HP and 2d10 HD (+3 Con)
    // Die average = 5.5. HD Value = 5.5 + 3 = 8.5 per die.
    // Max HD Value = 2 * 8.5 = 17.
    // Total Max = 100 + 17 = 117.
    
    current.insert("HitDiceD10".to_string(), 2.0);
    max.insert("HitDiceD10".to_string(), 2.0);
    
    let vit = calculate_vitality(100, &current, 100, &max, 3.0);
    assert!((vit - 100.0).abs() < 0.1, "Vitality should be 100% at start");
    
    // Half HP, full HD
    // Total Current = 50 + 17 = 67.
    // 67 / 117 = 57.26%
    let vit_half_hp = calculate_vitality(50, &current, 100, &max, 3.0);
    assert!((vit_half_hp - 57.26).abs() < 0.1, "Vitality should be ~57% with half HP");
    
    // Full HP, 0 HD
    // Total Current = 100 + 0 = 100.
    // 100 / 117 = 85.47%
    current.insert("HitDiceD10".to_string(), 0.0);
    let vit_no_hd = calculate_vitality(100, &current, 100, &max, 3.0);
    assert!((vit_no_hd - 85.47).abs() < 0.1, "Vitality should be ~85% with no HD");
}

#[test]
fn test_power_calculation() {
    let mut current = HashMap::new();
    let mut max = HashMap::new();
    let mut reset_rules = HashMap::new();
    
    // 2x Level 1 slots, 1x Action Surge (SR)
    // Slot value: round(15 * 1.6^1) = round(24.0) = 24. Total slots max = 2 * 24 = 48.
    // Feature value: SR = 10.
    // Total Max = 48 + 10 = 58.
    
    max.insert("SpellSlot(1)".to_string(), 2.0);
    current.insert("SpellSlot(1)".to_string(), 2.0);
    
    max.insert("ClassResource(ActionSurge)".to_string(), 1.0);
    current.insert("ClassResource(ActionSurge)".to_string(), 1.0);
    reset_rules.insert("ClassResource(ActionSurge)".to_string(), ResetType::ShortRest);
    
    let pow = calculate_power(&current, &max, &reset_rules);
    assert!((pow - 100.0).abs() < 0.1, "Power should be 100% at start");
    
    // Used 1 slot
    // Current = 1 * 24 + 10 = 34.
    // 34 / 58 = 58.62%
    current.insert("SpellSlot(1)".to_string(), 1.0);
    let pow_half_slots = calculate_power(&current, &max, &reset_rules);
    assert!((pow_half_slots - 58.62).abs() < 0.1, "Power should be ~58.6% with half slots");
    
    // Used Action Surge
    // Current = 2 * 24 + 0 = 48.
    // 48 / 58 = 82.75%
    current.insert("SpellSlot(1)".to_string(), 2.0);
    current.insert("ClassResource(ActionSurge)".to_string(), 0.0);
    let pow_no_sr = calculate_power(&current, &max, &reset_rules);
    assert!((pow_no_sr - 82.75).abs() < 0.1, "Power should be ~82.7% with no Action Surge");
}
