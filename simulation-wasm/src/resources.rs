use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Action,
    BonusAction,
    Reaction,
    Movement,
    SpellSlot,
    ClassResource,
    ItemCharge,
    HitDice,   // Generic HitDice resource (might be deprecated or refactored)
    HitDiceD6, // Specific hit dice types
    HitDiceD8,
    HitDiceD10,
    HitDiceD12,
    HP,
    Custom,
}

impl ResourceType {
    pub fn to_key(&self, val: Option<&str>) -> String {
        match self {
            ResourceType::Action => "Action".to_string(),
            ResourceType::BonusAction => "BonusAction".to_string(),
            ResourceType::Reaction => "Reaction".to_string(),
            ResourceType::Movement => "Movement".to_string(),
            ResourceType::HP => "HP".to_string(),
            ResourceType::SpellSlot => format!("SpellSlot({})", val.unwrap_or("1")),
            ResourceType::ClassResource => format!("ClassResource({})", val.unwrap_or("Default")),
            ResourceType::ItemCharge => format!("ItemCharge({})", val.unwrap_or("Default")),
            ResourceType::HitDice => format!("HitDice({})", val.unwrap_or("1")), // Generic
            ResourceType::HitDiceD6 => "HitDice(d6)".to_string(),                // New
            ResourceType::HitDiceD8 => "HitDice(d8)".to_string(),                // New
            ResourceType::HitDiceD10 => "HitDice(d10)".to_string(),              // New
            ResourceType::HitDiceD12 => "HitDice(d12)".to_string(),              // New
            ResourceType::Custom => format!("Custom({})", val.unwrap_or("Default")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    ShortRest,
    LongRest,
    Turn,
    Round,
    Encounter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceLedger {
    pub current: HashMap<String, f64>,
    pub max: HashMap<String, f64>,
    pub reset_rules: HashMap<String, ResetType>,
}

impl Default for ResourceLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLedger {
    pub fn new() -> Self {
        ResourceLedger {
            current: HashMap::new(),
            max: HashMap::new(),
            reset_rules: HashMap::new(),
        }
    }

    pub fn register_resource(
        &mut self,
        resource: ResourceType,
        val: Option<&str>,
        max_amount: f64,
        reset_rule: Option<ResetType>,
    ) {
        let key = resource.to_key(val);
        self.max.insert(key.clone(), max_amount);
        self.current.insert(key.clone(), max_amount);
        if let Some(rule) = reset_rule {
            self.reset_rules.insert(key, rule);
        }
    }

    pub fn has(&self, resource: ResourceType, val: Option<&str>, amount: f64) -> bool {
        let key = resource.to_key(val);
        *self.current.get(&key).unwrap_or(&0.0) >= amount
    }

    pub fn consume(
        &mut self,
        resource: ResourceType,
        val: Option<&str>,
        amount: f64,
    ) -> Result<(), String> {
        let key = resource.to_key(val);
        let current_val = self.current.get(&key).unwrap_or(&0.0);
        if *current_val >= amount {
            self.current.insert(key, current_val - amount);
            Ok(())
        } else {
            Err(format!(
                "Insufficient resource: {} (Required: {}, Available: {})",
                key, amount, current_val
            ))
        }
    }

    pub fn restore(&mut self, resource: ResourceType, val: Option<&str>, amount: f64) {
        let key = resource.to_key(val);
        let current_val = *self.current.get(&key).unwrap_or(&0.0);
        let max_val = *self.max.get(&key).unwrap_or(&f64::MAX);

        let new_val = (current_val + amount).min(max_val);
        self.current.insert(key, new_val);
    }

    pub fn reset(&mut self, reset_type: ResetType) {
        for (key, rule) in &self.reset_rules {
            let should_reset = match (rule, &reset_type) {
                (ResetType::ShortRest, ResetType::ShortRest) |
                (ResetType::ShortRest, ResetType::LongRest) => true,
                
                (ResetType::LongRest, ResetType::LongRest) => true,
                
                (ResetType::Turn, ResetType::Turn) |
                (ResetType::Turn, ResetType::Round) |
                (ResetType::Turn, ResetType::Encounter) |
                (ResetType::Turn, ResetType::ShortRest) |
                (ResetType::Turn, ResetType::LongRest) => true,
                
                (ResetType::Round, ResetType::Round) |
                (ResetType::Round, ResetType::Encounter) |
                (ResetType::Round, ResetType::ShortRest) |
                (ResetType::Round, ResetType::LongRest) => true,
                
                (ResetType::Encounter, ResetType::Encounter) |
                (ResetType::Encounter, ResetType::ShortRest) |
                (ResetType::Encounter, ResetType::LongRest) => true,
                
                _ => false,
            };

            if should_reset {
                if let Some(max_val) = self.max.get(key) {
                    self.current.insert(key.clone(), *max_val);
                }
            }
        }
    }

    pub fn reset_by_type(&mut self, reset_type: &ResetType) {
        self.reset(reset_type.clone());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ActionCost {
    #[serde(rename = "Discrete")]
    Discrete {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        #[serde(rename = "resourceVal")]
        resource_val: Option<String>,
        amount: f64,
    },
    #[serde(rename = "Variable")]
    Variable {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        #[serde(rename = "resourceVal")]
        resource_val: Option<String>,
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombatCondition {
    EnemyInRange(f64),
    IsSurprised,
    HasTempHP,
    // Add more as needed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ActionRequirement {
    #[serde(rename = "ResourceAvailable")]
    ResourceAvailable {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        #[serde(rename = "resourceVal")]
        resource_val: Option<String>,
        amount: f64,
    },
    #[serde(rename = "CombatState")]
    CombatState { condition: CombatCondition },
    #[serde(rename = "StatusEffect")]
    StatusEffect { effect: String },
    #[serde(rename = "Custom")]
    Custom { description: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTag {
    // Damage types
    Melee,
    Ranged,
    Spell,
    Weapon,
    Fire,
    Cold,
    Lightning,
    Poison,
    Acid,
    Thunder,
    Necrotic,
    Radiant,
    Psychic,
    Force,

    // Properties
    AoE,
    Concentration,
    RequiresSomatic,
    RequiresVerbal,
    RequiresMaterial,

    // School/Source
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,

    // Special
    Healing,
    TempHP,
    Utility,
    Movement,
    Social,

    // Combat categories
    Attack,
    Defense,
    Support,
    Control,
    Buff,
    Damage,

    // Feature specifics
    GWM,
    Sharpshooter,

    // Custom tags
    Custom(String),
}

#[cfg(test)]
#[path = "./resources_test.rs"]
mod resources_test;
