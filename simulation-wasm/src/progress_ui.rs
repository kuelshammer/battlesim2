use crate::background_simulation::BackgroundSimulationId;
use crate::progress_communication::{ProgressUpdate, ProgressUpdateType, ProgressCommunication};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;

/// UI state for progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressState {
    /// No active simulation
    Idle,
    /// Simulation is running
    Running,
    /// Simulation completed successfully
    Completed,
    /// Simulation failed
    Failed,
    /// Simulation was cancelled
    Cancelled,
}

/// Visual representation of progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressVisual {
    /// Current progress percentage (0.0 to 1.0)
    pub percentage: f64,
    /// Formatted progress string
    pub formatted_progress: String,
    /// Current phase
    pub phase: String,
    /// Time remaining formatted
    pub time_remaining: String,
    /// Iterations completed
    pub iterations_completed: usize,
    /// Total iterations
    pub total_iterations: usize,
    /// Progress bar segments (for visual representation)
    pub progress_segments: Vec<ProgressSegment>,
}

/// A segment of the progress bar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSegment {
    /// Start position (0.0 to 1.0)
    pub start: f64,
    /// End position (0.0 to 1.0)
    pub end: f64,
    /// Color or style for this segment
    pub style: ProgressStyle,
    /// Label for this segment
    pub label: String,
}

/// Style for progress segments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressStyle {
    /// Normal progress
    Normal,
    /// Warning or caution
    Warning,
    /// Error or failure
    Error,
    /// Success or completion
    Success,
    /// Active/running
    Active,
    /// Custom style
    Custom(String),
}

/// Configuration for progress UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUIConfig {
    /// Number of segments in progress bar
    pub progress_segments: usize,
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Whether to show detailed progress
    pub show_detailed: bool,
    /// Whether to show time estimates
    pub show_time_estimates: bool,
    /// Whether to animate progress
    pub animate_progress: bool,
    /// Color scheme
    pub color_scheme: ProgressColorScheme,
}

impl Default for ProgressUIConfig {
    fn default() -> Self {
        Self {
            progress_segments: 20,
            update_interval_ms: 500,
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

/// Complete progress information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Unique identifier for the simulation
    pub simulation_id: BackgroundSimulationId,
    /// Current state
    pub state: ProgressState,
    /// Visual progress representation
    pub visual: ProgressVisual,
    /// Status messages
    pub messages: Vec<String>,
    /// When the simulation started
    pub start_time: u64,
    /// When the simulation ended (if completed)
    pub end_time: Option<u64>,
    /// Total execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ProgressInfo {
    /// Create new progress info
    pub fn new(simulation_id: BackgroundSimulationId) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            simulation_id,
            state: ProgressState::Idle,
            visual: ProgressVisual {
                percentage: 0.0,
                formatted_progress: "Ready".to_string(),
                phase: "Idle".to_string(),
                time_remaining: "Unknown".to_string(),
                iterations_completed: 0,
                total_iterations: 0,
                progress_segments: vec![],
            },
            messages: vec![],
            start_time: now,
            end_time: None,
            execution_time_ms: None,
            metadata: HashMap::new(),
        }
    }

    /// Update from progress update
    pub fn update_from_progress_update(&mut self, update: &ProgressUpdate, config: &ProgressUIConfig) {
        // Update state based on update type
        self.state = match update.update_type {
            ProgressUpdateType::Started => ProgressState::Running,
            ProgressUpdateType::Progress => ProgressState::Running,
            ProgressUpdateType::Completed => {
                self.end_time = Some(update.timestamp);
                ProgressState::Completed
            },
            ProgressUpdateType::Failed => {
                self.end_time = Some(update.timestamp);
                ProgressState::Failed
            },
            ProgressUpdateType::Cancelled => {
                self.end_time = Some(update.timestamp);
                ProgressState::Cancelled
            },
        };

        // Update visual representation
        self.visual = ProgressVisual {
            percentage: update.progress_percentage,
            formatted_progress: update.format_progress(),
            phase: update.current_phase.clone(),
            time_remaining: update.format_time_remaining(),
            iterations_completed: update.iterations_completed,
            total_iterations: update.total_iterations,
            progress_segments: self.create_progress_segments(update.progress_percentage, config),
        };

        // Update messages
        self.messages = update.messages.clone();

        // Calculate execution time if completed
        if let Some(end_time) = self.end_time {
            self.execution_time_ms = Some((end_time - self.start_time) * 1000);
        }
    }

