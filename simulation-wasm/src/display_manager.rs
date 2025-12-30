use crate::storage::{ScenarioParameters, SlotSelection};
use crate::storage_manager::StorageManager;
use crate::model::{Creature, Encounter, SimulationResult, TimelineStep};
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::sync::PoisonError;
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;
use web_sys::console;

/// Display modes for simulation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    /// Always show the newest simulation results
    ShowNewest,
    /// Show the most similar results to current parameters
    ShowMostSimilar,
    /// Let user choose between available slots
    LetUserChoose,
    /// Show results from primary slot only
    PrimaryOnly,
    /// Show results from secondary slot only
    SecondaryOnly,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayMode::ShowNewest => write!(f, "ShowNewest"),
            DisplayMode::ShowMostSimilar => write!(f, "ShowMostSimilar"),
            DisplayMode::LetUserChoose => write!(f, "LetUserChoose"),
            DisplayMode::PrimaryOnly => write!(f, "PrimaryOnly"),
            DisplayMode::SecondaryOnly => write!(f, "SecondaryOnly"),
        }
    }
}

impl Default for DisplayMode {
    fn default() -> Self {
        DisplayMode::ShowNewest
    }
}

/// Configuration for display behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Default display mode
    pub default_mode: DisplayMode,
    /// Whether to show parameter comparison dialogs
    pub show_comparison_dialogs: bool,
    /// Whether to auto-switch to most similar when parameters change
    pub auto_switch_similar: bool,
    /// Similarity threshold for auto-switching (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Whether to show progress indicators during background simulations
    pub show_progress_indicators: bool,
    /// Update interval for progress displays (milliseconds)
    pub progress_update_interval_ms: u64,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            default_mode: DisplayMode::ShowNewest,
            show_comparison_dialogs: true,
            auto_switch_similar: true,
            similarity_threshold: 0.8,
            show_progress_indicators: true,
            progress_update_interval_ms: 500,
        }
    }
}

/// Information about a slot for user selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    /// Which slot (primary or secondary)
    pub slot_selection: SlotSelection,
    /// Age of the data in seconds
    pub age_seconds: u64,
    /// Parameter similarity score (0.0 to 1.0)
    pub similarity_score: f64,
    /// Number of iterations
    pub iterations: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Status of the simulation
    pub status: String,
    /// Key parameter differences
    pub parameter_differences: Vec<ParameterDifference>,
}

/// Difference between parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDifference {
    /// Name of the parameter
    pub parameter_name: String,
    /// Value in current parameters
    pub current_value: String,
    /// Value in stored parameters
    pub stored_value: String,
    /// Severity of difference (0.0 to 1.0)
    pub severity: f64,
}

/// Result of a display operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayResult {
    /// The simulation results to display
    pub results: Option<Vec<SimulationResult>>,
    /// Which slot was used
    pub slot_used: Option<SlotSelection>,
    /// Display mode that was used
    pub mode_used: DisplayMode,
    /// Whether user interaction was required
    pub user_interaction_required: bool,
    /// Available slots for user selection (if applicable)
    pub available_slots: Vec<SlotInfo>,
    /// Messages for the user
    pub messages: Vec<String>,
}

/// Manager for displaying simulation results with various modes
pub struct DisplayManager {
    /// Storage manager for accessing simulation data
    _storage_manager: std::sync::Arc<std::sync::Mutex<StorageManager>>,
    /// Display configuration
    config: DisplayConfig,
    /// Current display mode
    current_mode: DisplayMode,
    /// Last parameters that were displayed
    last_parameters: Option<ScenarioParameters>,
    /// Progress communicator for background simulations
    progress_communicator: Option<std::sync::Arc<std::sync::Mutex<crate::progress_communication::ProgressCommunication>>>,
    /// Receiver for progress updates
    progress_receiver: Option<mpsc::Receiver<crate::progress_communication::ProgressUpdate>>,
    /// Last received progress update
    last_progress_update: Option<crate::progress_communication::ProgressUpdate>,
    /// Currently active slot for display
    active_slot: Option<SlotSelection>,
}

