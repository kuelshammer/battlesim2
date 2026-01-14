use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// User preferences for GUI behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    /// Display mode preferences
    pub display: DisplayPreferences,
    /// Progress UI preferences
    pub progress: ProgressPreferences,
    /// Storage preferences
    pub storage: StoragePreferences,
    /// Background processing preferences
    pub background: BackgroundPreferences,
    /// General UI preferences
    pub ui: UIPreferences,
}

/// Display-related preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayPreferences {
    /// Default display mode
    pub default_mode: String,
    /// Whether to show parameter comparison dialogs
    pub show_comparison_dialogs: bool,
    /// Whether to auto-switch to most similar when parameters change
    pub auto_switch_similar: bool,
    /// Similarity threshold for auto-switching (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum number of slots to display
    pub max_slots_displayed: usize,
    /// Whether to show slot metadata
    pub show_slot_metadata: bool,
    /// Whether to show execution times
    pub show_execution_times: bool,
}

impl Default for DisplayPreferences {
    fn default() -> Self {
        Self {
            default_mode: "ShowNewest".to_string(),
            show_comparison_dialogs: true,
            auto_switch_similar: true,
            similarity_threshold: 0.8,
            max_slots_displayed: 10,
            show_slot_metadata: true,
            show_execution_times: true,
        }
    }
}

/// Progress UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressPreferences {
    /// Whether to show progress indicators
    pub show_progress_indicators: bool,
    /// Update interval for progress displays (milliseconds)
    pub progress_update_interval_ms: u64,
    /// Number of segments in progress bar
    pub progress_segments: usize,
    /// Whether to show detailed progress
    pub show_detailed: bool,
    /// Whether to show time estimates
    pub show_time_estimates: bool,
    /// Whether to animate progress
    pub animate_progress: bool,
    /// Color scheme for progress
    pub color_scheme: ProgressColorScheme,
}

impl Default for ProgressPreferences {
    fn default() -> Self {
        Self {
            show_progress_indicators: true,
            progress_update_interval_ms: 500,
            progress_segments: 20,
            show_detailed: true,
            show_time_estimates: true,
            animate_progress: true,
            color_scheme: ProgressColorScheme::default(),
        }
    }
}

/// Color scheme for progress display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressColorScheme {
    /// Color for normal progress
    pub normal_color: String,
    /// Color for active progress
    pub active_color: String,
    /// Color for completed progress
    pub success_color: String,
    /// Color for warnings
    pub warning_color: String,
    /// Color for errors
    pub error_color: String,
    /// Background color
    pub background_color: String,
}

impl Default for ProgressColorScheme {
    fn default() -> Self {
        Self {
            normal_color: "#3b82f6".to_string(), // blue-500
            active_color: "#10b981".to_string(), // emerald-500
            success_color: "#22c55e".to_string(), // green-500
            warning_color: "#f59e0b".to_string(), // amber-500
            error_color: "#ef4444".to_string(), // red-500
            background_color: "#e5e7eb".to_string(), // gray-200
        }
    }
}

/// Storage-related preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePreferences {
    /// Maximum number of slots to keep in memory
    pub max_memory_slots: usize,
    /// Maximum number of slots to keep in persistent storage
    pub max_persistent_slots: usize,
    /// Cleanup policy for old slots
    pub cleanup_policy: CleanupPolicy,
    /// Whether to compress stored data
    pub compress_data: bool,
    /// Whether to encrypt sensitive data
    pub encrypt_data: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval_seconds: u64,
}

impl Default for StoragePreferences {
    fn default() -> Self {
        Self {
            max_memory_slots: 100,
            max_persistent_slots: 1000,
            cleanup_policy: CleanupPolicy::default(),
            compress_data: true,
            encrypt_data: false,
            auto_save_interval_seconds: 30,
        }
    }
}

/// Cleanup policy for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    /// Maximum age for slots in seconds
    pub max_age_seconds: u64,
    /// Whether to cleanup on startup
    pub cleanup_on_startup: bool,
    /// Whether to cleanup when storage is full
    pub cleanup_when_full: bool,
    /// Priority for cleanup (oldest first, largest first, etc.)
    pub cleanup_priority: CleanupPriority,
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self {
            max_age_seconds: 86400 * 7, // 7 days
            cleanup_on_startup: true,
            cleanup_when_full: true,
            cleanup_priority: CleanupPriority::OldestFirst,
        }
    }
}