    /// Create progress bar segments
    fn create_progress_segments(&self, percentage: f64, config: &ProgressUIConfig) -> Vec<ProgressSegment> {
        let mut segments = Vec::new();
        let segment_size = 1.0 / config.progress_segments as f64;
        let filled_segments = (percentage * config.progress_segments as f64).floor() as usize;

        for i in 0..config.progress_segments {
            let start = i as f64 * segment_size;
            let end = (i + 1) as f64 * segment_size;

            let style = if i < filled_segments {
                match self.state {
                    ProgressState::Completed => ProgressStyle::Success,
                    ProgressState::Failed | ProgressState::Cancelled => ProgressStyle::Error,
                    ProgressState::Running => ProgressStyle::Active,
                    _ => ProgressStyle::Normal,
                }
            } else if i == filled_segments && (percentage * config.progress_segments as f64).fract() > 0.0 {
                ProgressStyle::Active
            } else {
                ProgressStyle::Normal
            };

            segments.push(ProgressSegment {
                start,
                end,
                style,
                label: if config.show_detailed {
                    format!("{:.0}%", (i + 1) as f64 * segment_size * 100.0)
                } else {
                    String::new()
                },
            });
        }

        segments
    }

    /// Check if progress is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.state, ProgressState::Completed | ProgressState::Failed | ProgressState::Cancelled)
    }

    /// Check if progress is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ProgressState::Running)
    }

    /// Get formatted execution time
    pub fn formatted_execution_time(&self) -> String {
        if let Some(ms) = self.execution_time_ms {
            if ms < 1000 {
                format!("{}ms", ms)
            } else if ms < 60_000 {
                format!("{:.1}s", ms as f64 / 1000.0)
            } else if ms < 3_600_000 {
                format!("{:.1}m", ms as f64 / 60_000.0)
            } else {
                format!("{:.1}h", ms as f64 / 3_600_000.0)
            }
        } else {
            "Unknown".to_string()
        }
    }
}

/// Manager for progress UI components
pub struct ProgressUIManager {
    /// Configuration
    config: ProgressUIConfig,
    /// Active progress tracking
    active_progress: Arc<Mutex<HashMap<BackgroundSimulationId, ProgressInfo>>>,
    /// Progress communication system
    progress_comm: ProgressCommunication,
    /// Last update timestamp
    last_update: Arc<Mutex<u64>>,
}

impl ProgressUIManager {
    /// Create a new progress UI manager
    pub fn new(config: ProgressUIConfig) -> Self {
        let (progress_comm, _) = ProgressCommunication::new();
        
        Self {
            config,
            active_progress: Arc::new(Mutex::new(HashMap::new())),
            progress_comm,
            last_update: Arc::new(Mutex::new(0)),
        }
    }

    /// Start tracking a new simulation
    pub fn start_tracking(&self, simulation_id: BackgroundSimulationId) {
        let mut active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        active.insert(simulation_id.clone(), ProgressInfo::new(simulation_id));
    }

    /// Stop tracking a simulation
    pub fn stop_tracking(&self, simulation_id: &BackgroundSimulationId) {
        let mut active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        active.remove(simulation_id);
    }

    /// Update progress from a progress update
    pub fn update_progress(&self, update: ProgressUpdate) -> Result<(), ProgressUIError> {
        let mut active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        
        if let Some(progress_info) = active.get_mut(&update.simulation_id) {
            progress_info.update_from_progress_update(&update, &self.config);
            
            // Update last update timestamp
            let mut last_update = self.last_update.lock().unwrap_or_else(PoisonError::into_inner);
            *last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
                
            Ok(())
        } else {
            Err(ProgressUIError::SimulationNotFound)
        }
    }

    /// Get progress info for a specific simulation
    pub fn get_progress(&self, simulation_id: &BackgroundSimulationId) -> Option<ProgressInfo> {
        let active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        active.get(simulation_id).cloned()
    }

    /// Get all active progress
    pub fn get_all_progress(&self) -> Vec<ProgressInfo> {
        let active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        active.values().cloned().collect()
    }

    /// Get progress summary for dashboard
    pub fn get_progress_summary(&self) -> ProgressSummary {
        let active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        let mut summary = ProgressSummary::default();

        for progress_info in active.values() {
            match progress_info.state {
                ProgressState::Idle => summary.idle += 1,
                ProgressState::Running => summary.running += 1,
                ProgressState::Completed => summary.completed += 1,
                ProgressState::Failed => summary.failed += 1,
                ProgressState::Cancelled => summary.cancelled += 1,
            }

            if progress_info.is_active() {
                summary.total_progress += progress_info.visual.percentage;
                summary.active_simulations += 1;
            }
        }

        if summary.active_simulations > 0 {
            summary.average_progress = summary.total_progress / summary.active_simulations as f64;
        }

        summary
    }