impl DisplayManager {
    /// Create a new display manager
    pub fn new(storage_manager: StorageManager, config: DisplayConfig) -> Self {
        Self {
            _storage_manager: std::sync::Arc::new(std::sync::Mutex::new(storage_manager)),
            config: config.clone(),
            current_mode: config.default_mode,
            last_parameters: None,
            progress_communicator: None,
            progress_receiver: None,
            last_progress_update: None,
            active_slot: None,
        }
    }

    /// Get results to display based on current mode and parameters
    pub fn get_display_results(
        &mut self,
        players: &[Creature],
        timeline: &[TimelineStep],
        iterations: usize,
    ) -> DisplayResult {
        let parameters = ScenarioParameters {
            players: players.to_vec(),
            timeline: timeline.to_vec(),
            iterations,
        };

        // Check if parameters have changed
        let parameters_changed = self.last_parameters.as_ref()
            .map(|last| !self.parameters_equal(&last, &parameters))
            .unwrap_or(true);

        // Auto-switch to most similar if enabled and parameters changed
        if parameters_changed && self.config.auto_switch_similar {
            if let Some(most_similar_slot) = self.find_most_similar_slot(&parameters) {
                if most_similar_slot.similarity_score >= self.config.similarity_threshold {
                    self.current_mode = DisplayMode::ShowMostSimilar;
                }
            }
        }

        // Store current parameters
        self.last_parameters = Some(parameters.clone());

        // Get results based on current mode
        match self.current_mode {
            DisplayMode::ShowNewest => self.get_newest_results(&parameters),
            DisplayMode::ShowMostSimilar => self.get_most_similar_results(&parameters),
            DisplayMode::LetUserChoose => self.present_user_choice(&parameters),
            DisplayMode::PrimaryOnly => self.get_slot_results(&parameters, SlotSelection::Primary),
            DisplayMode::SecondaryOnly => self.get_slot_results(&parameters, SlotSelection::Secondary),
        }
    }

    /// Get the newest results (most recent timestamp)
    fn get_newest_results(&self, _parameters: &ScenarioParameters) -> DisplayResult {
        // Since storage is removed, return empty results
        DisplayResult {
            results: None,
            slot_used: None,
            mode_used: DisplayMode::ShowNewest,
            user_interaction_required: false,
            available_slots: vec![],
            messages: vec!["Storage functionality removed - no cached results available".to_string()],
        }
    }

    /// Get the most similar results to current parameters
    fn get_most_similar_results(&self, _parameters: &ScenarioParameters) -> DisplayResult {
        // Since storage is removed, return empty results
        DisplayResult {
            results: None,
            slot_used: None,
            mode_used: DisplayMode::ShowMostSimilar,
            user_interaction_required: false,
            available_slots: vec![],
            messages: vec!["Storage functionality removed - no cached results available".to_string()],
        }
    }

    /// Present user choice between available slots
    fn present_user_choice(&self, _parameters: &ScenarioParameters) -> DisplayResult {
        // Since storage is removed, return empty results
        DisplayResult {
            results: None,
            slot_used: None,
            mode_used: DisplayMode::LetUserChoose,
            user_interaction_required: false,
            available_slots: vec![],
            messages: vec!["Storage functionality removed - no cached results available".to_string()],
        }
    }

    /// Get results from a specific slot
    fn get_slot_results(&self, _parameters: &ScenarioParameters, slot_selection: SlotSelection) -> DisplayResult {
        // Since storage is removed, return empty results
        DisplayResult {
            results: None,
            slot_used: Some(slot_selection.clone()),
            mode_used: if slot_selection == SlotSelection::Primary {
                DisplayMode::PrimaryOnly
            } else {
                DisplayMode::SecondaryOnly
            },
            user_interaction_required: false,
            available_slots: vec![],
            messages: vec![format!("Storage functionality removed - no data available in {:?} slot", slot_selection)],
        }
    }

