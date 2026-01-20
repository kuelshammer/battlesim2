//! GUI Integration - manages display, progress, and user interaction
//!
//! This module handles all GUI-related orchestration including:
//! - Display management (showing results)
//! - Progress UI management
//! - User interaction handling

use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use crate::display_manager::{DisplayManager, DisplayMode, DisplayConfig};
use crate::progress_ui::{ProgressUIManager, ProgressUIConfig};
use crate::user_interaction::{UserInteractionManager, UserEvent, UserInteractionConfig};
use crate::queue_manager::{QueueManager, QueueManagerConfig};
use crate::model::{Creature, TimelineStep};
use crate::user_interaction::SlotSelection;
#[cfg(not(target_arch = "wasm32"))]
use crate::background_simulation::BackgroundSimulationEngine;
use crate::background_simulation::{BackgroundSimulationId, SimulationPriority};

/// Combined GUI integration system
pub struct GuiIntegration {
    pub display_manager: Arc<Mutex<DisplayManager>>,
    pub progress_ui_manager: Arc<Mutex<ProgressUIManager>>,
    pub user_interaction_manager: Arc<Mutex<UserInteractionManager>>,
}

/// Global GUI integration singleton
static GUI_INTEGRATION: OnceLock<Mutex<GuiIntegration>> = OnceLock::new();

/// Initialize the GUI integration system
pub fn initialize_gui() -> Result<(), String> {
    GUI_INTEGRATION.get_or_init(|| {
        // Create display manager
        let display_config = DisplayConfig::default();
        let display_manager = DisplayManager::new(display_config);
        let display_manager_arc = Arc::new(Mutex::new(display_manager));

        // Create queue manager
        let queue_config = QueueManagerConfig::default();
        let queue_manager = QueueManager::new(queue_config);
        let queue_manager_arc = Arc::new(Mutex::new(queue_manager));

        // Create progress UI manager
        let progress_ui_manager = ProgressUIManager::new(ProgressUIConfig::default());
        let progress_ui_manager_arc = Arc::new(Mutex::new(progress_ui_manager));

        // Create user interaction manager (conditional compilation)
        let interaction_config = UserInteractionConfig::default();
        
        #[cfg(not(target_arch = "wasm32"))]
        let user_interaction_manager = {
            let (simulation_engine, _progress_receiver) = BackgroundSimulationEngine::new();
            let simulation_engine_arc = Arc::new(Mutex::new(simulation_engine));

            UserInteractionManager::new_with_simulation(
                display_manager_arc.clone(),
                progress_ui_manager_arc.clone(),
                simulation_engine_arc,
                queue_manager_arc.clone(),
                interaction_config,
            )
        };
        
        #[cfg(target_arch = "wasm32")]
        let user_interaction_manager = {
            UserInteractionManager::new(
                display_manager_arc.clone(),
                progress_ui_manager_arc.clone(),
                queue_manager_arc.clone(),
                interaction_config,
            )
        };

        Mutex::new(GuiIntegration {
            display_manager: display_manager_arc,
            progress_ui_manager: progress_ui_manager_arc,
            user_interaction_manager: Arc::new(Mutex::new(user_interaction_manager)),
        })
    });

    Ok(())
}

/// Get the GUI integration system (panics if not initialized)
pub fn get_gui() -> &'static Mutex<GuiIntegration> {
    GUI_INTEGRATION.get().expect("GUI Integration not initialized. Call initialize_gui() first.")
}

/// Get display results for current parameters
pub fn get_display_results(
    players: &[Creature],
    timeline: &[TimelineStep],
    iterations: usize,
) -> crate::display_manager::DisplayResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    display_manager.get_display_results(players, timeline, iterations)
}

/// Set display mode
pub fn set_display_mode(mode: DisplayMode) {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    display_manager.set_display_mode(mode);
}

/// Get current display mode
pub fn get_current_display_mode() -> DisplayMode {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    display_manager.get_display_mode()
}

