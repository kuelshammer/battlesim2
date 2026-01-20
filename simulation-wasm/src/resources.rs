use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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
    ActionUsage,
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
            ResourceType::ActionUsage => val.unwrap_or("Action").to_string(),
            ResourceType::Custom => format!("Custom({})", val.unwrap_or("Default")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Hash for ResourceLedger {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Sort keys for deterministic hashing
        let mut sorted_current: Vec<_> = self.current.iter().collect();
        sorted_current.sort_by_key(|a| a.0);
        for (k, v) in sorted_current {
            k.hash(state);
            crate::utils::hash_f64(*v, state);
        }

        let mut sorted_max: Vec<_> = self.max.iter().collect();
        sorted_max.sort_by_key(|a| a.0);
        for (k, v) in sorted_max {
            k.hash(state);
            crate::utils::hash_f64(*v, state);
        }

        let mut sorted_rules: Vec<_> = self.reset_rules.iter().collect();
        sorted_rules.sort_by_key(|a| a.0);
        sorted_rules.hash(state);
    }
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
            let rule_rank = match rule {
                ResetType::Turn => 1,
                ResetType::Round => 2,
                ResetType::Encounter => 3,
                ResetType::ShortRest => 4,
                ResetType::LongRest => 5,
            };

            let event_rank = match reset_type {
                ResetType::Turn => 1,
                ResetType::Round => 2,
                ResetType::Encounter => 3,
                ResetType::ShortRest => 4,
                ResetType::LongRest => 5,
            };

            if rule_rank <= event_rank {
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

impl Hash for ActionCost {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ActionCost::Discrete {
                resource_type,
                resource_val,
                amount,
            } => {
                0.hash(state);
                resource_type.hash(state);
                resource_val.hash(state);
                crate::utils::hash_f64(*amount, state);
            }
            ActionCost::Variable {
                resource_type,
                resource_val,
                min,
                max,
            } => {
                1.hash(state);
                resource_type.hash(state);
                resource_val.hash(state);
                crate::utils::hash_f64(*min, state);
                crate::utils::hash_f64(*max, state);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CombatCondition {
    EnemyInRange(f64),
    IsSurprised,
    HasTempHP,
    // Add more as needed
}

impl Hash for CombatCondition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            CombatCondition::EnemyInRange(v) => {
                0.hash(state);
                crate::utils::hash_f64(*v, state);
            }
            CombatCondition::IsSurprised => 1.hash(state),
            CombatCondition::HasTempHP => 2.hash(state),
        }
    }
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

impl Hash for ActionRequirement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ActionRequirement::ResourceAvailable {
                resource_type,
                resource_val,
                amount,
            } => {
                0.hash(state);
                resource_type.hash(state);
                resource_val.hash(state);
                crate::utils::hash_f64(*amount, state);
            }
            ActionRequirement::CombatState { condition } => {
                1.hash(state);
                condition.hash(state);
            }
            ActionRequirement::StatusEffect { effect } => {
                2.hash(state);
                effect.hash(state);
            }
            ActionRequirement::Custom { description } => {
                3.hash(state);
                description.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
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

impl From<String> for ActionTag {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Melee" => ActionTag::Melee,
            "Ranged" => ActionTag::Ranged,
            "Spell" => ActionTag::Spell,
            "Weapon" => ActionTag::Weapon,
            "Fire" => ActionTag::Fire,
            "Cold" => ActionTag::Cold,
            "Lightning" => ActionTag::Lightning,
            "Poison" => ActionTag::Poison,
            "Acid" => ActionTag::Acid,
            "Thunder" => ActionTag::Thunder,
            "Necrotic" => ActionTag::Necrotic,
            "Radiant" => ActionTag::Radiant,
            "Psychic" => ActionTag::Psychic,
            "Force" => ActionTag::Force,
            "AoE" => ActionTag::AoE,
            "Concentration" => ActionTag::Concentration,
            "RequiresSomatic" => ActionTag::RequiresSomatic,
            "RequiresVerbal" => ActionTag::RequiresVerbal,
            "RequiresMaterial" => ActionTag::RequiresMaterial,
            "Abjuration" => ActionTag::Abjuration,
            "Conjuration" => ActionTag::Conjuration,
            "Divination" => ActionTag::Divination,
            "Enchantment" => ActionTag::Enchantment,
            "Evocation" => ActionTag::Evocation,
            "Illusion" => ActionTag::Illusion,
            "Necromancy" => ActionTag::Necromancy,
            "Transmutation" => ActionTag::Transmutation,
            "Healing" => ActionTag::Healing,
            "TempHP" => ActionTag::TempHP,
            "Utility" => ActionTag::Utility,
            "Movement" => ActionTag::Movement,
            "Social" => ActionTag::Social,
            "Attack" => ActionTag::Attack,
            "Defense" => ActionTag::Defense,
            "Support" => ActionTag::Support,
            "Control" => ActionTag::Control,
            "Buff" => ActionTag::Buff,
            "Damage" => ActionTag::Damage,
            "GWM" => ActionTag::GWM,
            "Sharpshooter" => ActionTag::Sharpshooter,
            _ => ActionTag::Custom(s),
        }
    }
}

impl From<ActionTag> for String {
    fn from(tag: ActionTag) -> Self {
        match tag {
            ActionTag::Melee => "Melee".to_string(),
            ActionTag::Ranged => "Ranged".to_string(),
            ActionTag::Spell => "Spell".to_string(),
            ActionTag::Weapon => "Weapon".to_string(),
            ActionTag::Fire => "Fire".to_string(),
            ActionTag::Cold => "Cold".to_string(),
            ActionTag::Lightning => "Lightning".to_string(),
            ActionTag::Poison => "Poison".to_string(),
            ActionTag::Acid => "Acid".to_string(),
            ActionTag::Thunder => "Thunder".to_string(),
            ActionTag::Necrotic => "Necrotic".to_string(),
            ActionTag::Radiant => "Radiant".to_string(),
            ActionTag::Psychic => "Psychic".to_string(),
            ActionTag::Force => "Force".to_string(),
            ActionTag::AoE => "AoE".to_string(),
            ActionTag::Concentration => "Concentration".to_string(),
            ActionTag::RequiresSomatic => "RequiresSomatic".to_string(),
            ActionTag::RequiresVerbal => "RequiresVerbal".to_string(),
            ActionTag::RequiresMaterial => "RequiresMaterial".to_string(),
            ActionTag::Abjuration => "Abjuration".to_string(),
            ActionTag::Conjuration => "Conjuration".to_string(),
            ActionTag::Divination => "Divination".to_string(),
            ActionTag::Enchantment => "Enchantment".to_string(),
            ActionTag::Evocation => "Evocation".to_string(),
            ActionTag::Illusion => "Illusion".to_string(),
            ActionTag::Necromancy => "Necromancy".to_string(),
            ActionTag::Transmutation => "Transmutation".to_string(),
            ActionTag::Healing => "Healing".to_string(),
            ActionTag::TempHP => "TempHP".to_string(),
            ActionTag::Utility => "Utility".to_string(),
            ActionTag::Movement => "Movement".to_string(),
            ActionTag::Social => "Social".to_string(),
            ActionTag::Attack => "Attack".to_string(),
            ActionTag::Defense => "Defense".to_string(),
            ActionTag::Support => "Support".to_string(),
            ActionTag::Control => "Control".to_string(),
            ActionTag::Buff => "Buff".to_string(),
            ActionTag::Damage => "Damage".to_string(),
            ActionTag::GWM => "GWM".to_string(),
            ActionTag::Sharpshooter => "Sharpshooter".to_string(),
            ActionTag::Custom(s) => s,
        }
    }
}