    /// User selected a specific slot
    pub fn user_selected_slot(&mut self, slot_selection: SlotSelection) -> DisplayResult {
        if let Some(ref parameters) = self.last_parameters {
            self.current_mode = match slot_selection {
                SlotSelection::Primary => DisplayMode::PrimaryOnly,
                SlotSelection::Secondary => DisplayMode::SecondaryOnly,
            };
            self.get_slot_results(parameters, slot_selection)
        } else {
            DisplayResult {
                results: None,
                slot_used: None,
                mode_used: self.current_mode,
                user_interaction_required: false,
                available_slots: vec![],
                messages: vec!["No parameters available for slot selection".to_string()],
            }
        }
    }

    /// Switch display mode
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.current_mode = mode;
    }

    /// Get current display mode
    pub fn get_display_mode(&self) -> DisplayMode {
        self.current_mode
    }

    /// Update display configuration
    pub fn update_config(&mut self, config: DisplayConfig) {
        self.config = config;
        self.current_mode = self.config.default_mode;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &DisplayConfig {
        &self.config
    }

    /// Set progress communicator for background simulation updates
    pub fn set_progress_communicator(&mut self, communicator_arc: std::sync::Arc<std::sync::Mutex<crate::progress_communication::ProgressCommunication>>) {
        {
            let communicator = communicator_arc.lock().unwrap_or_else(PoisonError::into_inner);
            // Create a unique subscription ID for this display manager
            let subscription_id = format!("display_manager_sub_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
            let subscription = crate::progress_communication::ProgressSubscription::new(subscription_id);

            if let Ok(receiver) = communicator.subscribe(subscription) {
                self.progress_receiver = Some(receiver);
            } else {
                console::error_1(&"Failed to subscribe DisplayManager to ProgressCommunication".into());
            }
        }
        self.progress_communicator = Some(communicator_arc);
    }

    /// Get current progress from background simulations
    pub fn get_current_progress(&mut self) -> Option<crate::progress_communication::ProgressUpdate> {
        // Drain all available updates from the receiver and store the latest
        if let Some(receiver) = self.progress_receiver.as_ref() {
            while let Ok(update) = receiver.try_recv() {
                self.last_progress_update = Some(update);
            }
        }
        self.last_progress_update.clone()
    }

    /// Handle simulation completion
    pub fn handle_simulation_completed(&mut self, slot_selection: SlotSelection) -> Result<(), String> {
        console::log_1(&format!("Simulation completed for slot {:?}", slot_selection).into());
        
        if self.config.auto_switch_similar {
            self.active_slot = Some(slot_selection);
            self.current_mode = DisplayMode::ShowNewest;
        }

        Ok(())
    }

    /// Handle simulation failure
    pub fn handle_simulation_failed(&mut self, error: &str) -> Result<(), String> {
        console::log_1(&format!("Simulation failed: {}", error).into());
        Ok(())
    }

    /// Generate status text based on current state
    pub fn generate_status_text(&mut self) -> String {
        if let Some(progress) = self.get_current_progress() {
            match progress.update_type {
                crate::progress_communication::ProgressUpdateType::Started => "Starting".to_string(),
                crate::progress_communication::ProgressUpdateType::Progress => {
                    let percentage = (progress.progress_percentage * 100.0) as u32;
                    format!("Simulating: {}%", percentage)
                }
                crate::progress_communication::ProgressUpdateType::Completed => "Completed".to_string(),
                crate::progress_communication::ProgressUpdateType::Failed => "Failed".to_string(),
                crate::progress_communication::ProgressUpdateType::Cancelled => "Cancelled".to_string(),
            }
        } else {
            "Ready".to_string()
        }
    }

    /// Find the most similar slot to the given parameters
    fn find_most_similar_slot(&self, _parameters: &ScenarioParameters) -> Option<SlotInfo> {
        // Since storage is removed, return None
        None
    }

    /// Calculate similarity between two parameter sets (0.0 to 1.0)
    fn calculate_similarity(&self, params1: &ScenarioParameters, params2: &ScenarioParameters) -> f64 {
        let mut similarity = 1.0;
        let mut factors = 0;

        // Compare iterations
        if params1.iterations > 0 && params2.iterations > 0 {
            let iter_diff = (params1.iterations as f64 - params2.iterations as f64).abs();
            let iter_similarity = 1.0 - (iter_diff / params1.iterations.max(params2.iterations) as f64).min(1.0);
            similarity *= iter_similarity;
            factors += 1;
        }

        // Compare player count
        let player_count_diff = (params1.players.len() as f64 - params2.players.len() as f64).abs();
        let player_similarity = 1.0 - (player_count_diff / params1.players.len().max(params2.players.len()) as f64).min(1.0);
        similarity *= player_similarity;
        factors += 1;

        // Compare encounter count
        let encounter_count_diff = (params1.timeline.len() as f64 - params2.timeline.len() as f64).abs();
        let encounter_similarity = 1.0 - (encounter_count_diff / params1.timeline.len().max(params2.timeline.len()) as f64).min(1.0);
        similarity *= encounter_similarity;
        factors += 1;

        // Simple player comparison (could be enhanced)
        if !params1.players.is_empty() && !params2.players.is_empty() {
            let player_similarity = self.compare_creature_lists(&params1.players, &params2.players);
            similarity *= player_similarity;
            factors += 1;
        }

        // Simple encounter comparison (could be enhanced)
        if !params1.timeline.is_empty() && !params2.timeline.is_empty() {
            let encounter_similarity = self.compare_encounter_lists(&params1.timeline, &params2.timeline);
            similarity *= encounter_similarity;
            factors += 1;
        }

        if factors > 0 {
            similarity.powf(1.0 / factors as f64)
        } else {
            1.0
        }
    }

    /// Compare two creature lists
    fn compare_creature_lists(&self, list1: &[Creature], list2: &[Creature]) -> f64 {
        if list1.is_empty() && list2.is_empty() {
            return 1.0;
        }
        if list1.is_empty() || list2.is_empty() {
            return 0.0;
        }

        let mut total_similarity = 0.0;
        let max_len = list1.len().max(list2.len());

        for i in 0..max_len {
            let creature1 = list1.get(i);
            let creature2 = list2.get(i);

            match (creature1, creature2) {
                (Some(c1), Some(c2)) => {
                    let similarity = self.compare_creatures(c1, c2);
                    total_similarity += similarity;
                }
                _ => {
                    // One list is shorter
                    total_similarity += 0.5; // Partial credit
                }
            }
        }

        total_similarity / max_len as f64
    }

    /// Compare two creatures
    fn compare_creatures(&self, c1: &Creature, c2: &Creature) -> f64 {
        let mut similarity = 0.0;
        let mut factors = 0;

        // Name
        if c1.name == c2.name {
            similarity += 1.0;
        }
        factors += 1;

        // HP similarity
        let hp_diff = c1.hp.abs_diff(c2.hp) as f64;
        let hp_similarity = 1.0 - (hp_diff / (c1.hp.max(c2.hp) as f64)).min(1.0);
        similarity += hp_similarity;
        factors += 1;

        // AC similarity
        let ac_diff = c1.ac.abs_diff(c2.ac) as f64;
        let ac_similarity = 1.0 - (ac_diff / (c1.ac.max(c2.ac) as f64)).min(1.0);
        similarity += ac_similarity;
        factors += 1;

        // Count similarity
        let count_diff = (c1.count - c2.count).abs();
        let count_similarity = 1.0 - (count_diff / c1.count.max(c2.count)).min(1.0);
        similarity += count_similarity;
        factors += 1;

        if factors > 0 {
            similarity / factors as f64
        } else {
            1.0
        }
    }

    /// Compare two encounter lists
    fn compare_encounter_lists(&self, list1: &[TimelineStep], list2: &[TimelineStep]) -> f64 {
        if list1.is_empty() && list2.is_empty() {
            return 1.0;
        }
        if list1.is_empty() || list2.is_empty() {
            return 0.0;
        }

        let mut total_similarity = 0.0;
        let max_len = list1.len().max(list2.len());

        for i in 0..max_len {
            let step1 = list1.get(i);
            let step2 = list2.get(i);

            match (step1, step2) {
                (Some(TimelineStep::Combat(e1)), Some(TimelineStep::Combat(e2))) => {
                    let similarity = self.compare_encounters(e1, e2);
                    total_similarity += similarity;
                }
                (Some(TimelineStep::ShortRest(_)), Some(TimelineStep::ShortRest(_))) => {
                    total_similarity += 1.0;
                }
                (None, _) | (_, None) => {
                    total_similarity += 0.5;
                }
                _ => {
                    // Different types at same position
                    total_similarity += 0.0;
                }
            }
        }

        total_similarity / max_len as f64
    }

    /// Compare two encounters
    fn compare_encounters(&self, e1: &Encounter, e2: &Encounter) -> f64 {
        // Compare monster lists
        self.compare_creature_lists(&e1.monsters, &e2.monsters)
    }

    /// Calculate parameter differences for display
    #[allow(dead_code)]
    fn calculate_parameter_differences(&self, params1: &ScenarioParameters, params2: &ScenarioParameters) -> Vec<ParameterDifference> {
        let mut differences = Vec::new();

        // Iterations difference
        if params1.iterations != params2.iterations {
            differences.push(ParameterDifference {
                parameter_name: "Iterations".to_string(),
                current_value: params1.iterations.to_string(),
                stored_value: params2.iterations.to_string(),
                severity: if params1.iterations > 0 && params2.iterations > 0 {
                    ((params1.iterations as f64 - params2.iterations as f64).abs() / params1.iterations.max(params2.iterations) as f64).min(1.0)
                } else {
                    1.0
                },
            });
        }

        // Player count difference
        if params1.players.len() != params2.players.len() {
            differences.push(ParameterDifference {
                parameter_name: "Player Count".to_string(),
                current_value: params1.players.len().to_string(),
                stored_value: params2.players.len().to_string(),
                severity: ((params1.players.len() as f64 - params2.players.len() as f64).abs() / params1.players.len().max(params2.players.len()) as f64).min(1.0),
            });
        }

        // Encounter count difference
        if params1.timeline.len() != params2.timeline.len() {
            differences.push(ParameterDifference {
                parameter_name: "Encounter Count".to_string(),
                current_value: params1.timeline.len().to_string(),
                stored_value: params2.timeline.len().to_string(),
                severity: ((params1.timeline.len() as f64 - params2.timeline.len() as f64).abs() / params1.timeline.len().max(params2.timeline.len()) as f64).min(1.0),
            });
        }

        differences
    }

    /// Check if two parameter sets are effectively equal
    fn parameters_equal(&self, params1: &ScenarioParameters, params2: &ScenarioParameters) -> bool {
        params1.iterations == params2.iterations
            && params1.players.len() == params2.players.len()
            && params1.timeline.len() == params2.timeline.len()
            && self.calculate_similarity(params1, params2) > 0.95
    }
}

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
            hp: hp as u32,
            ac: ac as u32,
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
    fn test_display_mode_default() {
        let config = DisplayConfig::default();
        assert_eq!(config.default_mode, DisplayMode::ShowNewest);
    }

    #[test]
    fn test_creature_similarity() {
        let manager = DisplayManager::new(
            StorageManager::default(),
            DisplayConfig::default(),
        );

        let c1 = create_test_creature("Fighter", 50.0, 16.0);
        let c2 = create_test_creature("Fighter", 50.0, 16.0);
        let c3 = create_test_creature("Wizard", 30.0, 12.0);

        let sim_1_2 = manager.compare_creatures(&c1, &c2);
        let sim_1_3 = manager.compare_creatures(&c1, &c3);

        assert!(sim_1_2 > sim_1_3);
        assert!(sim_1_2 > 0.9); // Very similar
        assert!(sim_1_3 < 0.8); // Less similar
    }

    #[test]
    fn test_parameter_differences() {
        let manager = DisplayManager::new(
            StorageManager::default(),
            DisplayConfig::default(),
        );

        let params1 = ScenarioParameters {
            players: vec![create_test_creature("Fighter", 50.0, 16.0)],
            timeline: vec![],
            iterations: 100,
        };

        let params2 = ScenarioParameters {
            players: vec![create_test_creature("Fighter", 50.0, 16.0), create_test_creature("Cleric", 40.0, 14.0)],
            timeline: vec![],
            iterations: 200,
        };

        let differences = manager.calculate_parameter_differences(&params1, &params2);
        assert_eq!(differences.len(), 2); // Player count and iterations

        let iter_diff = differences.iter().find(|d| d.parameter_name == "Iterations").unwrap();
        assert_eq!(iter_diff.current_value, "100");
        assert_eq!(iter_diff.stored_value, "200");
    }
}

