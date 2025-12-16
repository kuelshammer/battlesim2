use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::model::{Creature, Encounter, SimulationResult};

/// Unique identifier for a simulation data set
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SimulationId(pub String);

/// Timestamp for tracking when simulations were created
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulationTimestamp(pub u64);

impl SimulationTimestamp {
    pub fn now() -> Self {
        SimulationTimestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    }
}

/// Hash of scenario parameters for quick comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParameterHash(pub String);

/// Complete set of simulation data including parameters, results, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationDataSet {
    /// Unique identifier for this simulation
    pub id: SimulationId,
    /// When this simulation was created
    pub timestamp: SimulationTimestamp,
    /// Hash of the input parameters
    pub parameter_hash: ParameterHash,
    /// The actual parameters used
    pub parameters: ScenarioParameters,
    /// Simulation results
    pub results: Vec<SimulationResult>,
    /// Metadata about the simulation
    pub metadata: SimulationMetadata,
}

/// Parameters that define a simulation scenario
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenarioParameters {
    /// Player creatures in the scenario
    pub players: Vec<Creature>,
    /// Enemy encounters
    pub encounters: Vec<Encounter>,
    /// Number of iterations to run
    pub iterations: usize,
    /// Additional configuration options
    pub config: SimulationConfig,
}

/// Configuration options for simulation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfig {
    /// Whether to enable detailed logging
    pub log_enabled: bool,
    /// Random seed for reproducible results
    pub seed: Option<u64>,
    /// Additional simulation options
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            log_enabled: false,
            seed: None,
            options: HashMap::new(),
        }
    }
}

/// Metadata about a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    /// How long the simulation took to run (in milliseconds)
    pub execution_time_ms: Option<u64>,
    /// Number of iterations actually completed
    pub iterations_completed: usize,
    /// Success/failure status
    pub status: SimulationStatus,
    /// Any warnings or errors that occurred
    pub messages: Vec<String>,
}

/// Status of a simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationStatus {
    /// Simulation completed successfully
    Success,
    /// Simulation failed with an error
    Failed(String),
    /// Simulation was cancelled
    Cancelled,
    /// Simulation is still in progress
    InProgress,
}

/// Dual-slot storage system for simulation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStorage {
    /// Primary storage slot (most recently used)
    pub primary_slot: Option<SimulationDataSet>,
    /// Secondary storage slot (second most recently used)
    pub secondary_slot: Option<SimulationDataSet>,
    /// Storage configuration
    pub config: StorageConfig,
}

