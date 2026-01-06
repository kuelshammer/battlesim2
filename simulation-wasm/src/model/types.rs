use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::resources::ResourceLedger;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct AcKnowledge {
    pub min: i32,
    pub max: i32,
}

impl Default for AcKnowledge {
    fn default() -> Self {
        Self { min: 0, max: 30 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableResourceLedger {
    pub current: HashMap<String, f64>,
    pub max: HashMap<String, f64>,
}

impl From<ResourceLedger> for SerializableResourceLedger {
    fn from(ledger: ResourceLedger) -> Self {
        let current = ledger.current.into_iter().collect();
        let max = ledger.max.into_iter().collect();
        SerializableResourceLedger { current, max }
    }
}

impl From<SerializableResourceLedger> for ResourceLedger {
    fn from(ledger: SerializableResourceLedger) -> Self {
        ResourceLedger {
            current: ledger.current,
            max: ledger.max,
            reset_rules: HashMap::new(),
        }
    }
}
