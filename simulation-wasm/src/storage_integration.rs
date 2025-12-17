use crate::background_simulation::{BackgroundSimulationId, BackgroundSimulationResult, SimulationProgress};
use crate::queue_manager::{SimulationQueue, SimulationRequest, QueueError};
use crate::progress_communication::{ProgressCommunication, ProgressUpdate, ProgressUpdateType, ProgressError};
use crate::storage_manager::StorageManager;
use crate::storage::{ScenarioParameters, SimulationStatus, generate_simulation_id};
use crate::model::{Creature, Encounter, SimulationResult};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::HashMap;

/// Integration layer that connects background processing with the storage system
pub struct StorageIntegration {
    /// Storage manager for persisting results
    storage_manager: Arc<Mutex<StorageManager>>,
    /// Queue for managing simulation requests
    queue: Arc<SimulationQueue>,
    /// Progress communication system
    pub progress_comm: Arc<ProgressCommunication>,
    /// Active background simulations
    active_simulations: Arc<Mutex<HashMap<BackgroundSimulationId, ActiveSimulationInfo>>>,
    /// Configuration for integration behavior
    config: StorageIntegrationConfig,
}

/// Information about an active simulation
#[derive(Debug)]
struct ActiveSimulationInfo {
    /// Request ID that started this simulation
    request_id: String,
    /// When the simulation was started
    start_time: Instant,
    /// Parameters for this simulation
    parameters: ScenarioParameters,
    /// Current progress tracking
    last_progress_update: Option<Instant>,
}

/// Configuration for storage integration behavior
#[derive(Debug, Clone)]
pub struct StorageIntegrationConfig {
    /// Maximum number of concurrent simulations
    pub max_concurrent_simulations: usize,
    /// Progress update interval in milliseconds
    pub progress_update_interval_ms: u64,
    /// Whether to automatically store completed simulations
    pub auto_store_completed: bool,
    /// Whether to clean up old simulations when storage is full
    pub auto_cleanup: bool,
    /// Maximum age for completed simulations before cleanup (in seconds)
    pub max_completed_age_seconds: u64,
}

impl Default for StorageIntegrationConfig {
    fn default() -> Self {
        Self {
            max_concurrent_simulations: 3,
            progress_update_interval_ms: 500,
            auto_store_completed: true,
            auto_cleanup: true,
            max_completed_age_seconds: 3600, // 1 hour
        }
    }
}

/// Result of submitting a simulation request
#[derive(Debug, Clone)]
pub struct SubmissionResult {
    /// Unique request identifier
    pub request_id: String,
    /// Simulation identifier (if started immediately)
    pub simulation_id: Option<BackgroundSimulationId>,
    /// Whether the request was queued or started immediately
    pub queued: bool,
    /// Estimated wait time in milliseconds
    pub estimated_wait_ms: Option<u64>,
}

/// Status of a simulation request
#[derive(Debug, Clone)]
pub enum SimulationRequestStatus {
    /// Request is queued waiting to be processed
    Queued {
        position: usize,
        estimated_wait_ms: Option<u64>,
    },
    /// Simulation is currently running
    Running {
        progress_percentage: f64,
        estimated_time_remaining_ms: Option<u64>,
    },
    /// Simulation completed successfully
    Completed {
        execution_time_ms: u64,
        results_available: bool,
    },
    /// Simulation failed
    Failed {
        error: String,
    },
    /// Simulation was cancelled
    Cancelled,
    /// Request not found
    NotFound,
}

