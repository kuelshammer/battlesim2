use simulation_wasm::intensity_calculation::{calculate_power, calculate_vitality};
use std::collections::HashMap;

#[test]
fn test_vitality_calculation() {
    let current_hp = 50;
    let max_hp = 100;
    let con_mod = 3.0;
    let mut resources = HashMap::new();
    let mut max_resources = HashMap::new();

    // Test base vitality (HP only)
    let vit = calculate_vitality(current_hp, &resources, max_hp, &max_resources, con_mod);
    assert!(vit > 45.0 && vit < 55.0); // Roughly matches HP %

    // Test with Hit Dice (at full capacity)
    resources.insert("HitDice(d8)".to_string(), 10.0);
    max_resources.insert("HitDice(d8)".to_string(), 10.0);
    let vit_full_hd = calculate_vitality(current_hp, &resources, max_hp, &max_resources, con_mod);
    // vit was 50% (50/100).
    // Now it's (50 + 10*7.5) / (100 + 10*7.5) = 125 / 175 = 71.4%
    assert!(vit_full_hd > vit);

    // Test with Hit Dice (empty)
    resources.insert("HitDice(d8)".to_string(), 0.0);
    let vit_empty_hd = calculate_vitality(current_hp, &resources, max_hp, &max_resources, con_mod);
    // (50 + 0) / (100 + 75) = 50 / 175 = 28.5%
    assert!(vit_empty_hd < vit);
}

#[test]
fn test_power_calculation() {
    let mut resources = HashMap::new();
    let mut max_resources = HashMap::new();
    let reset_rules = HashMap::new(); // Empty rules means all resources are at-will/ignored?

    // Test with Spell Slots
    resources.insert("SpellSlot(1)".to_string(), 2.0);
    max_resources.insert("SpellSlot(1)".to_string(), 4.0);

    let pow = calculate_power(&resources, &max_resources, &reset_rules);
    assert!(pow > 0.0);

    resources.insert("SpellSlot(1)".to_string(), 0.0);
    let pow_empty = calculate_power(&resources, &max_resources, &reset_rules);
    assert!(pow_empty < pow); // Less resources = less power
}
