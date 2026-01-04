use crate::display_manager::{DisplayManager, DisplayMode, DisplayResult};
use crate::background_simulation::{BackgroundSimulationId, SimulationPriority};
#[cfg(not(target_arch = "wasm32"))]
use crate::background_simulation::BackgroundSimulationEngine;
use crate::progress_ui::{ProgressUIManager, ProgressInfo};
use crate::queue_manager::QueueManager;
use crate::model::{Creature, TimelineStep};
// Simple scenario parameters since storage module was removed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioParameters {
    pub players: Vec<Creature>,
    pub timeline: Vec<TimelineStep>,
    pub iterations: usize,
}
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

/// User interaction events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    /// Parameters were changed
    ParametersChanged {
        parameters: ScenarioParameters,
    },
    /// User requested a new simulation
    RequestSimulation {
        parameters: ScenarioParameters,
        priority: SimulationPriority,
    },
    /// User selected a display mode
    SetDisplayMode(DisplayMode),
    /// User selected a specific slot
    SelectSlot {
        slot_selection: crate::storage::SlotSelection,
    },
    /// User cancelled a simulation
    CancelSimulation {
        simulation_id: BackgroundSimulationId,
    },
    /// User requested cache clear
    ClearCache,
    /// User requested storage cleanup
    CleanupStorage,
    /// User changed configuration
    UpdateConfiguration {
        display_config: Option<crate::display_manager::DisplayConfig>,
        progress_config: Option<crate::progress_ui::ProgressUIConfig>,
        queue_config: Option<crate::queue_manager::QueueManagerConfig>,
    },
}

/// Result of handling a user event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEventResult {
    /// Whether the event was handled successfully
    pub success: bool,
    /// Any messages for the user
    pub messages: Vec<String>,
    /// Display result if applicable
    pub display_result: Option<DisplayResult>,
    /// Progress info if applicable
    pub progress_info: Option<ProgressInfo>,
    /// Simulation ID if a new simulation was started
    pub simulation_id: Option<BackgroundSimulationId>,
    /// Whether the UI should be refreshed
    pub requires_ui_refresh: bool,
}

/// Configuration for user interaction handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteractionConfig {
    /// Whether to automatically start simulations when parameters change
    pub auto_simulate_on_change: bool,
    /// Whether to show confirmation dialogs for destructive actions
    pub show_confirmations: bool,
    /// Maximum number of concurrent simulations
    pub max_concurrent_simulations: usize,
    /// Whether to show detailed error messages
    pub show_detailed_errors: bool,
    /// Whether to animate UI transitions
    pub animate_transitions: bool,
    /// Timeout for user interactions (milliseconds)
    pub interaction_timeout_ms: u64,
}

impl Default for UserInteractionConfig {
    fn default() -> Self {
        Self {
            auto_simulate_on_change: true,
            show_confirmations: true,
            max_concurrent_simulations: 3,
            show_detailed_errors: true,
            animate_transitions: true,
            interaction_timeout_ms: 5000,
        }
    }
}

/// State for user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteractionState {
    /// Current parameters
    pub current_parameters: Option<ScenarioParameters>,
    /// Last time parameters were changed
    pub last_parameter_change: u64,
    /// Pending user confirmations
    pub pending_confirmations: Vec<ConfirmationRequest>,
    /// Active simulations
    pub active_simulations: Vec<BackgroundSimulationId>,
    /// User preferences
    pub user_preferences: UserPreferences,
}

/// A confirmation request for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    /// Unique identifier
    pub id: String,
    /// Type of confirmation
    pub confirmation_type: ConfirmationType,
    /// Message to show the user
    pub message: String,
    /// Detailed description
    pub description: String,
    /// When this request was created
    pub created_at: u64,
    /// Whether this request has been answered
    pub answered: bool,
}