impl StorageIntegration {
    /// Create a new storage integration system
    pub fn new(
        storage_manager: StorageManager,
        queue: SimulationQueue,
        progress_comm: ProgressCommunication,
        config: StorageIntegrationConfig,
    ) -> Self {
        Self {
            storage_manager: Arc::new(Mutex::new(storage_manager)),
            queue: Arc::new(queue),
            progress_comm: Arc::new(progress_comm),
            active_simulations: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Submit a new simulation request
    pub fn submit_simulation_request(
        &self,
        parameters: ScenarioParameters,
        priority: crate::background_simulation::SimulationPriority,
    ) -> Result<SubmissionResult, StorageIntegrationError> {
        // Check if we have cached results
        if self.config.auto_store_completed {
            if let Ok(storage_manager) = self.storage_manager.try_lock() {
                if let Some(_cached_results) = storage_manager.get_cached_results(
                    &parameters.players,
                    &parameters.encounters,
                    parameters.iterations,
                ) {
                    // We have cached results, no need to run simulation
                    return Ok(SubmissionResult {
                        request_id: format!("cached_{}", generate_simulation_id().0),
                        simulation_id: None,
                        queued: false,
                        estimated_wait_ms: Some(0),
                    });
                }
            }
        }

        // Create simulation request
        let request = SimulationRequest::new(parameters.clone(), priority);
        let request_id = request.request_id.clone();

        // Check if we can start immediately
        let can_start_immediately = {
            let active = self.active_simulations.lock().unwrap();
            active.len() < self.config.max_concurrent_simulations
        };

        if can_start_immediately {
            // Start simulation immediately
            // Note: In a real implementation, this would integrate with BackgroundSimulationEngine
            // For now, we'll simulate the immediate start
            let simulation_id = BackgroundSimulationId::new();
            
            // Track the active simulation
            {
                let mut active = self.active_simulations.lock().unwrap();
                active.insert(simulation_id.clone(), ActiveSimulationInfo {
                    request_id: request_id.clone(),
                    start_time: Instant::now(),
                    parameters: parameters.clone(),
                    last_progress_update: Some(Instant::now()),
                });
            }

            // Send initial progress update
            let progress_update = ProgressUpdate::new(
                simulation_id.clone(),
                ProgressUpdateType::Started,
                0.0,
                "Starting simulation",
            );
            let _ = self.progress_comm.send_update(progress_update);

            Ok(SubmissionResult {
                request_id,
                simulation_id: Some(simulation_id),
                queued: false,
                estimated_wait_ms: Some(0),
            })
        } else {
            // Queue the request
            self.queue.enqueue(request)
                .map_err(|e| StorageIntegrationError::QueueError(e))?;

            // Estimate wait time based on queue position
            let queue_stats = self.queue.get_stats();
            let estimated_wait_ms = if queue_stats.pending_count > 0 {
                // Rough estimate: 5 seconds per pending request
                Some(queue_stats.pending_count as u64 * 5000)
            } else {
                None
            };

            Ok(SubmissionResult {
                request_id,
                simulation_id: None,
                queued: true,
                estimated_wait_ms,
            })
        }
    }

    /// Process the next request from the queue
    pub fn process_next_request(&self) -> Result<Option<BackgroundSimulationId>, StorageIntegrationError> {
        // Check if we have capacity for more simulations
        {
            let active = self.active_simulations.lock().unwrap();
            if active.len() >= self.config.max_concurrent_simulations {
                return Ok(None);
            }
        }

        // Get next request from queue
        if let Some(request) = self.queue.dequeue() {
            let simulation_id = BackgroundSimulationId::new();
            let request_id = request.request_id.clone();

            // Track the active simulation
            {
                let mut active = self.active_simulations.lock().unwrap();
                active.insert(simulation_id.clone(), ActiveSimulationInfo {
                    request_id,
                    start_time: Instant::now(),
                    parameters: request.parameters,
                    last_progress_update: Some(Instant::now()),
                });
            }

            // Send initial progress update
            let progress_update = ProgressUpdate::new(
                simulation_id.clone(),
                ProgressUpdateType::Started,
                0.0,
                "Starting simulation",
            );
            let _ = self.progress_comm.send_update(progress_update);

            Ok(Some(simulation_id))
        } else {
            Ok(None)
        }
    }

    /// Update progress for a running simulation
    pub fn update_simulation_progress(
        &self,
        simulation_id: &BackgroundSimulationId,
        progress: &SimulationProgress,
    ) -> Result<(), StorageIntegrationError> {
        // Update last progress time
        {
            let mut active = self.active_simulations.lock().unwrap();
            if let Some(info) = active.get_mut(simulation_id) {
                info.last_progress_update = Some(Instant::now());
            }
        }

        // Send progress update
        let progress_update = ProgressUpdate::from_progress(progress, ProgressUpdateType::Progress);
        self.progress_comm.send_update(progress_update)
            .map_err(|e| StorageIntegrationError::ProgressError(e))?;

        Ok(())
    }

    /// Handle completion of a simulation
    pub fn handle_simulation_completion(
        &self,
        result: BackgroundSimulationResult,
    ) -> Result<(), StorageIntegrationError> {
        let simulation_id = &result.simulation_id;

        // Remove from active simulations
        let parameters = {
            let mut active = self.active_simulations.lock().unwrap();
            active.remove(simulation_id)
                .map(|info| info.parameters)
        };

        // Send completion update
        let update_type = if result.success {
            ProgressUpdateType::Completed
        } else {
            ProgressUpdateType::Failed
        };

        let progress_update = ProgressUpdate::new(
            simulation_id.clone(),
            update_type,
            if result.success { 1.0 } else { result.metadata.iterations_completed as f64 / result.metadata.iterations_completed.max(1) as f64 },
            if result.success { "Completed" } else { "Failed" },
        ).with_message(if result.success {
            "Simulation completed successfully".to_string()
        } else {
            result.error.clone().unwrap_or_else(|| "Simulation failed".to_string())
        });

        self.progress_comm.send_update(progress_update)
            .map_err(|e| StorageIntegrationError::ProgressError(e))?;

        // Store results if successful and auto-store is enabled
        if result.success && self.config.auto_store_completed {
            if let (Some(parameters), Some(results)) = (parameters, &result.results) {
                let mut storage_manager = self.storage_manager.lock().unwrap();
                
                storage_manager.store_simulation_results(
                    &parameters.players,
                    &parameters.encounters,
                    parameters.iterations,
                    results.clone(),
                    Some(result.execution_time_ms),
                    SimulationStatus::Success,
                    result.metadata.messages.clone(),
                ).map_err(|e| StorageIntegrationError::StorageError(e.to_string()))?;
            }
        }

        // Mark queue request as completed
        // Note: We need to track which request ID corresponds to this simulation
        // This would be handled in the ActiveSimulationInfo in a real implementation

        Ok(())
    }

    /// Cancel a simulation request
    pub fn cancel_request(&self, request_id: &str) -> Result<(), StorageIntegrationError> {
        // Try to cancel from queue first
        if let Err(e) = self.queue.cancel_request(request_id) {
            match e {
                QueueError::RequestNotFound => {
                    // Might be an active simulation, try to cancel it
                    let mut active = self.active_simulations.lock().unwrap();
                    let mut to_cancel = None;
                    
            for (sim_id, info) in active.iter() {
                if info.request_id == request_id {
                    to_cancel = Some(sim_id.clone());
                    break;
                }
            }
                    
                    if let Some(sim_id) = to_cancel {
                        // Send cancellation update
                        let progress_update = ProgressUpdate::new(
                            sim_id.clone(),
                            ProgressUpdateType::Cancelled,
                            0.0,
                            "Cancelled",
                        );
                        let _ = self.progress_comm.send_update(progress_update);
                        
                        // Remove from active
                        active.remove(&sim_id);
                        
                        return Ok(());
                    } else {
                        return Err(StorageIntegrationError::RequestNotFound);
                    }
                }
                other => return Err(StorageIntegrationError::QueueError(other)),
            }
        }

        Ok(())
    }

    /// Get status of a simulation request
    pub fn get_request_status(&self, request_id: &str) -> SimulationRequestStatus {
        // Check if it's in the queue
        if self.queue.is_pending(request_id) {
            let queue_stats = self.queue.get_stats();
            let position = queue_stats.pending_count; // Simplified - would need actual position
            let estimated_wait_ms = Some(position as u64 * 5000); // Rough estimate
            
            return SimulationRequestStatus::Queued {
                position,
                estimated_wait_ms,
            };
        }

        // Check if it's an active simulation
        {
            let active = self.active_simulations.lock().unwrap();
            for (_sim_id, info) in active.iter() {
                if info.request_id == request_id {
                    let elapsed = info.start_time.elapsed().as_millis() as u64;
                    let progress = if info.parameters.iterations > 0 {
                        // Rough progress estimate based on time
                        let estimated_total_time = info.parameters.iterations as u64 * 100; // 100ms per iteration
                        (elapsed as f64 / estimated_total_time as f64).min(0.99)
                    } else {
                        0.0
                    };
                    
                    return SimulationRequestStatus::Running {
                        progress_percentage: progress,
                        estimated_time_remaining_ms: Some(
                            ((1.0 - progress) * elapsed as f64) as u64
                        ),
                    };
                }
            }
        }

        // Check if results are available in storage
        // This would require tracking completed requests in a real implementation
        
        SimulationRequestStatus::NotFound
    }

    /// Get integration statistics
    pub fn get_integration_stats(&self) -> IntegrationStats {
        let queue_stats = self.queue.get_stats();
        let active_count = self.active_simulations.lock().unwrap().len();
        let subscription_count = self.progress_comm.subscription_count();

        IntegrationStats {
            queue_stats,
            active_simulations: active_count,
            max_concurrent_simulations: self.config.max_concurrent_simulations,
            progress_subscriptions: subscription_count,
            auto_store_enabled: self.config.auto_store_completed,
            auto_cleanup_enabled: self.config.auto_cleanup,
        }
    }

    /// Perform cleanup of old completed simulations
    pub fn perform_cleanup(&self) -> Result<CleanupResult, StorageIntegrationError> {
        let mut cleaned_simulations = 0;
        let mut cleaned_storage = false;

        // Clean up old active simulations (those that have been running too long)
        {
            let mut active = self.active_simulations.lock().unwrap();
            let now = Instant::now();
            
            active.retain(|_, info| {
                let age_seconds = now.duration_since(info.start_time).as_secs();
                // Remove simulations that have been running for more than 1 hour without progress updates
                let should_keep = age_seconds < 3600 || 
                    info.last_progress_update
                        .map(|last| now.duration_since(last).as_secs() < 300) // 5 minutes without updates
                        .unwrap_or(false);
                
                if !should_keep {
                    cleaned_simulations += 1;
                }
                
                should_keep
            });
        }

        // Clean up storage if enabled
        if self.config.auto_cleanup {
            if let Ok(mut storage_manager) = self.storage_manager.try_lock() {
                if storage_manager.cleanup().is_ok() {
                    cleaned_storage = true;
                }
            }
        }

        Ok(CleanupResult {
            cleaned_simulations,
            cleaned_storage,
        })
    }

    /// Get cached results if available
    pub fn get_cached_results(
        &self,
        players: &[Creature],
        encounters: &[Encounter],
        iterations: usize,
    ) -> Option<Vec<SimulationResult>> {
        let storage_manager = self.storage_manager.lock().unwrap();
        storage_manager.get_cached_results(players, encounters, iterations)
    }
}

/// Statistics about the integration system
#[derive(Debug, Clone)]
pub struct IntegrationStats {
    /// Queue statistics
    pub queue_stats: crate::queue_manager::QueueStats,
    /// Number of currently active simulations
    pub active_simulations: usize,
    /// Maximum concurrent simulations allowed
    pub max_concurrent_simulations: usize,
    /// Number of active progress subscriptions
    pub progress_subscriptions: usize,
    /// Whether auto-store is enabled
    pub auto_store_enabled: bool,
    /// Whether auto-cleanup is enabled
    pub auto_cleanup_enabled: bool,
}

/// Result of cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    /// Number of simulations cleaned up
    pub cleaned_simulations: usize,
    /// Whether storage was cleaned up
    pub cleaned_storage: bool,
}

/// Errors that can occur during storage integration
#[derive(Debug, Clone)]
pub enum StorageIntegrationError {
    /// Queue-related error
    QueueError(QueueError),
    /// Progress communication error
    ProgressError(ProgressError),
    /// Storage-related error
    StorageError(String),
    /// Request not found
    RequestNotFound,
    /// Too many concurrent simulations
    TooManyConcurrentSimulations,
    /// Invalid configuration
    InvalidConfiguration(String),
}

impl std::fmt::Display for StorageIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageIntegrationError::QueueError(e) => write!(f, "Queue error: {}", e),
            StorageIntegrationError::ProgressError(e) => write!(f, "Progress error: {}", e),
            StorageIntegrationError::StorageError(e) => write!(f, "Storage error: {}", e),
            StorageIntegrationError::RequestNotFound => write!(f, "Request not found"),
            StorageIntegrationError::TooManyConcurrentSimulations => write!(f, "Too many concurrent simulations"),
            StorageIntegrationError::InvalidConfiguration(e) => write!(f, "Invalid configuration: {}", e),
        }
    }
}

