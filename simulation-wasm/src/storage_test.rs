use crate::storage::{SimulationStorage, ScenarioParameters, SimulationConfig, generate_simulation_id, SimulationTimestamp, SimulationMetadata, SimulationStatus};
use crate::storage_manager::StorageManager;
use crate::model::Creature;

#[cfg(test)]
mod tests {
    use super::*;
    
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
            initiative_bonus: crate::model::DiceFormula::Value(0.0),
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
    fn test_dual_slot_storage() {
        let mut storage = SimulationStorage::default();
        
        let players = vec![create_test_creature("Player1", 10.0, 15.0)];
        let encounters = vec![];
        let iterations = 100;
        
        let parameters = ScenarioParameters {
            players: players.clone(),
            encounters: encounters.clone(),
            iterations,
            config: SimulationConfig::default(),
        };
        
        // Initially should need new simulation
        assert!(storage.needs_new_simulation(&parameters));
        
        // Store simulation data
        let data_set = crate::storage::SimulationDataSet {
            id: generate_simulation_id(),
            timestamp: SimulationTimestamp::now(),
            parameter_hash: storage.calculate_parameter_hash(&parameters),
            parameters: parameters.clone(),
            results: vec![],
            metadata: SimulationMetadata {
                execution_time_ms: Some(1000),
                iterations_completed: iterations,
                status: SimulationStatus::Success,
                messages: vec![],
            },
        };
        
        storage.store_simulation(data_set).unwrap();
        
        // Now should not need new simulation
        assert!(!storage.needs_new_simulation(&parameters));
        
        // Should be able to retrieve the data
        let retrieved = storage.get_simulation(&parameters);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.iterations_completed, iterations);
    }
    
    #[test]
    fn test_parameter_hashing() {
        let storage = SimulationStorage::default();
        
        let players1 = vec![create_test_creature("Player1", 10.0, 15.0)];
        let players2 = vec![create_test_creature("Player1", 10.0, 15.0)]; // Same stats
        let players3 = vec![create_test_creature("Player1", 12.0, 15.0)]; // Different HP
        
        let encounters = vec![];
        let iterations = 100;
        
        let params1 = ScenarioParameters {
            players: players1,
            encounters: encounters.clone(),
            iterations,
            config: SimulationConfig::default(),
        };
        
        let params2 = ScenarioParameters {
            players: players2,
            encounters: encounters.clone(),
            iterations,
            config: SimulationConfig::default(),
        };
        
        let params3 = ScenarioParameters {
            players: players3,
            encounters: encounters.clone(),
            iterations,
            config: SimulationConfig::default(),
        };
        
        let hash1 = storage.calculate_parameter_hash(&params1);
        let hash2 = storage.calculate_parameter_hash(&params2);
        let hash3 = storage.calculate_parameter_hash(&params3);
        
        // Same parameters should have same hash
        assert_eq!(hash1, hash2);
        
        // Different parameters should have different hash
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_storage_manager_caching() {
        let temp_dir = std::env::temp_dir().join("storage_manager_test");
        let config = crate::storage::StorageConfig {
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
    
    #[test]
    fn test_slot_selection() {
        let storage = SimulationStorage::default();
        
        let players = vec![create_test_creature("Player1", 10.0, 15.0)];
        let encounters = vec![];
        let iterations = 100;
        
        let parameters = ScenarioParameters {
            players: players.clone(),
            encounters: encounters.clone(),
            iterations,
            config: SimulationConfig::default(),
        };
        
        // With empty storage, should select primary slot
        let (slot, overwrite) = storage.select_target_slot(&parameters);
        assert_eq!(slot, crate::storage::SlotSelection::Primary);
        assert!(overwrite);
    }
}