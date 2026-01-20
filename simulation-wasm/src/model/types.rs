use crate::resources::ResourceLedger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Calculate the distance between two positions in feet
/// Uses Euclidean distance for 3D space
pub fn calculate_distance(p1: &Position, p2: &Position) -> f64 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    let dz = p1.z - p2.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

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
