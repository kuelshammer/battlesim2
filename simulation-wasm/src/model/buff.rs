use super::formula::DiceFormula;
use crate::enums::{
    BuffDuration, CreatureCondition, TriggerCondition, TriggerEffect, TriggerRequirement,
};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectTrigger {
    pub condition: TriggerCondition,
    #[serde(default)]
    pub requirements: Vec<TriggerRequirement>,
    pub effect: TriggerEffect,
}

impl Hash for EffectTrigger {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash TriggerCondition using discriminant
        std::mem::discriminant(&self.condition).hash(state);
        // Hash variant fields
        match &self.condition {
            crate::enums::TriggerCondition::And { conditions } => {
                conditions.len().hash(state);
                for cond in conditions {
                    std::mem::discriminant(cond).hash(state);
                }
            }
            crate::enums::TriggerCondition::Or { conditions } => {
                conditions.len().hash(state);
                for cond in conditions {
                    std::mem::discriminant(cond).hash(state);
                }
            }
            crate::enums::TriggerCondition::Not { condition } => {
                std::mem::discriminant(condition.as_ref()).hash(state);
            }
            crate::enums::TriggerCondition::EnemyCountAtLeast { count } => {
                count.hash(state);
            }
            crate::enums::TriggerCondition::DamageExceedsPercent { threshold } => {
                crate::utils::hash_f64(*threshold, state);
            }
            crate::enums::TriggerCondition::BelowHpPercent { threshold } => {
                crate::utils::hash_f64(*threshold, state);
            }
            crate::enums::TriggerCondition::AboveHpPercent { threshold } => {
                crate::utils::hash_f64(*threshold, state);
            }
            _ => {}
        }
        self.requirements.hash(state);
        self.effect.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Buff {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub duration: BuffDuration,

    pub ac: Option<DiceFormula>,
    #[serde(rename = "toHit")]
    pub to_hit: Option<DiceFormula>,
    pub damage: Option<DiceFormula>,
    #[serde(rename = "damageReduction")]
    pub damage_reduction: Option<DiceFormula>,
    #[serde(rename = "damageMultiplier")]
    pub damage_multiplier: Option<f64>,
    #[serde(rename = "damageTakenMultiplier")]
    pub damage_taken_multiplier: Option<f64>,
    pub dc: Option<DiceFormula>,
    pub save: Option<DiceFormula>,
    pub condition: Option<CreatureCondition>,

    pub magnitude: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub concentration: bool,
    #[serde(default)]
    pub triggers: Vec<EffectTrigger>,
    #[serde(
        rename = "suppressedUntil",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub suppressed_until: Option<u32>,
}

impl Hash for Buff {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.display_name.hash(state);
        self.duration.hash(state);
        self.ac.hash(state);
        self.to_hit.hash(state);
        self.damage.hash(state);
        self.damage_reduction.hash(state);
        crate::utils::hash_opt_f64(self.damage_multiplier, state);
        crate::utils::hash_opt_f64(self.damage_taken_multiplier, state);
        self.dc.hash(state);
        self.save.hash(state);
        self.condition.hash(state);
        crate::utils::hash_opt_f64(self.magnitude, state);
        self.source.hash(state);
        self.concentration.hash(state);
        self.triggers.hash(state);
        self.suppressed_until.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiderEffect {
    pub dc: f64,
    pub buff: Buff,
}

impl Hash for RiderEffect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        crate::utils::hash_f64(self.dc, state);
        self.buff.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub enum CleanupInstruction {
    RemoveAllBuffsFromSource(String), // Combatant ID of the source that died
    BreakConcentration(String, String), // (Combatant ID of concentrator, Buff ID)
}