// WASM bindings for JavaScript integration
#[wasm_bindgen]
pub struct DisplayManagerWrapper {
    inner: std::sync::Arc<std::sync::Mutex<DisplayManager>>,
}

#[wasm_bindgen]
impl DisplayManagerWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<DisplayManagerWrapper, JsValue> {
        let storage_manager = StorageManager::default();
        let display_manager = DisplayManager::new(storage_manager, DisplayConfig::default());
        Ok(DisplayManagerWrapper { 
            inner: std::sync::Arc::new(std::sync::Mutex::new(display_manager))
        })
    }

    #[wasm_bindgen(js_name = setDisplayMode)]
    pub fn set_display_mode(&self, mode: String) -> Result<(), JsValue> {
        let display_mode = match mode.as_str() {
            "ShowNewest" => DisplayMode::ShowNewest,
            "ShowMostSimilar" => DisplayMode::ShowMostSimilar,
            "LetUserChoose" => DisplayMode::LetUserChoose,
            "PrimaryOnly" => DisplayMode::PrimaryOnly,
            "SecondaryOnly" => DisplayMode::SecondaryOnly,
            _ => return Err(JsValue::from_str("Invalid display mode")),
        };
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .set_display_mode(display_mode);
        Ok(())
    }

    #[wasm_bindgen(js_name = getDisplayResults)]
    pub fn get_display_results(
        &self,
        players: &JsValue,
        timeline: &JsValue,
        iterations: usize,
    ) -> Result<JsValue, JsValue> {
        let players: Vec<Creature> = serde_wasm_bindgen::from_value(players.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
        let timeline: Vec<TimelineStep> = serde_wasm_bindgen::from_value(timeline.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse timeline: {}", e)))?;

        let result = self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_display_results(&players, &timeline, iterations);
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = userSelectedSlot)]
    pub fn user_selected_slot(&self, slot: String) -> Result<JsValue, JsValue> {
        let slot_selection = match slot.as_str() {
            "Primary" => SlotSelection::Primary,
            "Secondary" => SlotSelection::Secondary,
            _ => return Err(JsValue::from_str("Invalid slot selection")),
        };

        let result = self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .user_selected_slot(slot_selection);
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = getDisplayMode)]
    pub fn get_display_mode(&self) -> String {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner).get_display_mode().to_string()
    }

    #[wasm_bindgen(js_name = getStatusText)]
    pub fn get_status_text(&self) -> String {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner).generate_status_text()
    }

    #[wasm_bindgen(js_name = handleSimulationCompleted)]
    pub fn handle_simulation_completed(&self, slot: String) -> Result<(), JsValue> {
        let slot_selection = match slot.as_str() {
            "Primary" => SlotSelection::Primary,
            "Secondary" => SlotSelection::Secondary,
            _ => return Err(JsValue::from_str("Invalid slot selection")),
        };

        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .handle_simulation_completed(slot_selection)
            .map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen(js_name = handleSimulationFailed)]
    pub fn handle_simulation_failed(&self, error: String) -> Result<(), JsValue> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .handle_simulation_failed(&error)
            .map_err(|e| JsValue::from_str(&e))
    }
}