    /// Create a progress bar HTML representation
    pub fn create_progress_bar_html(&self, progress_info: &ProgressInfo) -> String {
        let mut html = String::new();
        let config = &self.config;

        html.push_str(&format!(
            r#"<div class="progress-container" data-simulation-id="{}">"#,
            progress_info.simulation_id.0
        ));

        // Progress bar
        html.push_str(r#"<div class="progress-bar">"#);
        
        for segment in &progress_info.visual.progress_segments {
            let color = match segment.style {
                ProgressStyle::Normal => &config.color_scheme.normal_color,
                ProgressStyle::Active => &config.color_scheme.active_color,
                ProgressStyle::Success => &config.color_scheme.success_color,
                ProgressStyle::Warning => &config.color_scheme.warning_color,
                ProgressStyle::Error => &config.color_scheme.error_color,
                ProgressStyle::Custom(ref color) => color,
            };

            html.push_str(&format!(
                r#"<div class="progress-segment" style="left: {:.1}%; width: {:.1}%; background-color: {};""#,
                segment.start * 100.0,
                (segment.end - segment.start) * 100.0,
                color
            ));

            if !segment.label.is_empty() {
                html.push_str(&format!(r#" title="{}""#, segment.label));
            }

            html.push_str(r#"></div>"#);
        }

        html.push_str(r#"</div>"#);

        // Progress text
        if config.show_detailed {
            html.push_str(&format!(
                r#"<div class="progress-text">{} - {}</div>"#,
                progress_info.visual.formatted_progress,
                progress_info.visual.phase
            ));
        }

        html.push_str(r#"</div>"#);
        html
    }

    /// Create a compact progress indicator
    pub fn create_compact_indicator(&self, progress_info: &ProgressInfo) -> String {
        let state_class = match progress_info.state {
            ProgressState::Idle => "idle",
            ProgressState::Running => "running",
            ProgressState::Completed => "completed",
            ProgressState::Failed => "failed",
            ProgressState::Cancelled => "cancelled",
        };

        format!(
            r#"<div class="progress-indicator {}" data-simulation-id="{}" title="{}">{:.1}%</div>"#,
            state_class,
            progress_info.simulation_id.0,
            progress_info.visual.phase,
            progress_info.visual.percentage * 100.0
        )
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ProgressUIConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ProgressUIConfig {
        &self.config
    }

    /// Clear all progress tracking
    pub fn clear_all(&self) {
        let mut active = self.active_progress.lock().unwrap_or_else(PoisonError::into_inner);
        active.clear();
    }

    /// Get progress communication system for external updates
    pub fn get_progress_comm(&self) -> &ProgressCommunication {
        &self.progress_comm
    }
}

/// Summary of all progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProgressSummary {
    /// Number of idle simulations
    pub idle: usize,
    /// Number of running simulations
    pub running: usize,
    /// Number of completed simulations
    pub completed: usize,
    /// Number of failed simulations
    pub failed: usize,
    /// Number of cancelled simulations
    pub cancelled: usize,
    /// Number of active simulations
    pub active_simulations: usize,
    /// Average progress across active simulations
    pub average_progress: f64,
    /// Total progress sum (for calculation)
    pub total_progress: f64,
}

/// Errors that can occur in progress UI
#[derive(Debug, Clone)]
pub enum ProgressUIError {
    /// Simulation not found
    SimulationNotFound,
    /// Invalid progress data
    InvalidProgressData(String),
    /// Communication error
    CommunicationError(String),
}

impl std::fmt::Display for ProgressUIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressUIError::SimulationNotFound => write!(f, "Simulation not found"),
            ProgressUIError::InvalidProgressData(msg) => write!(f, "Invalid progress data: {}", msg),
            ProgressUIError::CommunicationError(msg) => write!(f, "Communication error: {}", msg),
        }
    }
}

impl std::error::Error for ProgressUIError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background_simulation::BackgroundSimulationId;

    #[test]
    fn test_progress_info_creation() {
        let sim_id = BackgroundSimulationId::new();
        let progress_info = ProgressInfo::new(sim_id.clone());

        assert_eq!(progress_info.simulation_id, sim_id);
        assert!(matches!(progress_info.state, ProgressState::Idle));
        assert_eq!(progress_info.visual.percentage, 0.0);
    }

    #[test]
    fn test_progress_segments() {
        let sim_id = BackgroundSimulationId::new();
        let mut progress_info = ProgressInfo::new(sim_id);
        let config = ProgressUIConfig {
            progress_segments: 10,
            ..Default::default()
        };

        progress_info.visual.percentage = 0.75;
        let segments = progress_info.create_progress_segments(0.75, &config);

        assert_eq!(segments.len(), 10);
        assert_eq!(segments.iter().filter(|s| matches!(s.style, ProgressStyle::Active)).count(), 1);
    }

    #[test]
    fn test_progress_summary() {
        let manager = ProgressUIManager::new(ProgressUIConfig::default());
        let summary = manager.get_progress_summary();

        assert_eq!(summary.running, 0);
        assert_eq!(summary.completed, 0);
        assert_eq!(summary.average_progress, 0.0);
    }

    #[test]
    fn test_progress_bar_html() {
        let sim_id = BackgroundSimulationId::new();
        let mut progress_info = ProgressInfo::new(sim_id);
        progress_info.visual.percentage = 0.5;
        progress_info.visual.progress_segments = vec![
            ProgressSegment {
                start: 0.0,
                end: 0.5,
                style: ProgressStyle::Success,
                label: "50%".to_string(),
            },
            ProgressSegment {
                start: 0.5,
                end: 1.0,
                style: ProgressStyle::Normal,
                label: "".to_string(),
            },
        ];

        let manager = ProgressUIManager::new(ProgressUIConfig::default());
        let html = manager.create_progress_bar_html(&progress_info);

        assert!(html.contains("progress-container"));
        assert!(html.contains("progress-bar"));
        assert!(html.contains("progress-segment"));
    }

    #[test]
    fn test_compact_indicator() {
        let sim_id = BackgroundSimulationId::new();
        let mut progress_info = ProgressInfo::new(sim_id);
        progress_info.state = ProgressState::Running;
        progress_info.visual.percentage = 0.75;
        progress_info.visual.phase = "Running".to_string();

        let manager = ProgressUIManager::new(ProgressUIConfig::default());
        let indicator = manager.create_compact_indicator(&progress_info);

        assert!(indicator.contains("progress-indicator"));
        assert!(indicator.contains("75.0%"));
        assert!(indicator.contains("running"));
    }
}

// WASM bindings for JavaScript integration
#[wasm_bindgen]
pub struct ProgressUIManagerWrapper {
    inner: Arc<Mutex<ProgressUIManager>>,
}

#[wasm_bindgen]
impl ProgressUIManagerWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ProgressUIManagerWrapper, JsValue> {
        let config = ProgressUIConfig::default();
        let manager = ProgressUIManager::new(config);
        Ok(ProgressUIManagerWrapper {
            inner: Arc::new(Mutex::new(manager))
        })
    }

    #[wasm_bindgen(js_name = startTracking)]
    pub fn start_tracking(&self, simulation_id: String) -> Result<(), JsValue> {
        let sim_id = BackgroundSimulationId::from_string(&simulation_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid simulation ID: {}", e)))?;
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .start_tracking(sim_id);
        Ok(())
    }

    #[wasm_bindgen(js_name = stopTracking)]
    pub fn stop_tracking(&self, simulation_id: String) -> Result<(), JsValue> {
        let sim_id = BackgroundSimulationId::from_string(&simulation_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid simulation ID: {}", e)))?;
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .stop_tracking(&sim_id);
        Ok(())
    }

    #[wasm_bindgen(js_name = getProgress)]
    pub fn get_progress(&self, simulation_id: String) -> Result<JsValue, JsValue> {
        let sim_id = BackgroundSimulationId::from_string(&simulation_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid simulation ID: {}", e)))?;
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_progress(&sim_id)
            .ok_or_else(|| JsValue::from_str("Progress not found"))
            .and_then(|progress| {
                serde_wasm_bindgen::to_value(&progress)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            })
    }

    #[wasm_bindgen(js_name = getAllProgress)]
    pub fn get_all_progress(&self) -> Result<JsValue, JsValue> {
        let progress_list = self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_all_progress();
        
        serde_wasm_bindgen::to_value(&progress_list)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = getProgressSummary)]
    pub fn get_progress_summary(&self) -> Result<JsValue, JsValue> {
        let summary = self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_progress_summary();
        
        serde_wasm_bindgen::to_value(&summary)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    #[wasm_bindgen(js_name = createProgressBarHtml)]
    pub fn create_progress_bar_html(&self, simulation_id: String) -> Result<String, JsValue> {
        let sim_id = BackgroundSimulationId::from_string(&simulation_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid simulation ID: {}", e)))?;
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_progress(&sim_id)
            .ok_or_else(|| JsValue::from_str("Progress not found"))
            .map(|progress| self.inner.lock().unwrap_or_else(PoisonError::into_inner).create_progress_bar_html(&progress))
    }

    #[wasm_bindgen(js_name = createCompactIndicator)]
    pub fn create_compact_indicator(&self, simulation_id: String) -> Result<String, JsValue> {
        let sim_id = BackgroundSimulationId::from_string(&simulation_id)
            .map_err(|e| JsValue::from_str(&format!("Invalid simulation ID: {}", e)))?;
        
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .get_progress(&sim_id)
            .ok_or_else(|| JsValue::from_str("Progress not found"))
            .map(|progress| self.inner.lock().unwrap_or_else(PoisonError::into_inner).create_compact_indicator(&progress))
    }

    #[wasm_bindgen(js_name = clearAll)]
    pub fn clear_all(&self) -> Result<(), JsValue> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
            .clear_all();
        Ok(())
    }
}