/// Types of confirmations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationType {
    /// Clear cache confirmation
    ClearCache,
    /// Cancel simulation confirmation
    CancelSimulation,
    /// Switch slot confirmation
    SwitchSlot,
    /// Cleanup storage confirmation
    CleanupStorage,
    /// Custom confirmation
    Custom(String),
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred display mode
    pub preferred_display_mode: DisplayMode,
    /// Whether to show progress indicators
    pub show_progress_indicators: bool,
    /// Whether to auto-switch to similar results
    pub auto_switch_similar: bool,
    /// Theme preference
    pub theme: String,
    /// Language preference
    pub language: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_display_mode: DisplayMode::ShowNewest,
            show_progress_indicators: true,
            auto_switch_similar: true,
            theme: "default".to_string(),
            language: "en".to_string(),
        }
    }
}

/// Manager for user interactions
pub struct UserInteractionManager {
    /// Display manager
    display_manager: Arc<Mutex<DisplayManager>>,
    /// Progress UI manager
    progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
    /// Queue manager
    queue_manager: Arc<Mutex<QueueManager>>,
    /// Configuration
    config: UserInteractionConfig,
    /// Current state
    state: Arc<Mutex<UserInteractionState>>,
    /// Event history
    event_history: Arc<Mutex<Vec<UserEvent>>>,
    #[cfg(not(target_arch = "wasm32"))]
    /// Background simulation engine
    #[allow(dead_code)]
    simulation_engine: Arc<Mutex<BackgroundSimulationEngine>>,
}