impl std::error::Error for StorageIntegrationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Creature, DiceFormula};

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

    fn create_test_parameters() -> ScenarioParameters {
        ScenarioParameters {
            players: vec![create_test_creature("Player1", 10.0, 15.0)],
            encounters: vec![],
            iterations: 100,
            config: Default::default(),
        }
    }

    #[test]
    fn test_storage_integration_creation() {
        let storage_manager = StorageManager::default();
        let queue = SimulationQueue::new(10);
        let (progress_comm, _) = ProgressCommunication::new();
        let config = StorageIntegrationConfig::default();

        let integration = StorageIntegration::new(
            storage_manager,
            queue,
            progress_comm,
            config,
        );

        let stats = integration.get_integration_stats();
        assert_eq!(stats.active_simulations, 0);
        assert_eq!(stats.max_concurrent_simulations, 3);
        assert!(stats.auto_store_enabled);
    }

    #[test]
    fn test_simulation_submission() {
        let storage_manager = StorageManager::default();
        let queue = SimulationQueue::new(10);
        let (progress_comm, _) = ProgressCommunication::new();
        let config = StorageIntegrationConfig::default();

        let integration = StorageIntegration::new(
            storage_manager,
            queue,
            progress_comm,
            config,
        );

        let parameters = create_test_parameters();
        let result = integration.submit_simulation_request(
            parameters,
            crate::background_simulation::SimulationPriority::Normal,
        ).unwrap();

        assert!(!result.request_id.is_empty());
        assert!(result.simulation_id.is_some()); // Should start immediately
        assert!(!result.queued);
        assert_eq!(result.estimated_wait_ms, Some(0));
    }

    #[test]
    fn test_request_status() {
        let storage_manager = StorageManager::default();
        let queue = SimulationQueue::new(10);
        let (progress_comm, _) = ProgressCommunication::new();
        let config = StorageIntegrationConfig::default();

        let integration = StorageIntegration::new(
            storage_manager,
            queue,
            progress_comm,
            config,
        );

        let parameters = create_test_parameters();
        let result = integration.submit_simulation_request(
            parameters,
            crate::background_simulation::SimulationPriority::Normal,
        ).unwrap();

        let status = integration.get_request_status(&result.request_id);
        assert!(matches!(status, SimulationRequestStatus::Running { .. }));
    }

    #[test]
    fn test_cleanup() {
        let storage_manager = StorageManager::default();
        let queue = SimulationQueue::new(10);
        let (progress_comm, _) = ProgressCommunication::new();
        let config = StorageIntegrationConfig::default();

        let integration = StorageIntegration::new(
            storage_manager,
            queue,
            progress_comm,
            config,
        );

        let cleanup_result = integration.perform_cleanup().unwrap();
        assert_eq!(cleanup_result.cleaned_simulations, 0);
    }
}