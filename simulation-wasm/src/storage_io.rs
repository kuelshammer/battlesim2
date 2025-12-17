use crate::storage::{SimulationDataSet, StorageConfig, StorageError};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

/// Compress simulation data using LZ4
pub fn compress_simulation_data(data: &SimulationDataSet) -> Result<Vec<u8>, StorageError> {
    let json_str = serde_json::to_string(data)
        .map_err(|e| StorageError::SerializationError(format!("Failed to serialize data: {}", e)))?;
    
    // lz4_flex::block::compress returns Vec<u8> directly, not a Result
    let compressed = lz4_flex::block::compress(json_str.as_bytes());
    
    if compressed.is_empty() && !json_str.is_empty() {
        return Err(StorageError::CompressionError("Compression failed - empty result".to_string()));
    }
    
    Ok(compressed)
}

/// Decompress simulation data using LZ4
pub fn decompress_simulation_data(compressed: &[u8]) -> Result<SimulationDataSet, StorageError> {
    let decompressed = lz4_flex::block::decompress(compressed, 10 * 1024 * 1024) // 10MB max
        .map_err(|_| StorageError::CompressionError("Failed to decompress data".to_string()))?;
    
    let json_str = String::from_utf8(decompressed)
        .map_err(|e| StorageError::SerializationError(format!("Invalid UTF-8 in decompressed data: {}", e)))?;
    
    serde_json::from_str(&json_str)
        .map_err(|e| StorageError::SerializationError(format!("Failed to deserialize data: {}", e)))
}

/// Get file path for a simulation slot
pub fn get_slot_file_path(config: &StorageConfig, slot: SlotFile) -> PathBuf {
    let filename = match slot {
        SlotFile::Primary => "simulation_primary.sim",
        SlotFile::Secondary => "simulation_secondary.sim",
    };
    
    Path::new(&config.storage_directory).join(filename)
}

/// Save simulation data to disk
pub fn save_simulation_to_disk(data: &SimulationDataSet, config: &StorageConfig, slot: SlotFile) -> Result<(), StorageError> {
    // Ensure storage directory exists
    fs::create_dir_all(&config.storage_directory)
        .map_err(|e| StorageError::IoError(format!("Failed to create storage directory: {}", e)))?;
    
    let file_path = get_slot_file_path(config, slot);
    
    if config.enable_compression {
        let compressed = compress_simulation_data(data)?;
        fs::write(&file_path, compressed)
            .map_err(|e| StorageError::IoError(format!("Failed to write compressed data to {:?}: {}", file_path, e)))?;
    } else {
        let json_str = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::SerializationError(format!("Failed to serialize data: {}", e)))?;
        fs::write(&file_path, json_str)
            .map_err(|e| StorageError::IoError(format!("Failed to write data to {:?}: {}", file_path, e)))?;
    }
    
    Ok(())
}

/// Load simulation data from disk
pub fn load_simulation_from_disk(config: &StorageConfig, slot: SlotFile) -> Result<Option<SimulationDataSet>, StorageError> {
    let file_path = get_slot_file_path(config, slot);
    
    if !file_path.exists() {
        return Ok(None);
    }
    
    let data = fs::read(&file_path)
        .map_err(|e| StorageError::IoError(format!("Failed to read data from {:?}: {}", file_path, e)))?;
    
    let simulation_data = if config.enable_compression {
        decompress_simulation_data(&data)?
    } else {
        let json_str = String::from_utf8(data)
            .map_err(|e| StorageError::SerializationError(format!("Invalid UTF-8 in file {:?}: {}", file_path, e)))?;
        serde_json::from_str(&json_str)
            .map_err(|e| StorageError::SerializationError(format!("Failed to deserialize data from {:?}: {}", file_path, e)))?
    };
    
    Ok(Some(simulation_data))
}

/// Clean up old simulation files
pub fn cleanup_old_files(config: &StorageConfig) -> Result<(), StorageError> {
    let storage_dir = Path::new(&config.storage_directory);
    
    if !storage_dir.exists() {
        return Ok(());
    }
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let entries = fs::read_dir(storage_dir)
        .map_err(|e| StorageError::IoError(format!("Failed to read storage directory: {}", e)))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| StorageError::IoError(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time = modified.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    if current_time - modified_time > config.max_age_seconds {
                        fs::remove_file(&path)
                            .map_err(|e| StorageError::IoError(format!("Failed to remove old file {:?}: {}", path, e)))?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Get total size of storage directory
pub fn get_storage_size(config: &StorageConfig) -> Result<u64, StorageError> {
    let storage_dir = Path::new(&config.storage_directory);
    
    if !storage_dir.exists() {
        return Ok(0);
    }
    
    let mut total_size = 0u64;
    
    fn calculate_dir_size(dir: &Path, total: &mut u64) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                *total += entry.metadata()?.len();
            } else if path.is_dir() {
                calculate_dir_size(&path, total)?;
            }
        }
        Ok(())
    }
    
    calculate_dir_size(storage_dir, &mut total_size)
        .map_err(|e| StorageError::IoError(format!("Failed to calculate storage size: {}", e)))?;
    
    Ok(total_size)
}

/// Check if storage quota is exceeded
pub fn is_quota_exceeded(config: &StorageConfig) -> Result<bool, StorageError> {
    let current_size = get_storage_size(config)?;
    Ok(current_size > config.max_storage_bytes)
}

/// Which slot file to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotFile {
    Primary,
    Secondary,
}

impl SlotFile {
    /// Get the opposite slot
    pub fn opposite(self) -> Self {
        match self {
            SlotFile::Primary => SlotFile::Secondary,
            SlotFile::Secondary => SlotFile::Primary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{SimulationDataSet, ScenarioParameters, SimulationConfig, SimulationMetadata, SimulationStatus, generate_simulation_id, SimulationTimestamp};
    
    fn create_test_data() -> SimulationDataSet {
        SimulationDataSet {
            id: generate_simulation_id(),
            timestamp: SimulationTimestamp::now(),
            parameter_hash: crate::storage::ParameterHash("test_hash".to_string()),
            parameters: ScenarioParameters {
                players: vec![],
                encounters: vec![],
                iterations: 100,
                config: SimulationConfig::default(),
            },
            results: vec![],
            metadata: SimulationMetadata {
                execution_time_ms: Some(1000),
                iterations_completed: 100,
                status: SimulationStatus::Success,
                messages: vec![],
            },
        }
    }
    
    #[test]
    fn test_compression_roundtrip() {
        let data = create_test_data();
        
        let compressed = compress_simulation_data(&data).unwrap();
        let decompressed = decompress_simulation_data(&compressed).unwrap();
        
        assert_eq!(data.id, decompressed.id);
        assert_eq!(data.parameter_hash, decompressed.parameter_hash);
    }
    
    #[test]
    fn test_file_operations() {
        let temp_dir = std::env::temp_dir().join("sim_test");
        let config = StorageConfig {
            storage_directory: temp_dir.to_string_lossy().to_string(),
            enable_compression: true,
            max_age_seconds: 3600,
            max_storage_bytes: 1024 * 1024,
        };
        
        let data = create_test_data();
        
        // Save to primary slot
        save_simulation_to_disk(&data, &config, SlotFile::Primary).unwrap();
        
        // Load from primary slot
        let loaded = load_simulation_from_disk(&config, SlotFile::Primary).unwrap();
        assert!(loaded.is_some());
        assert_eq!(data.id, loaded.unwrap().id);
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}