/// Cleanup priority strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupPriority {
    /// Remove oldest slots first
    OldestFirst,
    /// Remove largest slots first
    LargestFirst,
    /// Remove least recently used slots first
    LeastRecentlyUsed,
    /// Remove slots with lowest similarity first
    LowestSimilarity,
}

/// Background processing preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundPreferences {
    /// Whether to enable background processing
    pub enable_background_processing: bool,
    /// Maximum number of concurrent background simulations
    pub max_concurrent_simulations: usize,
    /// Queue size limit
    pub max_queue_size: usize,
    /// Whether to prioritize recent simulations
    pub prioritize_recent: bool,
    /// Background processing timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to pause background processing when tab is inactive
    pub pause_on_inactive: bool,
}

impl Default for BackgroundPreferences {
    fn default() -> Self {
        Self {
            enable_background_processing: true,
            max_concurrent_simulations: 2,
            max_queue_size: 10,
            prioritize_recent: true,
            timeout_seconds: 300, // 5 minutes
            pause_on_inactive: true,
        }
    }
}

/// General UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPreferences {
    /// Theme preference
    pub theme: String,
    /// Language preference
    pub language: String,
    /// Whether to show tooltips
    pub show_tooltips: bool,
    /// Whether to show keyboard shortcuts
    pub show_keyboard_shortcuts: bool,
    /// Animation speed multiplier
    pub animation_speed: f64,
    /// Whether to enable sound effects
    pub enable_sound_effects: bool,
    /// Custom CSS overrides
    pub custom_css: HashMap<String, String>,
}

impl Default for UIPreferences {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            language: "en".to_string(),
            show_tooltips: true,
            show_keyboard_shortcuts: true,
            animation_speed: 1.0,
            enable_sound_effects: false,
            custom_css: HashMap::new(),
        }
    }
}

/// Configuration manager for user preferences
pub struct ConfigManager {
    preferences: UserPreferences,
    /// Whether preferences have been modified
    dirty: bool,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            preferences: UserPreferences::default(),
            dirty: false,
        }
    }

    /// Create a configuration manager with custom preferences
    pub fn with_preferences(preferences: UserPreferences) -> Self {
        Self {
            preferences,
            dirty: false,
        }
    }

    /// Get current preferences
    pub fn get_preferences(&self) -> &UserPreferences {
        &self.preferences
    }

    /// Update preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.preferences = preferences;
        self.dirty = true;
    }

    /// Update display preferences
    pub fn update_display_preferences(&mut self, display: DisplayPreferences) {
        self.preferences.display = display;
        self.dirty = true;
    }

    /// Update progress preferences
    pub fn update_progress_preferences(&mut self, progress: ProgressPreferences) {
        self.preferences.progress = progress;
        self.dirty = true;
    }

    /// Update storage preferences
    pub fn update_storage_preferences(&mut self, storage: StoragePreferences) {
        self.preferences.storage = storage;
        self.dirty = true;
    }

    /// Update background preferences
    pub fn update_background_preferences(&mut self, background: BackgroundPreferences) {
        self.preferences.background = background;
        self.dirty = true;
    }

    /// Update UI preferences
    pub fn update_ui_preferences(&mut self, ui: UIPreferences) {
        self.preferences.ui = ui;
        self.dirty = true;
    }

    /// Check if preferences have been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark preferences as clean (saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Reset preferences to defaults
    pub fn reset_to_defaults(&mut self) {
        self.preferences = UserPreferences::default();
        self.dirty = true;
    }

    /// Export preferences to JSON
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.preferences)
    }

    /// Import preferences from JSON
    pub fn import_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let preferences: UserPreferences = serde_json::from_str(json)?;
        self.preferences = preferences;
        self.dirty = true;
        Ok(())
    }

    /// Validate preferences
    pub fn validate_preferences(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate display preferences
        if self.preferences.display.similarity_threshold < 0.0 || self.preferences.display.similarity_threshold > 1.0 {
            errors.push("Similarity threshold must be between 0.0 and 1.0".to_string());
        }

        if self.preferences.display.max_slots_displayed == 0 {
            errors.push("Max slots displayed must be greater than 0".to_string());
        }

        // Validate progress preferences
        if self.preferences.progress.progress_update_interval_ms == 0 {
            errors.push("Progress update interval must be greater than 0".to_string());
        }

        if self.preferences.progress.progress_segments == 0 {
            errors.push("Progress segments must be greater than 0".to_string());
        }

        // Validate storage preferences
        if self.preferences.storage.max_memory_slots == 0 {
            errors.push("Max memory slots must be greater than 0".to_string());
        }

        if self.preferences.storage.max_persistent_slots == 0 {
            errors.push("Max persistent slots must be greater than 0".to_string());
        }

        // Validate background preferences
        if self.preferences.background.max_concurrent_simulations == 0 {
            errors.push("Max concurrent simulations must be greater than 0".to_string());
        }

        if self.preferences.background.max_queue_size == 0 {
            errors.push("Max queue size must be greater than 0".to_string());
        }

        // Validate UI preferences
        if self.preferences.ui.animation_speed <= 0.0 {
            errors.push("Animation speed must be greater than 0".to_string());
        }

        errors
    }
}