/// Configuration for the storage system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Directory where simulation data is stored
    pub storage_directory: String,
    /// Whether to enable compression
    pub enable_compression: bool,
    /// Maximum age of data before cleanup (in seconds)
    pub max_age_seconds: u64,
    /// Maximum size of storage directory (in bytes)
    pub max_storage_bytes: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_directory: "simulation_cache".to_string(),
            enable_compression: true,
            max_age_seconds: 86400 * 7, // 7 days
            max_storage_bytes: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl Default for SimulationStorage {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

impl SimulationStorage {
    /// Create a new simulation storage system
    pub fn new(config: StorageConfig) -> Self {
        Self {
            primary_slot: None,
            secondary_slot: None,
            config,
        }
    }

    /// Check if a new simulation is needed for the given parameters
    pub fn needs_new_simulation(&self, parameters: &ScenarioParameters) -> bool {
        let param_hash = self.calculate_parameter_hash(parameters);
        
        // Check primary slot
        if let Some(ref data) = self.primary_slot {
            if data.parameter_hash == param_hash {
                return false;
            }
        }
        
        // Check secondary slot
        if let Some(ref data) = self.secondary_slot {
            if data.parameter_hash == param_hash {
                return false;
            }
        }
        
        true
    }

    /// Select the target slot for storing new simulation data
    /// Returns which slot to use and whether to overwrite existing data
    pub fn select_target_slot(&self, parameters: &ScenarioParameters) -> (SlotSelection, bool) {
        let param_hash = self.calculate_parameter_hash(parameters);
        
        // Check if parameters match either existing slot
        let primary_match = self.primary_slot.as_ref()
            .map(|data| data.parameter_hash == param_hash)
            .unwrap_or(false);
        let secondary_match = self.secondary_slot.as_ref()
            .map(|data| data.parameter_hash == param_hash)
            .unwrap_or(false);
        
        if primary_match {
            (SlotSelection::Primary, true)
        } else if secondary_match {
            (SlotSelection::Secondary, true)
        } else {
            // Select slot based on age - overwrite the oldest
            let primary_timestamp = self.primary_slot.as_ref()
                .map(|data| data.timestamp.0)
                .unwrap_or(0);
            let secondary_timestamp = self.secondary_slot.as_ref()
                .map(|data| data.timestamp.0)
                .unwrap_or(0);
            
            if primary_timestamp <= secondary_timestamp {
                (SlotSelection::Primary, true)
            } else {
                (SlotSelection::Secondary, true)
            }
        }
    }

    /// Store simulation data in the selected slot
    pub fn store_simulation(&mut self, data: SimulationDataSet) -> Result<(), StorageError> {
        let (slot_selection, _) = self.select_target_slot(&data.parameters);
        
        match slot_selection {
            SlotSelection::Primary => {
                self.primary_slot = Some(data);
            }
            SlotSelection::Secondary => {
                self.secondary_slot = Some(data);
            }
        }
        
        Ok(())
    }

    /// Retrieve simulation data if available for the given parameters
    pub fn get_simulation(&self, parameters: &ScenarioParameters) -> Option<&SimulationDataSet> {
        let param_hash = self.calculate_parameter_hash(parameters);
        
        // Check primary slot first (most recent)
        if let Some(ref data) = self.primary_slot {
            if data.parameter_hash == param_hash {
                return Some(data);
            }
        }
        
        // Check secondary slot
        if let Some(ref data) = self.secondary_slot {
            if data.parameter_hash == param_hash {
                return Some(data);
            }
        }
        
        None
    }

    /// Calculate hash for scenario parameters
    pub fn calculate_parameter_hash(&self, parameters: &ScenarioParameters) -> ParameterHash {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash players
        parameters.players.len().hash(&mut hasher);
        for player in &parameters.players {
            player.name.hash(&mut hasher);
            player.hp.to_bits().hash(&mut hasher);
            player.ac.to_bits().hash(&mut hasher);
            // Add other key fields that affect simulation
        }
        
        // Hash encounters
        parameters.encounters.len().hash(&mut hasher);
        for encounter in &parameters.encounters {
            encounter.monsters.len().hash(&mut hasher);
            for monster in &encounter.monsters {
                monster.name.hash(&mut hasher);
                monster.hp.to_bits().hash(&mut hasher);
                monster.ac.to_bits().hash(&mut hasher);
                // Add other key fields
            }
        }
        
        // Hash iterations and config
        parameters.iterations.hash(&mut hasher);
        parameters.config.log_enabled.hash(&mut hasher);
        parameters.config.seed.hash(&mut hasher);
        
        ParameterHash(format!("{:x}", hasher.finish()))
    }

    /// Calculate parameter similarity score (0.0 to 1.0)
    pub fn calculate_similarity(&self, params1: &ScenarioParameters, params2: &ScenarioParameters) -> f64 {
        let mut similarity = 0.0;
        let mut total_factors = 0.0;
        
        // Compare player count
        let player_count_sim = if params1.players.len() == params2.players.len() { 1.0 } else { 0.0 };
        similarity += player_count_sim;
        total_factors += 1.0;
        
        // Compare encounter count
        let encounter_count_sim = if params1.encounters.len() == params2.encounters.len() { 1.0 } else { 0.0 };
        similarity += encounter_count_sim;
        total_factors += 1.0;
        
        // Compare iterations
        let iteration_sim = if params1.iterations == params2.iterations { 1.0 } else { 0.0 };
        similarity += iteration_sim;
        total_factors += 1.0;
        
        // Compare total HP and AC as rough similarity indicators
        let total_hp_1: f64 = params1.players.iter().map(|p| p.hp).sum();
        let total_hp_2: f64 = params2.players.iter().map(|p| p.hp).sum();
        let hp_similarity = 1.0 - (total_hp_1 - total_hp_2).abs() / (total_hp_1 + total_hp_2 + 1.0);
        similarity += hp_similarity;
        total_factors += 1.0;
        
        if total_factors > 0.0 {
            similarity / total_factors
        } else {
            0.0
        }
    }
}

/// Which storage slot to use
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SlotSelection {
    Primary,
    Secondary,
}

/// Errors that can occur during storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageError {
    /// IO error during file operations
    IoError(String),
    /// Serialization/deserialization error
    SerializationError(String),
    /// Compression/decompression error
    CompressionError(String),
    /// Storage quota exceeded
    QuotaExceeded,
    /// Invalid data format
    InvalidData(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(msg) => write!(f, "IO error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            StorageError::QuotaExceeded => write!(f, "Storage quota exceeded"),
            StorageError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// Generate a unique simulation ID
pub fn generate_simulation_id() -> SimulationId {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    SimulationId(format!("sim_{}", timestamp))
}