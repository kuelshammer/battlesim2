use crate::resources::{ResourceLedger, ResetType};
use std::collections::HashMap;

pub const HP_WEIGHT: f64 = 1.0;
pub const HIT_DIE_WEIGHT: f64 = 8.0;
pub const SPELL_SLOT_BASE: f64 = 15.0;
pub const SR_FEATURE_WEIGHT: f64 = 15.0;
pub const LR_FEATURE_WEIGHT: f64 = 30.0;

/// Calculates the Effective HP (EHP) points for a given set of resources.
pub fn calculate_ehp_points(current: &HashMap<String, f64>, reset_rules: &HashMap<String, ResetType>) -> f64 {
    let mut total = 0.0;

    for (key, &amount) in current {
        if amount <= 0.0 { continue; }

        if key == "HP" || key == "TempHP" {
            total += amount * HP_WEIGHT;
        } else if key.starts_with("HitDice") {
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
    calculate_ehp_points(&ledger.current, &ledger.reset_rules)
}

pub fn calculate_ledger_max_ehp(ledger: &ResourceLedger) -> f64 {
    calculate_ehp_points(&ledger.max, &ledger.reset_rules)
}
