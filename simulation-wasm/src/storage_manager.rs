use crate::storage::{SimulationStorage, SimulationDataSet, ScenarioParameters, StorageConfig, StorageError, SlotSelection, generate_simulation_id, SimulationTimestamp, SimulationMetadata, SimulationStatus};
use crate::storage_io::{save_simulation_to_disk, load_simulation_from_disk, cleanup_old_files, is_quota_exceeded, SlotFile};
use crate::model::{Creature, Encounter, SimulationResult};
use std::time::Instant;

/// Enhanced storage manager that integrates dual-slot memory storage with persistent disk storage
#[derive(Clone)]
pub struct StorageManager {
    /// In-memory dual-slot storage
    pub(crate) memory_storage: SimulationStorage,
    /// Configuration for storage behavior
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager with the given configuration
    pub fn new(config: StorageConfig) -> Result<Self, StorageError> {
        let mut manager = Self {
            memory_storage: SimulationStorage::new(config.clone()),
            config,
        };
        
        // Load existing data from disk
        manager.load_from_disk()?;
        
        Ok(manager)
    }
    
    /// Load existing simulation data from disk into memory
    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        // Load primary slot
        if let Some(primary_data) = load_simulation_from_disk(&self.config, SlotFile::Primary)? {
            self.memory_storage.primary_slot = Some(primary_data);
        }
        
        // Load secondary slot
        if let Some(secondary_data) = load_simulation_from_disk(&self.config, SlotFile::Secondary)? {
            self.memory_storage.secondary_slot = Some(secondary_data);
        }
        
