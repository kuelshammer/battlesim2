use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Action,
    BonusAction,
    Reaction,
    Movement, 
    SpellSlot(u8),
    ClassResource(String), // "Rage", "Ki", "SorceryPoints"
    ItemCharge(String),
    HitDice(u8),
    HP, 
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResetType {
    ShortRest,
    LongRest,
    Turn,
    Round,
    Encounter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLedger {
    pub current: HashMap<ResourceType, f64>,
    pub max: HashMap<ResourceType, f64>,
    pub reset_rules: HashMap<ResourceType, ResetType>,
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

    pub fn register_resource(&mut self, resource: ResourceType, max_amount: f64, reset_rule: Option<ResetType>) {
        self.max.insert(resource.clone(), max_amount);
        self.current.insert(resource.clone(), max_amount); // Start full? Usually yes.
        if let Some(rule) = reset_rule {
            self.reset_rules.insert(resource, rule);
        }
    }

    pub fn has(&self, resource: &ResourceType, amount: f64) -> bool {
        *self.current.get(resource).unwrap_or(&0.0) >= amount
    }

    pub fn consume(&mut self, resource: &ResourceType, amount: f64) -> Result<(), String> {
        let current_val = self.current.get(resource).unwrap_or(&0.0);
        if *current_val >= amount {
            self.current.insert(resource.clone(), current_val - amount);
            Ok(())
        } else {
            Err(format!("Insufficient resource: {:?} (Required: {}, Available: {})", resource, amount, current_val))
        }
    }

    pub fn restore(&mut self, resource: &ResourceType, amount: f64) {
        let current_val = *self.current.get(resource).unwrap_or(&0.0);
        // If max is not set, assume no cap (or maybe we should treat 0 as no max? safer to rely on max map)
        // If not in max map, use f64::MAX? Or current_val? 
        // Let's say if it's not registered in max, it can go up indefinitely (like temp HP or custom counters)
        // BUT for standard resources, max should be registered.
        let max_val = *self.max.get(resource).unwrap_or(&f64::MAX);
        
        let new_val = (current_val + amount).min(max_val);
        self.current.insert(resource.clone(), new_val);
    }
    
    pub fn reset(&mut self, reset_type: ResetType) {
        for (resource, rule) in &self.reset_rules {
            match (rule, &reset_type) {
                (ResetType::ShortRest, ResetType::ShortRest) |
                (ResetType::ShortRest, ResetType::LongRest) |
                (ResetType::LongRest, ResetType::LongRest) => {
                     if let Some(max_val) = self.max.get(resource) {
                         self.current.insert(resource.clone(), *max_val);
                     }
                },
                _ => {}
            }
        }
    }

    /// Reset resources by reset type (alias for reset method)
    pub fn reset_by_type(&mut self, reset_type: &ResetType) {
        self.reset(reset_type.clone());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionCost {
    #[serde(rename = "Discrete")]
    Discrete {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        amount: f64,
    },
    #[serde(rename = "Variable")]
    Variable {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatCondition {
    EnemyInRange(f64),
    IsSurprised,
    // Add more as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionRequirement {
    #[serde(rename = "ResourceAvailable")]
    ResourceAvailable {
        #[serde(rename = "resourceType")]
        resource_type: ResourceType,
        amount: f64,
    },
    #[serde(rename = "CombatState")]
    CombatState {
        condition: CombatCondition,
    },
    #[serde(rename = "StatusEffect")]
    StatusEffect {
        effect: String,
    },
    #[serde(rename = "Custom")]
    Custom {
        description: String,
    },
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
    Utility,
    Movement,
    Social,

    // Combat categories
    Attack,
    Defense,
    Support,
    Control,

    // Custom tags
    Custom(String),
}

#[cfg(test)]
#[path = "./resources_test.rs"]
mod resources_test;