/// User selected a specific slot
pub fn user_selected_slot(slot: SlotSelection) -> crate::display_manager::DisplayResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
    display_manager.user_selected_slot(slot)
}

/// Start a background simulation
pub fn start_background_simulation(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
    priority: SimulationPriority,
) -> crate::user_interaction::UserEventResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    
    let event = UserEvent::RequestSimulation {
        parameters: crate::user_interaction::ScenarioParameters {
            players,
            timeline,
            iterations,
        },
        priority,
    };

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.handle_event(event)
}

/// Get progress for all active simulations
pub fn get_all_progress() -> Vec<crate::progress_ui::ProgressInfo> {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    progress_ui.get_all_progress()
}

/// Get progress for a specific simulation
pub fn get_simulation_progress(simulation_id: &BackgroundSimulationId) -> Option<crate::progress_ui::ProgressInfo> {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    progress_ui.get_progress(simulation_id)
}

/// Create HTML progress bar for a simulation
pub fn create_progress_bar_html(simulation_id: &BackgroundSimulationId) -> String {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    
    match progress_ui.get_progress(simulation_id) {
        Some(info) => progress_ui.create_progress_bar_html(&info),
        None => String::new(),
    }
}

/// Create compact progress indicator
pub fn create_compact_indicator(simulation_id: &BackgroundSimulationId) -> String {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    
    match progress_ui.get_progress(simulation_id) {
        Some(info) => progress_ui.create_compact_indicator(&info),
        None => String::new(),
    }
}

/// Cancel a running simulation
pub fn cancel_simulation(simulation_id: BackgroundSimulationId) -> crate::user_interaction::UserEventResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let event = UserEvent::CancelSimulation { simulation_id };
    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.handle_event(event)
}

/// Clear simulation cache
pub fn clear_cache() -> crate::user_interaction::UserEventResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let event = UserEvent::ClearCache;
    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.handle_event(event)
}

/// Get pending user confirmations
pub fn get_pending_confirmations() -> Vec<crate::user_interaction::ConfirmationRequest> {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.get_pending_confirmations()
}

/// Answer a confirmation request
pub fn answer_confirmation(confirmation_id: &str, confirmed: bool) -> crate::user_interaction::UserEventResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.answer_confirmation(confirmation_id, confirmed)
}

/// Get current user interaction state
pub fn get_interaction_state() -> crate::user_interaction::UserInteractionState {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.get_state()
}

/// Update GUI configuration
pub fn update_configuration(
    display_config: Option<DisplayConfig>,
    progress_config: Option<ProgressUIConfig>,
    interaction_config: Option<UserInteractionConfig>,
) {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);

    if let Some(config) = display_config {
        let mut display_manager = gui.display_manager.lock().unwrap_or_else(PoisonError::into_inner);
        display_manager.update_config(config);
    }

    if let Some(config) = progress_config {
        let mut progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
        progress_ui.update_config(config);
    }

    if let Some(config) = interaction_config {
        let mut user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
        user_interaction.update_config(config);
    }
}

/// Get progress summary for dashboard
pub fn get_progress_summary() -> crate::progress_ui::ProgressSummary {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    let progress_ui = gui.progress_ui_manager.lock().unwrap_or_else(PoisonError::into_inner);
    progress_ui.get_progress_summary()
}

/// Handle parameter change event
pub fn handle_parameters_changed(
    players: Vec<Creature>,
    timeline: Vec<TimelineStep>,
    iterations: usize,
) -> crate::user_interaction::UserEventResult {
    let gui = get_gui().lock().unwrap_or_else(PoisonError::into_inner);
    
    let event = UserEvent::ParametersChanged {
        parameters: crate::user_interaction::ScenarioParameters {
            players,
            timeline,
            iterations,
        },
    };

    let user_interaction = gui.user_interaction_manager.lock().unwrap_or_else(PoisonError::into_inner);
    user_interaction.handle_event(event)
}
