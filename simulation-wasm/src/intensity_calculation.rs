use crate::resources::{ResourceLedger, ResetType};
use std::collections::HashMap;

pub const HP_WEIGHT: f64 = 1.0;
pub const HIT_DIE_WEIGHT: f64 = 8.0;
pub const SPELL_SLOT_BASE: f64 = 15.0;
pub const SR_FEATURE_WEIGHT: f64 = 15.0;
pub const LR_FEATURE_WEIGHT: f64 = 30.0;

/// Calculates the Effective HP (EHP) points for a given set of resources.
pub fn calculate_ehp_points(
    hp: u32,
    temp_hp: u32,
    current: &HashMap<String, f64>, 
    reset_rules: &HashMap<String, ResetType>
) -> f64 {
    let mut total = (hp as f64 + temp_hp as f64) * HP_WEIGHT;

    for (key, &amount) in current {
        if amount <= 0.0 { continue; }

        if key.starts_with("HitDice") {
            total += amount * HIT_DIE_WEIGHT;
        } else if key.starts_with("SpellSlot") {
            // Key format: SpellSlot(Level)
            if let Some(level_str) = extract_level(key) {
                if let Ok(level) = level_str.parse::<f64>() {
                    total += amount * SPELL_SLOT_BASE * level.powf(1.5);
                }
            }
        } else if key.starts_with("ClassResource") || key.starts_with("Custom") {
            // Check reset rule
            if let Some(reset) = reset_rules.get(key) {
                match reset {
                    ResetType::ShortRest => total += amount * SR_FEATURE_WEIGHT,
                    ResetType::LongRest => total += amount * LR_FEATURE_WEIGHT,
                    _ => {} // Other resets don't count towards daily budget
                }
            }
        }
    }

    total
}

/// Helper to extract level from "SpellSlot(N)"
fn extract_level(key: &str) -> Option<&str> {
    let start = key.find('(')? + 1;
    let end = key.find(')')?;
    if start < end {
        Some(&key[start..end])
    } else {
        None
    }
}

pub fn calculate_ledger_ehp(ledger: &ResourceLedger) -> f64 {
    calculate_ehp_points(0, 0, &ledger.current, &ledger.reset_rules)
}

pub fn calculate_ledger_max_ehp(creature: &crate::model::Creature, ledger: &ResourceLedger) -> f64 {
    calculate_ehp_points(creature.hp, 0, &ledger.max, &ledger.reset_rules)
}

pub fn calculate_serializable_ehp(
    hp: u32,
    temp_hp: u32,
    ledger: &crate::model::SerializableResourceLedger, 
    reset_rules: &HashMap<String, ResetType>
) -> f64 {
    calculate_ehp_points(hp, temp_hp, &ledger.current, reset_rules)
}

pub fn calculate_vitality(
    hp: u32,
    current: &HashMap<String, f64>,
    max_hp: u32,
    max: &HashMap<String, f64>,
    con_modifier: f64,
) -> f64 {
    let mut current_hd_val = 0.0;
    let mut max_hd_val = 0.0;

    for (key, &max_amt) in max {
        if key.starts_with("HitDice") {
            let die_size = if key.contains("D6") { 6.0 }
                else if key.contains("D8") { 8.0 }
                else if key.contains("D10") { 10.0 }
                else if key.contains("D12") { 12.0 }
                else { 8.0 };
            
            let avg_val = die_size / 2.0 + 0.5 + con_modifier;
            current_hd_val += current.get(key).cloned().unwrap_or(0.0) * avg_val;
            max_hd_val += max_amt * avg_val;
        }
    }

    let total_current = hp as f64 + current_hd_val;
    let total_max = max_hp as f64 + max_hd_val;

    if total_max > 0.0 {
        (total_current / total_max) * 100.0
    } else {
        0.0
    }
}

pub fn calculate_power(
    current: &HashMap<String, f64>,
    max: &HashMap<String, f64>,
    reset_rules: &HashMap<String, ResetType>,
) -> f64 {
    let mut current_val = 0.0;
    let mut max_val = 0.0;

    for (key, &max_amt) in max {
        let weight = if key.starts_with("SpellSlot") {
            if let Some(level_str) = extract_level(key) {
                if let Ok(level) = level_str.parse::<f64>() {
                    (15.0 * 1.6_f64.powf(level)).round()
                } else { 0.0 }
            } else { 0.0 }
        } else if key.starts_with("ClassResource") || key.starts_with("Custom") {
            match reset_rules.get(key) {
                Some(ResetType::ShortRest) => 10.0,
                Some(ResetType::LongRest) => 30.0,
                _ => 0.0
            }
        } else { 0.0 };

        if weight > 0.0 {
            current_val += current.get(key).cloned().unwrap_or(0.0) * weight;
            max_val += max_amt * weight;
        }
    }

    if max_val > 0.0 {
        (current_val / max_val) * 100.0
    } else {
        100.0 // Default to full power if no limited resources exist
    }
}
