use super::buff::{Buff, RiderEffect};
use super::formula::DiceFormula;
use crate::enums::{ActionCondition, AllyTarget, EnemyTarget, TargetType, TriggerCondition};
use crate::resources::{ActionCost, ActionRequirement, ActionTag};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(untagged)]
pub enum Frequency {
    Static(String), // "at will", "1/fight", "1/day"
    Recharge {
        reset: String, // "recharge"
        #[serde(rename = "cooldownRounds")]
        cooldown_rounds: i32,
    },
    Limited {
        reset: String, // "sr", "lr"
        uses: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "atk")]
    Atk(AtkAction),
    #[serde(rename = "heal")]
    Heal(HealAction),
    #[serde(rename = "buff")]
    Buff(BuffAction),
    #[serde(rename = "debuff")]
    Debuff(DebuffAction),
    #[serde(rename = "template")]
    Template(TemplateAction),
}

impl Action {
    pub fn base(&self) -> ActionBase {
        match self {
            Action::Atk(a) => a.base(),
            Action::Heal(a) => a.base(),
            Action::Buff(a) => a.base(),
            Action::Debuff(a) => a.base(),
            Action::Template(a) => a.base(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct ActionBase {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct AtkAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub dpr: DiceFormula,
    #[serde(rename = "toHit")]
    pub to_hit: DiceFormula,
    pub target: EnemyTarget,
    #[serde(rename = "useSaves")]
    pub use_saves: Option<bool>,
    #[serde(rename = "halfOnSave")]
    pub half_on_save: Option<bool>,
    #[serde(rename = "riderEffect")]
    pub rider_effect: Option<RiderEffect>,
}

impl AtkAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct HealAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub amount: DiceFormula,
    #[serde(rename = "tempHP")]
    pub temp_hp: Option<bool>,
    pub target: AllyTarget,
}

impl HealAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct BuffAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub target: AllyTarget,
    pub buff: Buff,
}

impl BuffAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebuffAction {
    pub id: String,
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    pub targets: i32,

    pub target: EnemyTarget,
    #[serde(rename = "saveDC")]
    pub save_dc: f64,
    pub buff: Buff,
}

impl Hash for DebuffAction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.action_slot.hash(state);
        self.cost.hash(state);
        self.requirements.hash(state);
        self.tags.hash(state);
        self.freq.hash(state);
        self.condition.hash(state);
        self.targets.hash(state);
        self.target.hash(state);
        crate::utils::hash_f64(self.save_dc, state);
        self.buff.hash(state);
    }
}

impl DebuffAction {
    pub fn base(&self) -> ActionBase {
        ActionBase {
            id: self.id.clone(),
            name: self.name.clone(),
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct TemplateAction {
    pub id: String,
    #[serde(default, deserialize_with = "default_template_name")]
    pub name: String,

    // Legacy field - kept for backward compatibility during transition
    #[serde(rename = "actionSlot", default, skip_serializing)]
    pub action_slot: Option<i32>,

    // New fields replacing action_slot
    #[serde(default)]
    pub cost: Vec<ActionCost>,
    #[serde(default)]
    pub requirements: Vec<ActionRequirement>,
    #[serde(default)]
    pub tags: Vec<ActionTag>,

    pub freq: Frequency,
    pub condition: ActionCondition,
    #[serde(default = "default_targets")]
    pub targets: i32,

    #[serde(rename = "templateOptions")]
    pub template_options: TemplateOptions,
}

fn default_targets() -> i32 {
    1
}

fn default_template_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Try to deserialize the name field, if it doesn't exist, return empty string
    // The actual name will be set from template_options.template_name in a post-processing step
    Option::<String>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateOptions {
    #[serde(rename = "templateName")]
    pub template_name: String,
    // Optional target that can be either ally or enemy
    pub target: Option<TargetType>,
    // Add other options as needed, like saveDC, amount, etc.
    #[serde(rename = "saveDC")]
    pub save_dc: Option<f64>,
    pub amount: Option<DiceFormula>,
}

impl Hash for TemplateOptions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.template_name.hash(state);
        self.target.hash(state);
        crate::utils::hash_opt_f64(self.save_dc, state);
        self.amount.hash(state);
    }
}

impl TemplateAction {
    pub fn base(&self) -> ActionBase {
        // Use template_name if name is empty
        let name = if self.name.is_empty() {
            self.template_options.template_name.clone()
        } else {
            self.name.clone()
        };

        ActionBase {
            id: self.id.clone(),
            name,
            action_slot: self.action_slot,
            cost: self.cost.clone(),
            requirements: self.requirements.clone(),
            tags: self.tags.clone(),
            freq: self.freq.clone(),
            condition: self.condition.clone(),
            targets: self.targets,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionTrigger {
    pub id: String,
    pub condition: TriggerCondition,
    pub action: Action,    // The action to execute when triggered
    pub cost: Option<i32>, // e.g. Reaction (4)
}

impl Hash for ActionTrigger {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
        self.action.hash(state);
        self.cost.hash(state);
    }
}
