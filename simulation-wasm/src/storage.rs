// Stub storage module - functionality removed
// This module provides minimal type definitions to allow compilation

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioParameters {
    pub players: Vec<crate::model::Creature>,
    pub encounters: Vec<crate::model::Encounter>,
    pub iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SlotSelection {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub log_enabled: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self { log_enabled: false }
    }
}