impl UserInteractionManager {
    /// Create a new user interaction manager
    pub fn new(
        display_manager: Arc<Mutex<DisplayManager>>,
        progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
        queue_manager: Arc<Mutex<QueueManager>>,
        config: UserInteractionConfig,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Suppress unused warnings for native builds where these aren't used
            let _ = display_manager;
            let _ = progress_ui_manager;
            let _ = queue_manager;
            let _ = config;
            // For native builds, we need to provide a dummy simulation_engine
            // This constructor shouldn't be used in native code - use new_with_simulation instead
            // But we need to provide something valid to compile
            panic!("Use new_with_simulation() for native builds");
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self {
                display_manager,
                progress_ui_manager,
                queue_manager,
                config,
                state: Arc::new(Mutex::new(UserInteractionState {
                    current_parameters: None,
                    last_parameter_change: 0,
                    pending_confirmations: Vec::new(),
                    active_simulations: Vec::new(),
                    user_preferences: UserPreferences::default(),
                })),
                event_history: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Create a new user interaction manager with background simulation support
    pub fn new_with_simulation(
        display_manager: Arc<Mutex<DisplayManager>>,
        progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
        simulation_engine: Arc<Mutex<BackgroundSimulationEngine>>,
        queue_manager: Arc<Mutex<QueueManager>>,
        config: UserInteractionConfig,
    ) -> Self {
        Self {
            display_manager,
            progress_ui_manager,
            queue_manager,
            config,
            state: Arc::new(Mutex::new(UserInteractionState {
                current_parameters: None,
                last_parameter_change: 0,
                pending_confirmations: Vec::new(),
                active_simulations: Vec::new(),
                user_preferences: UserPreferences::default(),
            })),
            event_history: Arc::new(Mutex::new(Vec::new())),
            simulation_engine,
        }
    }

    /// Handle a user event
    pub fn handle_event(&self, event: UserEvent) -> UserEventResult {
        // Store event in history
        {
            let mut history = self.event_history.lock().unwrap_or_else(PoisonError::into_inner);
            history.push(event.clone());
            // Keep only last 100 events
            if history.len() > 100 {
                history.remove(0);
            }
        }

        match event {
            UserEvent::ParametersChanged { parameters } => {
                self.handle_parameters_changed(parameters)
            },
            UserEvent::RequestSimulation { parameters, priority } => {
                self.handle_simulation_request(parameters, priority)
            },
            UserEvent::SetDisplayMode(mode) => {
                self.handle_set_display_mode(mode)
            },
            UserEvent::SelectSlot { slot_selection } => {
                self.handle_select_slot(slot_selection)
            },
            UserEvent::CancelSimulation { simulation_id } => {
                self.handle_cancel_simulation(simulation_id)
            },
            UserEvent::ClearCache => {
                self.handle_clear_cache()
            },
            UserEvent::CleanupStorage => {
                self.handle_cleanup_storage()
            },
            UserEvent::UpdateConfiguration { display_config, progress_config, queue_config } => {
                self.handle_update_configuration(display_config, progress_config, queue_config)
            },
        }
    }

    /// Handle parameter changes
    fn handle_parameters_changed(
        &self,
        parameters: ScenarioParameters,
    ) -> UserEventResult {
        let mut messages = Vec::new();
        let requires_ui_refresh = true;

        // Update state
        {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.current_parameters = Some(parameters.clone());
            state.last_parameter_change = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }

        // Get display results
        let display_result = {
            let mut display_manager = self.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
            display_manager.get_display_results(&parameters.players, &parameters.timeline, parameters.iterations)
        };


        // Auto-simulate if enabled and no cached results
        let simulation_id = if self.config.auto_simulate_on_change && display_result.results.is_none() {
            // Check if we're under the concurrent limit
            let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.active_simulations.len() < self.config.max_concurrent_simulations {
                drop(state);

                #[cfg(not(target_arch = "wasm32"))]
                let sim_result = self.start_background_simulation(&parameters, SimulationPriority::Normal);
                #[cfg(target_arch = "wasm32")]
                let sim_result: Result<BackgroundSimulationId, String> = Err("Background simulation not available in WASM".to_string());

                match sim_result {
                    Ok(sim_id) => {
                        messages.push("Background simulation started".to_string());
                        Some(sim_id)
                    },
                    Err(e) => {
                        messages.push(format!("Failed to start simulation: {}", e));
                        None
                    }
                }
            } else {
                messages.push("Maximum concurrent simulations reached".to_string());
                None
            }
        } else {
            None
        };

        UserEventResult {
            success: true,
            messages,
            display_result: Some(display_result),
            progress_info: None,
            simulation_id,
            requires_ui_refresh,
        }
    }

    /// Handle simulation request
    fn handle_simulation_request(
        &self,
        _parameters: ScenarioParameters,
        priority: SimulationPriority,
    ) -> UserEventResult {
        let mut messages = Vec::new();

        // Check concurrent limit
        {
            let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.active_simulations.len() >= self.config.max_concurrent_simulations {
                return UserEventResult {
                    success: false,
                    messages: vec!["Maximum concurrent simulations reached".to_string()],
                    display_result: None,
                    progress_info: None,
                    simulation_id: None,
                    requires_ui_refresh: false,
                };
            }
        }

        // Start simulation
        #[cfg(not(target_arch = "wasm32"))]
        let sim_result = self.start_background_simulation(&_parameters, priority);
        #[cfg(target_arch = "wasm32")]
        let sim_result: Result<BackgroundSimulationId, String> = Err("Background simulation not available in WASM".to_string());

        match sim_result {
            Ok(simulation_id) => {
                messages.push(format!("Simulation {} started with priority {:?}", simulation_id.0, priority));

                // Get progress info
                let progress_info = {
                    let progress_ui = self.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
                    progress_ui.get_progress(&simulation_id)
                };

                UserEventResult {
                    success: true,
                    messages,
                    display_result: None,
                    progress_info,
                    simulation_id: Some(simulation_id),
                    requires_ui_refresh: true,
                }
            },
            Err(e) => {
                let error_msg = if self.config.show_detailed_errors {
                    format!("Failed to start simulation: {}", e)
                } else {
                    "Failed to start simulation".to_string()
                };

                UserEventResult {
                    success: false,
                    messages: vec![error_msg],
                    display_result: None,
                    progress_info: None,
                    simulation_id: None,
                    requires_ui_refresh: false,
                }
            }
        }
    }

    /// Handle display mode change
    fn handle_set_display_mode(&self, mode: DisplayMode) -> UserEventResult {
        let mut messages = Vec::new();

        {
            let mut display_manager = self.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
            display_manager.set_display_mode(mode);
        }

        messages.push(format!("Display mode changed to {:?}", mode));

        // Update user preferences
        {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.user_preferences.preferred_display_mode = mode;
        }

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Handle slot selection
    fn handle_select_slot(&self, slot_selection: crate::storage::SlotSelection) -> UserEventResult {
        let mut messages = Vec::new();

        let display_result = {
            let mut display_manager = self.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
            display_manager.user_selected_slot(slot_selection.clone())
        };

        messages.push(format!("Selected {:?} slot", slot_selection));

        UserEventResult {
            success: true,
            messages,
            display_result: Some(display_result),
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Handle simulation cancellation
    fn handle_cancel_simulation(&self, simulation_id: BackgroundSimulationId) -> UserEventResult {
        let mut messages = Vec::new();

        // Remove from active simulations
        {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.active_simulations.retain(|id| *id != simulation_id);
        }

        // Stop progress tracking
        {
            let progress_ui = self.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
            progress_ui.stop_tracking(&simulation_id);
        }

        // Note: In a real implementation, you'd also cancel the actual simulation
        // This would involve communicating with the background simulation engine

        messages.push(format!("Simulation {} cancelled", simulation_id.0));

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Handle cache clear
    fn handle_clear_cache(&self) -> UserEventResult {
        let mut messages = Vec::new();

        if self.config.show_confirmations {
            // Add confirmation request
            let confirmation = ConfirmationRequest {
                id: format!("clear_cache_{}", SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()),
                confirmation_type: ConfirmationType::ClearCache,
                message: "Clear all cached simulation results?".to_string(),
                description: "This will remove all stored simulation data from memory and disk.".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                answered: false,
            };

            {
                let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
                state.pending_confirmations.push(confirmation);
            }

            messages.push("Confirmation required to clear cache".to_string());
        } else {
            // Clear cache directly
            match self.clear_cache_internal() {
                Ok(_) => {
                    messages.push("Cache cleared successfully".to_string());
                },
                Err(e) => {
                    messages.push(format!("Failed to clear cache: {}", e));
                }
            }
        }

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Handle storage cleanup
    fn handle_cleanup_storage(&self) -> UserEventResult {
        let mut messages = Vec::new();

        if self.config.show_confirmations {
            // Add confirmation request
            let confirmation = ConfirmationRequest {
                id: format!("cleanup_storage_{}", SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()),
                confirmation_type: ConfirmationType::CleanupStorage,
                message: "Clean up old storage files?".to_string(),
                description: "This will remove old simulation data files to free up disk space.".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                answered: false,
            };

            {
                let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
                state.pending_confirmations.push(confirmation);
            }

            messages.push("Confirmation required to cleanup storage".to_string());
        } else {
            // Cleanup directly
            match self.cleanup_storage_internal() {
                Ok(_) => {
                    messages.push("Storage cleanup completed".to_string());
                },
                Err(e) => {
                    messages.push(format!("Failed to cleanup storage: {}", e));
                }
            }
        }

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Handle configuration update
    fn handle_update_configuration(
        &self,
        display_config: Option<crate::display_manager::DisplayConfig>,
        progress_config: Option<crate::progress_ui::ProgressUIConfig>,
        queue_config: Option<crate::queue_manager::QueueManagerConfig>,
    ) -> UserEventResult {
        let mut messages = Vec::new();

        if let Some(config) = display_config {
            {
                let mut display_manager = self.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
                display_manager.update_config(config.clone());
            }
            messages.push("Display configuration updated".to_string());
        }

        if let Some(config) = progress_config {
            {
                let mut progress_ui = self.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
                progress_ui.update_config(config);
            }
            messages.push("Progress UI configuration updated".to_string());
        }

        if let Some(config) = queue_config {
            {
                let mut queue_manager = self.queue_manager.lock().unwrap_or_else(PoisonError::into_inner);
                queue_manager.update_config(config);
            }
            messages.push("Queue configuration updated".to_string());
        }

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh: true,
        }
    }

    /// Start a background simulation (not available in WASM)
    #[cfg(not(target_arch = "wasm32"))]
    fn start_background_simulation(
        &self,
        parameters: &ScenarioParameters,
        priority: SimulationPriority,
    ) -> Result<BackgroundSimulationId, String> {

        // Start simulation directly
        let mut engine = self.simulation_engine.lock().unwrap_or_else(PoisonError::into_inner);
        let simulation_id = engine.start_simulation(
            parameters.clone(),
            priority,
        )?;

        // Add to active simulations
        {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.active_simulations.push(simulation_id.clone());
        }

        // Start progress tracking
        {
            let progress_ui = self.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
            progress_ui.start_tracking(simulation_id.clone());
        }

        Ok(simulation_id)
    }

    /// Clear cache internally
    fn clear_cache_internal(&self) -> Result<(), String> {
        // Since storage functionality is removed, just clear progress tracking
        {
            let progress_ui = self.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
            progress_ui.clear_all();
        }

        // Clear the simulation results cache as well
        crate::cache::clear_cache();

        Ok(())
    }

    /// Cleanup storage internally (stub since storage is removed)
    fn cleanup_storage_internal(&self) -> Result<(), String> {
        // Since storage functionality is removed, this is a no-op
        Ok(())
    }

    
    /// Answer a confirmation request
    pub fn answer_confirmation(&self, confirmation_id: &str, confirmed: bool) -> UserEventResult {
        let mut messages = Vec::new();
        let mut requires_ui_refresh = false;

        // Find and remove the confirmation
        let confirmation = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            state.pending_confirmations
                .iter()
                .position(|c| c.id == confirmation_id)
                .map(|i| state.pending_confirmations.remove(i))
        };

        if let Some(confirmation) = confirmation {
            if confirmed {
                match confirmation.confirmation_type {
                    ConfirmationType::ClearCache => {
                        match self.clear_cache_internal() {
                            Ok(_) => {
                                messages.push("Cache cleared successfully".to_string());
                                requires_ui_refresh = true;
                            },
                            Err(e) => {
                                messages.push(format!("Failed to clear cache: {}", e));
                            }
                        }
                    },
                    ConfirmationType::CleanupStorage => {
                        match self.cleanup_storage_internal() {
                            Ok(_) => {
                                messages.push("Storage cleanup completed".to_string());
                                requires_ui_refresh = true;
                            },
                            Err(e) => {
                                messages.push(format!("Failed to cleanup storage: {}", e));
                            }
                        }
                    },
                    ConfirmationType::CancelSimulation => {
                        // This would be handled by the specific simulation cancellation
                        messages.push("Simulation cancellation confirmed".to_string());
                        requires_ui_refresh = true;
                    },
                    ConfirmationType::SwitchSlot => {
                        // This would be handled by the specific slot selection
                        messages.push("Slot switch confirmed".to_string());
                        requires_ui_refresh = true;
                    },
                    ConfirmationType::Custom(_) => {
                        messages.push("Custom action confirmed".to_string());
                        requires_ui_refresh = true;
                    },
                }
            } else {
                messages.push("Action cancelled".to_string());
            }
        } else {
            messages.push("Confirmation request not found".to_string());
        }

        UserEventResult {
            success: true,
            messages,
            display_result: None,
            progress_info: None,
            simulation_id: None,
            requires_ui_refresh,
        }
    }

    /// Get current state
    pub fn get_state(&self) -> UserInteractionState {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Get pending confirmations
    pub fn get_pending_confirmations(&self) -> Vec<ConfirmationRequest> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).pending_confirmations.clone()
    }

    /// Get event history
    pub fn get_event_history(&self) -> Vec<UserEvent> {
        self.event_history.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: UserInteractionConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &UserInteractionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_interaction_config_default() {
        let config = UserInteractionConfig::default();
        assert!(config.auto_simulate_on_change);
        assert!(config.show_confirmations);
        assert_eq!(config.max_concurrent_simulations, 3);
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.preferred_display_mode, DisplayMode::ShowNewest);
        assert!(prefs.show_progress_indicators);
        assert!(prefs.auto_switch_similar);
    }

    #[test]
    fn test_confirmation_request_creation() {
        let confirmation = ConfirmationRequest {
            id: "test".to_string(),
            confirmation_type: ConfirmationType::ClearCache,
            message: "Clear cache?".to_string(),
            description: "This will clear all cache".to_string(),
            created_at: 1234567890,
            answered: false,
        };

        assert_eq!(confirmation.id, "test");
        assert!(!confirmation.answered);
        assert!(matches!(confirmation.confirmation_type, ConfirmationType::ClearCache));
    }

    #[test]
    fn test_user_event_serialization() {
        let event = UserEvent::SetDisplayMode(DisplayMode::ShowMostSimilar);
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: UserEvent = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized, UserEvent::SetDisplayMode(DisplayMode::ShowMostSimilar)));
    }
}