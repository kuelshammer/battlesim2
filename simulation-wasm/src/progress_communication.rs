use crate::background_simulation::{BackgroundSimulationId, SimulationProgress};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of progress updates that can be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressUpdateType {
    /// Initial progress when simulation starts
    Started,
    /// Regular progress update
    Progress,
    /// Simulation completed successfully
    Completed,
    /// Simulation failed
    Failed,
    /// Simulation was cancelled
    Cancelled,
}

/// A progress update that can be sent to subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Unique identifier for this update
    pub update_id: String,
    /// Simulation identifier
    pub simulation_id: BackgroundSimulationId,
    /// Type of update
    pub update_type: ProgressUpdateType,
    /// Progress percentage (0.0 to 1.0)
    pub progress_percentage: f64,
    /// Number of iterations completed
    pub iterations_completed: usize,
    /// Total iterations to run
    pub total_iterations: usize,
    /// Estimated time remaining in milliseconds
    pub estimated_time_remaining_ms: Option<u64>,
    /// Current phase of simulation
    pub current_phase: String,
    /// Any status messages
    pub messages: Vec<String>,
    /// When this update was generated
    pub timestamp: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ProgressUpdate {
    /// Create a new progress update from simulation progress
    pub fn from_progress(progress: &SimulationProgress, update_type: ProgressUpdateType) -> Self {
        Self {
            update_id: format!(
                "update_{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            simulation_id: progress.simulation_id.clone(),
            update_type,
            progress_percentage: progress.progress_percentage,
            iterations_completed: progress.iterations_completed,
            total_iterations: progress.total_iterations,
            estimated_time_remaining_ms: progress.estimated_time_remaining_ms,
            current_phase: progress.current_phase.clone(),
            messages: progress.messages.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Create a simple progress update
    pub fn new(
        simulation_id: BackgroundSimulationId,
        update_type: ProgressUpdateType,
        progress_percentage: f64,
        phase: &str,
    ) -> Self {
        Self {
            update_id: format!(
                "update_{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            simulation_id,
            update_type,
            progress_percentage,
            iterations_completed: 0,
            total_iterations: 0,
            estimated_time_remaining_ms: None,
            current_phase: phase.to_string(),
            messages: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the update
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add a message to the update
    pub fn with_message(mut self, message: String) -> Self {
        self.messages.push(message);
        self
    }

    /// Check if this update represents a completion state
    pub fn is_completion(&self) -> bool {
        matches!(
            self.update_type,
            ProgressUpdateType::Completed
                | ProgressUpdateType::Failed
                | ProgressUpdateType::Cancelled
        )
    }

    /// Check if this update represents an error state
    pub fn is_error(&self) -> bool {
        matches!(
            self.update_type,
            ProgressUpdateType::Failed | ProgressUpdateType::Cancelled
        )
    }

    /// Get a formatted progress string
    pub fn format_progress(&self) -> String {
        if self.total_iterations > 0 {
            format!(
                "{}: {:.1}% ({}/{}) - {}",
                self.current_phase,
                self.progress_percentage * 100.0,
                self.iterations_completed,
                self.total_iterations,
                self.format_time_remaining()
            )
        } else {
            format!(
                "{}: {:.1}%",
                self.current_phase,
                self.progress_percentage * 100.0
            )
        }
    }

    /// Format estimated time remaining
    pub fn format_time_remaining(&self) -> String {
        if let Some(remaining_ms) = self.estimated_time_remaining_ms {
            if remaining_ms < 1000 {
                format!("{}ms", remaining_ms)
            } else if remaining_ms < 60_000 {
                format!("{:.1}s", remaining_ms as f64 / 1000.0)
            } else if remaining_ms < 3_600_000 {
                format!("{:.1}m", remaining_ms as f64 / 60_000.0)
            } else {
                format!("{:.1}h", remaining_ms as f64 / 3_600_000.0)
            }
        } else {
            "Unknown".to_string()
        }
    }
}

/// Subscription filter type
pub type ProgressFilter = Box<dyn Fn(&ProgressUpdate) -> bool + Send + Sync>;

/// Subscription filter for progress updates
pub struct ProgressSubscription {
    /// Unique subscription identifier
    pub subscription_id: String,
    /// Simulation ID to subscribe to (None for all simulations)
    pub simulation_id: Option<BackgroundSimulationId>,
    /// Minimum update type to receive (inclusive)
    pub min_update_type: ProgressUpdateType,
    /// Whether to receive only completion updates
    pub completions_only: bool,
    /// Custom filter function
    pub custom_filter: Option<ProgressFilter>,
}

impl Clone for ProgressSubscription {
    fn clone(&self) -> Self {
        Self {
            subscription_id: self.subscription_id.clone(),
            simulation_id: self.simulation_id.clone(),
            min_update_type: self.min_update_type.clone(),
            completions_only: self.completions_only,
            custom_filter: None, // Can't clone function pointers, reset to None
        }
    }
}

impl std::fmt::Debug for ProgressSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressSubscription")
            .field("subscription_id", &self.subscription_id)
            .field("simulation_id", &self.simulation_id)
            .field("min_update_type", &self.min_update_type)
            .field("completions_only", &self.completions_only)
            .field("custom_filter", &self.custom_filter.is_some())
            .finish()
    }
}

impl ProgressSubscription {
    /// Create a new subscription
    pub fn new(subscription_id: String) -> Self {
        Self {
            subscription_id,
            simulation_id: None,
            min_update_type: ProgressUpdateType::Started,
            completions_only: false,
            custom_filter: None,
        }
    }

    /// Subscribe to a specific simulation
    pub fn for_simulation(mut self, simulation_id: BackgroundSimulationId) -> Self {
        self.simulation_id = Some(simulation_id);
        self
    }

    /// Set minimum update type
    pub fn min_type(mut self, min_type: ProgressUpdateType) -> Self {
        self.min_update_type = min_type;
        self
    }

    /// Only receive completion updates
    pub fn completions_only(mut self) -> Self {
        self.completions_only = true;
        self
    }

    /// Add custom filter
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&ProgressUpdate) -> bool + Send + Sync + 'static,
    {
        self.custom_filter = Some(Box::new(filter));
        self
    }

    /// Check if an update matches this subscription
    pub fn matches(&self, update: &ProgressUpdate) -> bool {
        // Check simulation ID filter
        if let Some(ref sim_id) = self.simulation_id {
            if update.simulation_id != *sim_id {
                return false;
            }
        }

        // Check completion-only filter
        if self.completions_only && !update.is_completion() {
            return false;
        }

        // Check minimum update type (simple priority check)
        if !self.meets_min_type(&update.update_type) {
            return false;
        }

        // Check custom filter
        if let Some(ref filter) = self.custom_filter {
            if !filter(update) {
                return false;
            }
        }

        true
    }

    /// Simple priority-based type checking
    fn meets_min_type(&self, update_type: &ProgressUpdateType) -> bool {
        // Define priority order (higher = more significant)
        let type_priority = |t: &ProgressUpdateType| match t {
            ProgressUpdateType::Started => 0,
            ProgressUpdateType::Progress => 1,
            ProgressUpdateType::Completed => 2,
            ProgressUpdateType::Failed => 3,
            ProgressUpdateType::Cancelled => 4,
        };

        type_priority(update_type) >= type_priority(&self.min_update_type)
    }
}

/// Thread-safe progress communication system
pub struct ProgressCommunication {
    /// Channel for broadcasting updates
    update_sender: mpsc::Sender<ProgressUpdate>,
    /// Active subscriptions
    subscriptions: Arc<Mutex<HashMap<String, ProgressSubscription>>>,
    /// Channel receivers for each subscription
    subscription_channels: Arc<Mutex<HashMap<String, mpsc::Sender<ProgressUpdate>>>>,
}

impl ProgressCommunication {
    /// Create a new progress communication system
    pub fn new() -> (Self, mpsc::Receiver<ProgressUpdate>) {
        let (update_sender, update_receiver) = mpsc::channel();

        let system = Self {
            update_sender,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            subscription_channels: Arc::new(Mutex::new(HashMap::new())),
        };

        (system, update_receiver)
    }

    /// Send a progress update to all matching subscribers
    pub fn send_update(&self, update: ProgressUpdate) -> Result<(), ProgressError> {
        // Send to the main broadcast channel
        self.update_sender
            .send(update.clone())
            .map_err(|_| ProgressError::SendError)?;

        // Send to matching subscriptions
        let subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let channels = self
            .subscription_channels
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        for (subscription_id, subscription) in subscriptions.iter() {
            if subscription.matches(&update) {
                if let Some(sender) = channels.get(subscription_id) {
                    // Don't fail if a subscriber is disconnected
                    let _ = sender.send(update.clone());
                }
            }
        }

        Ok(())
    }

    /// Subscribe to progress updates
    pub fn subscribe(
        &self,
        subscription: ProgressSubscription,
    ) -> Result<mpsc::Receiver<ProgressUpdate>, ProgressError> {
        let (sender, receiver) = mpsc::channel();
        let subscription_id = subscription.subscription_id.clone();

        // Add subscription
        {
            let mut subscriptions = self
                .subscriptions
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // Add channel
        {
            let mut channels = self
                .subscription_channels
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            channels.insert(subscription_id, sender);
        }

        Ok(receiver)
    }

    /// Unsubscribe from progress updates
    pub fn unsubscribe(&self, subscription_id: &str) -> Result<(), ProgressError> {
        // Remove subscription
        {
            let mut subscriptions = self
                .subscriptions
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            subscriptions.remove(subscription_id);
        }

        // Remove channel
        {
            let mut channels = self
                .subscription_channels
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            channels.remove(subscription_id);
        }

        Ok(())
    }

    /// Get all active subscriptions
    pub fn get_subscriptions(&self) -> Vec<ProgressSubscription> {
        let subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        subscriptions.values().cloned().collect()
    }

    /// Get subscription count
    pub fn subscription_count(&self) -> usize {
        let subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        subscriptions.len()
    }

    /// Clear all subscriptions
    pub fn clear_subscriptions(&self) {
        let mut subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut channels = self
            .subscription_channels
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        subscriptions.clear();
        channels.clear();
    }

    /// Send a simple progress update
    pub fn send_simple_update(
        &self,
        simulation_id: BackgroundSimulationId,
        update_type: ProgressUpdateType,
        progress_percentage: f64,
        phase: &str,
    ) -> Result<(), ProgressError> {
        let update = ProgressUpdate::new(simulation_id, update_type, progress_percentage, phase);
        self.send_update(update)
    }

    /// Send a progress update from simulation progress
    pub fn send_from_progress(
        &self,
        progress: &SimulationProgress,
        update_type: ProgressUpdateType,
    ) -> Result<(), ProgressError> {
        let update = ProgressUpdate::from_progress(progress, update_type);
        self.send_update(update)
    }
}

impl Default for ProgressCommunication {
    fn default() -> Self {
        let (system, _) = Self::new();
        system
    }
}

impl Clone for ProgressCommunication {
    fn clone(&self) -> Self {
        // Create a new communication system - this is a simplified clone
        // In a real implementation, you might want to share same channels
        Self::default()
    }
}

/// Errors that can occur during progress communication
#[derive(Debug, Clone)]
pub enum ProgressError {
    /// Failed to send update
    SendError,
    /// Subscription not found
    SubscriptionNotFound,
    /// Invalid subscription parameters
    InvalidSubscription(String),
}

impl std::fmt::Display for ProgressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressError::SendError => write!(f, "Failed to send progress update"),
            ProgressError::SubscriptionNotFound => write!(f, "Subscription not found"),
            ProgressError::InvalidSubscription(msg) => write!(f, "Invalid subscription: {}", msg),
        }
    }
}

impl std::error::Error for ProgressError {}

/// Utility functions for progress calculation and formatting
pub mod progress_utils {

    /// Calculate progress percentage with safe division
    pub fn calculate_percentage(completed: usize, total: usize) -> f64 {
        if total == 0 {
            1.0
        } else {
            (completed as f64 / total as f64).min(1.0)
        }
    }

    /// Estimate time remaining based on current progress
    pub fn estimate_time_remaining(elapsed_ms: u64, completed: usize, total: usize) -> Option<u64> {
        if completed == 0 || total == 0 {
            return None;
        }

        let avg_time_per_iteration = elapsed_ms as f64 / completed as f64;
        let remaining_iterations = total.saturating_sub(completed) as f64;

        Some((avg_time_per_iteration * remaining_iterations) as u64)
    }

    /// Format a duration in milliseconds to human-readable string
    pub fn format_duration(duration_ms: u64) -> String {
        if duration_ms < 1000 {
            format!("{}ms", duration_ms)
        } else if duration_ms < 60_000 {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        } else if duration_ms < 3_600_000 {
            format!("{:.1}m", duration_ms as f64 / 60_000.0)
        } else {
            format!("{:.1}h", duration_ms as f64 / 3_600_000.0)
        }
    }

    /// Create a standard progress message
    pub fn create_progress_message(
        phase: &str,
        completed: usize,
        total: usize,
        elapsed_ms: u64,
    ) -> String {
        let percentage = calculate_percentage(completed, total);
        let time_remaining = estimate_time_remaining(elapsed_ms, completed, total)
            .map(format_duration)
            .unwrap_or_else(|| "Unknown".to_string());

        format!(
            "{}: {:.1}% ({}/{}) - {} remaining",
            phase,
            percentage * 100.0,
            completed,
            total,
            time_remaining
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background_simulation::{BackgroundSimulationId, SimulationProgress};

    #[test]
    fn test_progress_update_creation() {
        let sim_id = BackgroundSimulationId::new();
        let update =
            ProgressUpdate::new(sim_id.clone(), ProgressUpdateType::Progress, 0.5, "Running");

        assert_eq!(update.simulation_id, sim_id);
        assert_eq!(update.progress_percentage, 0.5);
        assert_eq!(update.current_phase, "Running");
        assert!(!update.is_completion());
        assert!(!update.is_error());
    }

    #[test]
    fn test_progress_update_from_simulation_progress() {
        let sim_id = BackgroundSimulationId::new();
        let mut progress = SimulationProgress::new(sim_id.clone(), 100);
        progress.update_progress(50, "Running");
        progress.add_message("Test message".to_string());

        let update = ProgressUpdate::from_progress(&progress, ProgressUpdateType::Progress);

        assert_eq!(update.simulation_id, sim_id);
        assert_eq!(update.progress_percentage, 0.5);
        assert_eq!(update.iterations_completed, 50);
        assert_eq!(update.total_iterations, 100);
        assert_eq!(update.current_phase, "Running");
        assert!(update.messages.contains(&"Test message".to_string()));
    }

    #[test]
    fn test_progress_subscription() {
        let sim_id = BackgroundSimulationId::new();
        let subscription = ProgressSubscription::new("test_sub".to_string())
            .for_simulation(sim_id.clone())
            .completions_only();

        assert_eq!(subscription.subscription_id, "test_sub");
        assert_eq!(subscription.simulation_id, Some(sim_id.clone()));
        assert!(subscription.completions_only);

        // Test matching
        let progress_update =
            ProgressUpdate::new(sim_id.clone(), ProgressUpdateType::Progress, 0.5, "Running");
        assert!(!subscription.matches(&progress_update)); // Not a completion

        let completion_update = ProgressUpdate::new(
            sim_id.clone(),
            ProgressUpdateType::Completed,
            1.0,
            "Completed",
        );
        assert!(subscription.matches(&completion_update)); // Is a completion
    }

    #[test]
    fn test_progress_communication() {
        let (comm, _receiver) = ProgressCommunication::new();
        let sim_id = BackgroundSimulationId::new();

        // Subscribe to all updates
        let subscription = ProgressSubscription::new("test".to_string());
        let sub_receiver = comm.subscribe(subscription).unwrap();

        // Send an update
        let update = ProgressUpdate::new(sim_id, ProgressUpdateType::Started, 0.0, "Starting");
        comm.send_update(update).unwrap();

        // Receive the update
        let received = sub_receiver.try_recv().unwrap();
        assert_eq!(received.current_phase, "Starting");
    }

    #[test]
    fn test_progress_utils() {
        // Test percentage calculation
        assert_eq!(progress_utils::calculate_percentage(50, 100), 0.5);
        assert_eq!(progress_utils::calculate_percentage(100, 100), 1.0);
        assert_eq!(progress_utils::calculate_percentage(0, 0), 1.0); // Edge case

        // Test time estimation
        let estimated = progress_utils::estimate_time_remaining(1000, 10, 100);
        assert_eq!(estimated, Some(9000)); // 100ms per iteration * 90 remaining

        // Test duration formatting
        assert_eq!(progress_utils::format_duration(500), "500ms");
        assert_eq!(progress_utils::format_duration(1500), "1.5s");
        assert_eq!(progress_utils::format_duration(90000), "1.5m");
        assert_eq!(progress_utils::format_duration(7200000), "2.0h");
    }

    #[test]
    fn test_update_formatting() {
        let mut update = ProgressUpdate::new(
            BackgroundSimulationId::new(),
            ProgressUpdateType::Progress,
            0.75,
            "Running",
        );
        update.iterations_completed = 75;
        update.total_iterations = 100;
        update.estimated_time_remaining_ms = Some(25000);

        let formatted = update.format_progress();
        assert!(formatted.contains("75.0%"));
        assert!(formatted.contains("(75/100)"));
        assert!(formatted.contains("Running"));
    }
}