// WASM bindings for JavaScript integration
#[wasm_bindgen]
pub struct ConfigManagerWrapper {
    inner: ConfigManager,
}

impl Default for ConfigManagerWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl ConfigManagerWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ConfigManagerWrapper {
        ConfigManagerWrapper {
            inner: ConfigManager::new(),
        }
    }

    #[wasm_bindgen(js_name = getPreferences)]
    pub fn get_preferences(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.get_preferences())
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = updatePreferences)]
    pub fn update_preferences(&mut self, preferences: &JsValue) -> Result<(), JsValue> {
        let prefs: UserPreferences = serde_wasm_bindgen::from_value(preferences.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse preferences: {}", e)))?;
        
        self.inner.update_preferences(prefs);
        Ok(())
    }

    #[wasm_bindgen(js_name = exportToJson)]
    pub fn export_to_json(&self) -> Result<String, JsValue> {
        self.inner.export_to_json()
            .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
    }

    #[wasm_bindgen(js_name = importFromJson)]
    pub fn import_from_json(&mut self, json: &str) -> Result<(), JsValue> {
        self.inner.import_from_json(json)
            .map_err(|e| JsValue::from_str(&format!("Import error: {}", e)))
    }

    #[wasm_bindgen(js_name = validatePreferences)]
    pub fn validate_preferences(&self) -> Result<JsValue, JsValue> {
        let errors = self.inner.validate_preferences();
        serde_wasm_bindgen::to_value(&errors)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = resetToDefaults)]
    pub fn reset_to_defaults(&mut self) {
        self.inner.reset_to_defaults();
    }

    #[wasm_bindgen(js_name = isDirty)]
    pub fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    #[wasm_bindgen(js_name = markClean)]
    pub fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.display.default_mode, "ShowNewest");
        assert_eq!(prefs.progress.show_progress_indicators, true);
        assert_eq!(prefs.storage.max_memory_slots, 100);
        assert_eq!(prefs.background.enable_background_processing, true);
        assert_eq!(prefs.ui.theme, "light");
    }

    #[test]
    fn test_config_manager() {
        let mut manager = ConfigManager::new();
        assert!(!manager.is_dirty());

        manager.update_display_preferences(DisplayPreferences::default());
        assert!(manager.is_dirty());

        manager.mark_clean();
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_preferences_validation() {
        let mut prefs = UserPreferences::default();
        prefs.display.similarity_threshold = 1.5; // Invalid
        
        let manager = ConfigManager::with_preferences(prefs);
        let errors = manager.validate_preferences();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Similarity threshold")));
    }

    #[test]
    fn test_json_export_import() {
        let manager = ConfigManager::new();
        let json = manager.export_to_json().unwrap();
        
        let mut manager2 = ConfigManager::new();
        manager2.import_from_json(&json).unwrap();
        
        assert_eq!(
            manager.get_preferences().display.default_mode,
            manager2.get_preferences().display.default_mode
        );
    }
}