        Ok(())
    }
    
    /// Check if a new simulation is needed for the given parameters
    pub fn needs_new_simulation(&self, players: &[Creature], encounters: &[Encounter], iterations: usize) -> bool {
        let parameters = ScenarioParameters {
            players: players.to_vec(),
            encounters: encounters.to_vec(),
            iterations,
            config: Default::default(),
        };
        
        self.memory_storage.needs_new_simulation(&parameters)
    }
    
    /// Get cached simulation results if available
    pub fn get_cached_results(&self, players: &[Creature], encounters: &[Encounter], iterations: usize) -> Option<Vec<SimulationResult>> {
        let parameters = ScenarioParameters {
            players: players.to_vec(),
            encounters: encounters.to_vec(),
            iterations,
            config: Default::default(),
        };
        
        self.memory_storage.get_simulation(&parameters).map(|data| data.results.clone())
    }
    
    /// Store simulation results with metadata
    pub fn store_simulation_results(
        &mut self,
        players: &[Creature],
        encounters: &[Encounter],
        iterations: usize,
        results: Vec<SimulationResult>,
        execution_time_ms: Option<u64>,
        status: SimulationStatus,
        messages: Vec<String>,
    ) -> Result<(), StorageError> {
        let parameters = ScenarioParameters {
            players: players.to_vec(),
            encounters: encounters.to_vec(),
            iterations,
            config: Default::default(),
        };
        
        let data_set = SimulationDataSet {
            id: generate_simulation_id(),
            timestamp: SimulationTimestamp::now(),
            parameter_hash: self.memory_storage.calculate_parameter_hash(&parameters),
            parameters,
            results,
            metadata: SimulationMetadata {
                execution_time_ms,
                iterations_completed: iterations,
                status,
                messages,
            },
        };
        
        // Store in memory
        self.memory_storage.store_simulation(data_set.clone())?;
        
        // Persist to disk
        let (slot_selection, _) = self.memory_storage.select_target_slot(&data_set.parameters);
        let slot_file = match slot_selection {
            SlotSelection::Primary => SlotFile::Primary,
            SlotSelection::Secondary => SlotFile::Secondary,
        };
        
        save_simulation_to_disk(&data_set, &self.config, slot_file)?;
        
        // Clean up old files if quota is exceeded
        if is_quota_exceeded(&self.config)? {
            cleanup_old_files(&self.config)?;
        }
        
        Ok(())
    }
    
    /// Run a simulation with automatic caching
    pub fn run_simulation_with_cache<F>(
        &mut self,
        players: &[Creature],
        encounters: &[Encounter],
        iterations: usize,
        simulation_fn: F,
    ) -> Result<Vec<SimulationResult>, StorageError>
    where
        F: FnOnce(&[Creature], &[Encounter], usize) -> Result<Vec<SimulationResult>, String>,
    {
        // Check if we have cached results
        if let Some(cached_results) = self.get_cached_results(players, encounters, iterations) {
            return Ok(cached_results);
        }
        
        // Run new simulation
        let start_time = Instant::now();
        let results = simulation_fn(players, encounters, iterations)
            .map_err(|e| StorageError::InvalidData(format!("Simulation failed: {}", e)))?;
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Store results
        self.store_simulation_results(
            players,
            encounters,
            iterations,
            results.clone(),
            Some(execution_time),
            SimulationStatus::Success,
            vec![],
        )?;
        
        Ok(results)
    }
    
    /// Get storage statistics
    pub fn get_storage_stats(&self) -> StorageStats {
        let primary_age = self.memory_storage.primary_slot.as_ref()
            .map(|data| data.timestamp.0)
            .unwrap_or(0);
        let secondary_age = self.memory_storage.secondary_slot.as_ref()
            .map(|data| data.timestamp.0)
            .unwrap_or(0);
        
        let current_time = SimulationTimestamp::now().0;
        
        StorageStats {
            primary_slot_occupied: self.memory_storage.primary_slot.is_some(),
            secondary_slot_occupied: self.memory_storage.secondary_slot.is_some(),
            primary_age_seconds: current_time.saturating_sub(primary_age),
            secondary_age_seconds: current_time.saturating_sub(secondary_age),
            total_cached_simulations: self.memory_storage.primary_slot.as_ref().map(|_| 1).unwrap_or(0)
                + self.memory_storage.secondary_slot.as_ref().map(|_| 1).unwrap_or(0),
        }
    }
    
    /// Clear all cached data
    pub fn clear_cache(&mut self) -> Result<(), StorageError> {
        self.memory_storage.primary_slot = None;
        self.memory_storage.secondary_slot = None;
        
        // Remove disk files
        let primary_path = crate::storage_io::get_slot_file_path(&self.config, SlotFile::Primary);
        let secondary_path = crate::storage_io::get_slot_file_path(&self.config, SlotFile::Secondary);
        
        if primary_path.exists() {
            std::fs::remove_file(&primary_path)
                .map_err(|e| StorageError::IoError(format!("Failed to remove primary cache file: {}", e)))?;
        }
        
        if secondary_path.exists() {
            std::fs::remove_file(&secondary_path)
                .map_err(|e| StorageError::IoError(format!("Failed to remove secondary cache file: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Force cleanup of old files
    pub fn cleanup(&mut self) -> Result<(), StorageError> {
        cleanup_old_files(&self.config)?;
        self.load_from_disk()?; // Reload after cleanup
        Ok(())
    }
}

/// Statistics about the storage system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StorageStats {
    /// Whether primary slot is occupied
    pub primary_slot_occupied: bool,
    /// Whether secondary slot is occupied
    pub secondary_slot_occupied: bool,
    /// Age of primary slot data in seconds
    pub primary_age_seconds: u64,
    /// Age of secondary slot data in seconds
    pub secondary_age_seconds: u64,
    /// Total number of cached simulations
    pub total_cached_simulations: u32,
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new(StorageConfig::default()).unwrap_or_else(|_| {
            Self {
                memory_storage: SimulationStorage::default(),
                config: StorageConfig::default(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Creature, Action, AtkAction, ActionBase, DiceFormula};
    
    fn create_test_creature(name: &str, hp: f64, ac: f64) -> Creature {
        Creature {
            id: name.to_string(),
            arrival: None,
            mode: "player".to_string(),
            name: name.to_string(),
            count: 1.0,
            hp,
            ac,
            speed_fly: None,
            save_bonus: 0.0,
            str_save_bonus: None,
            dex_save_bonus: None,
            con_save_bonus: None,
            int_save_bonus: None,
            wis_save_bonus: None,
            cha_save_bonus: None,
            con_save_advantage: None,
            save_advantage: None,
            initiative_bonus: DiceFormula::Value(0.0),
            initiative_advantage: false,
            actions: vec![],
            triggers: vec![],
            spell_slots: None,
            class_resources: None,
            hit_dice: None,
            con_modifier: None,
        }
    }
    
    #[test]
    fn test_storage_manager_caching() {
        let temp_dir = std::env::temp_dir().join("storage_manager_test");
        let config = StorageConfig {
            storage_directory: temp_dir.to_string_lossy().to_string(),
            enable_compression: true,
            max_age_seconds: 3600,
            max_storage_bytes: 1024 * 1024,
        };
        
        let mut manager = StorageManager::new(config).unwrap();
        
        let players = vec![create_test_creature("Player1", 10.0, 15.0)];
        let encounters = vec![];
        let iterations = 10;
        
        // First call should run simulation
        let results1 = manager.run_simulation_with_cache(
            &players,
            &encounters,
            iterations,
            |_, _, _| Ok(vec![]),
        ).unwrap();
        
        // Second call should use cache
        let results2 = manager.run_simulation_with_cache(
            &players,
            &encounters,
            iterations,
            |_, _, _| panic!("Should not be called due to cache"),
        ).unwrap();
        
        // Both should be empty vectors, so we can compare lengths
        assert_eq!(results1.len(), results2.len());
        
        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}