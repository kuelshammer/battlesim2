use crate::background_simulation::{SimulationPriority};
use crate::user_interaction::ScenarioParameters;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// A request to run a simulation with specific parameters
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationRequest {
    /// Unique identifier for this request
    pub request_id: String,
    /// Simulation parameters
    pub parameters: ScenarioParameters,
    /// Priority level (higher = more important)
    pub priority: SimulationPriority,
    /// When this request was created
    pub timestamp: u64,
    /// Optional callback for progress updates
    pub progress_callback: Option<String>,
    /// Whether this request can be deduplicated
    pub allow_deduplication: bool,
}

impl SimulationRequest {
    /// Create a new simulation request
    pub fn new(parameters: ScenarioParameters, priority: SimulationPriority) -> Self {
        Self {
            request_id: format!("req_{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            parameters,
            priority,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            progress_callback: None,
            allow_deduplication: true,
        }

    }

    /// Create a request with custom callback
    pub fn with_callback(mut self, callback: String) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// Disable deduplication for this request
    pub fn without_deduplication(mut self) -> Self {
        self.allow_deduplication = false;
        self
    }

    /// Calculate a hash for deduplication purposes
    pub fn deduplication_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash key parameters that determine if simulations are equivalent
        self.parameters.players.len().hash(&mut hasher);
        self.parameters.encounters.len().hash(&mut hasher);
        self.parameters.iterations.hash(&mut hasher);
        
        // Hash player characteristics
        for player in &self.parameters.players {
            player.name.hash(&mut hasher);
            player.hp.hash(&mut hasher); // hp is u32
            player.ac.hash(&mut hasher); // ac is u32
        }
        
        // Hash encounter characteristics
        for encounter in &self.parameters.encounters {
            encounter.monsters.len().hash(&mut hasher);
            for monster in &encounter.monsters {
                monster.name.hash(&mut hasher);
                monster.hp.hash(&mut hasher); // hp is u32
                monster.ac.hash(&mut hasher); // ac is u32
            }
        }
        
        format!("{:x}", hasher.finish())
    }
}

/// Wrapper for priority queue ordering (reverse order for max-heap)
#[derive(Debug)]
struct PriorityRequest {
    request: SimulationRequest,
    // For tie-breaking: newer requests get higher priority
    insertion_order: u64,
}

impl PartialEq for PriorityRequest {
    fn eq(&self, other: &Self) -> bool {
        self.request.priority == other.request.priority && 
        self.insertion_order == other.insertion_order
    }
}

impl Eq for PriorityRequest {}

impl Ord for PriorityRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for max-heap (higher priority first)
        other.request.priority.cmp(&self.request.priority)
            .then_with(|| other.insertion_order.cmp(&self.insertion_order))
    }
}

impl PartialOrd for PriorityRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Thread-safe queue for managing simulation requests
pub struct SimulationQueue {
    /// Priority queue of pending requests
    pending_requests: Arc<Mutex<BinaryHeap<PriorityRequest>>>,
    /// Set of request IDs currently being processed
    processing_requests: Arc<Mutex<HashSet<String>>>,
    /// Map of deduplication hashes to request IDs
    deduplication_map: Arc<Mutex<HashMap<String, String>>>,
    /// Counter for insertion order
    insertion_counter: Arc<Mutex<u64>>,
    /// Maximum queue size
    max_queue_size: usize,
}

impl SimulationQueue {
    /// Create a new simulation queue
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            pending_requests: Arc::new(Mutex::new(BinaryHeap::new())),
            processing_requests: Arc::new(Mutex::new(HashSet::new())),
            deduplication_map: Arc::new(Mutex::new(HashMap::new())),
            insertion_counter: Arc::new(Mutex::new(0)),
            max_queue_size,
        }
    }

    /// Add a new request to the queue
    pub fn enqueue(&self, request: SimulationRequest) -> Result<(), QueueError> {
        // Check queue size limit
        {
            let pending = self.pending_requests.lock().unwrap();
            if pending.len() >= self.max_queue_size {
                return Err(QueueError::QueueFull);
            }
        }

        // Check for deduplication
        if request.allow_deduplication {
            let dedup_hash = request.deduplication_hash();
            let mut dedup_map = self.deduplication_map.lock().unwrap();
            
            if let Some(existing_request_id) = dedup_map.get(&dedup_hash) {
                // Check if the existing request is still pending or processing
                let pending = self.pending_requests.lock().unwrap();
                let processing = self.processing_requests.lock().unwrap();
                
                if pending.iter().any(|pr| pr.request.request_id == *existing_request_id) ||
                   processing.contains(existing_request_id) {
                    return Err(QueueError::DuplicateRequest(existing_request_id.clone()));
                }
            }
            
            // Add to deduplication map
            dedup_map.insert(dedup_hash, request.request_id.clone());
        }

        // Add to priority queue
        let insertion_order = {
            let mut counter = self.insertion_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        let priority_request = PriorityRequest {
            request,
            insertion_order,
        };

        let mut pending = self.pending_requests.lock().unwrap();
        pending.push(priority_request);

        Ok(())
    }

    /// Get the next request from the queue (highest priority first)
    pub fn dequeue(&self) -> Option<SimulationRequest> {
        let mut pending = self.pending_requests.lock().unwrap();
        
        if let Some(priority_request) = pending.pop() {
            let request = priority_request.request;
            
            // Mark as processing
            let mut processing = self.processing_requests.lock().unwrap();
            processing.insert(request.request_id.clone());
            
            Some(request)
        } else {
            None
        }
    }

    /// Mark a request as completed (remove from processing set)
    pub fn mark_completed(&self, request_id: &str) {
        let mut processing = self.processing_requests.lock().unwrap();
        processing.remove(request_id);

        // Remove from deduplication map
        let mut dedup_map = self.deduplication_map.lock().unwrap();
        dedup_map.retain(|_, existing_id| existing_id != request_id);
    }

    /// Cancel a pending request
    pub fn cancel_request(&self, request_id: &str) -> Result<(), QueueError> {
        // Remove from pending queue (need to rebuild since BinaryHeap doesn't support removal)
        let mut pending = self.pending_requests.lock().unwrap();
        let mut new_pending = BinaryHeap::new();
        let mut found = false;

        while let Some(priority_request) = pending.pop() {
            if priority_request.request.request_id == request_id {
                found = true;
                // Don't add it back to the new heap
            } else {
                new_pending.push(priority_request);
            }
        }

        *pending = new_pending;

        if found {
            // Remove from deduplication map
            let mut dedup_map = self.deduplication_map.lock().unwrap();
            dedup_map.retain(|_, existing_id| existing_id != request_id);
            Ok(())
        } else {
            // Check if it's being processed
            let processing = self.processing_requests.lock().unwrap();
            if processing.contains(request_id) {
                Err(QueueError::RequestAlreadyProcessing)
            } else {
                Err(QueueError::RequestNotFound)
            }
        }
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> QueueStats {
        let pending = self.pending_requests.lock().unwrap();
        let processing = self.processing_requests.lock().unwrap();
        let dedup_map = self.deduplication_map.lock().unwrap();

        let mut priority_counts = HashMap::new();
        for priority_request in pending.iter() {
            *priority_counts.entry(priority_request.request.priority).or_insert(0) += 1;
        }

        QueueStats {
            pending_count: pending.len(),
            processing_count: processing.len(),
            total_capacity: self.max_queue_size,
            priority_counts,
            deduplication_cache_size: dedup_map.len(),
        }
    }

    /// Clear all pending requests
    pub fn clear_pending(&self) {
        let mut pending = self.pending_requests.lock().unwrap();
        pending.clear();

        // Clear deduplication map for pending requests only
        let processing = self.processing_requests.lock().unwrap();
        let mut dedup_map = self.deduplication_map.lock().unwrap();
        dedup_map.retain(|_, existing_id| !processing.contains(existing_id));
    }

    /// Check if a specific request is pending
    pub fn is_pending(&self, request_id: &str) -> bool {
        let pending = self.pending_requests.lock().unwrap();
        pending.iter().any(|pr| pr.request.request_id == request_id)
    }

    /// Check if a specific request is being processed
    pub fn is_processing(&self, request_id: &str) -> bool {
        let processing = self.processing_requests.lock().unwrap();
        processing.contains(request_id)
    }

    /// Get the next request without removing it from the queue
    pub fn peek_next(&self) -> Option<SimulationRequest> {
        let pending = self.pending_requests.lock().unwrap();
        pending.peek().map(|pr| pr.request.clone())
    }

    /// Get all pending requests (for debugging/monitoring)
    pub fn get_pending_requests(&self) -> Vec<SimulationRequest> {
        let pending = self.pending_requests.lock().unwrap();
        pending.iter().map(|pr| pr.request.clone()).collect()
    }

    /// Get all processing request IDs
    pub fn get_processing_request_ids(&self) -> Vec<String> {
        let processing = self.processing_requests.lock().unwrap();
        processing.iter().cloned().collect()
    }
}

impl Default for SimulationQueue {
    fn default() -> Self {
        Self::new(1000) // Default max queue size of 1000
    }
}

/// Statistics about the queue state
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of requests waiting to be processed
    pub pending_count: usize,
    /// Number of requests currently being processed
    pub processing_count: usize,
    /// Maximum queue capacity
    pub total_capacity: usize,
    /// Count of requests by priority level
    pub priority_counts: HashMap<SimulationPriority, usize>,
    /// Size of deduplication cache
    pub deduplication_cache_size: usize,
}

/// Errors that can occur during queue operations
#[derive(Debug, Clone)]
pub enum QueueError {
    /// Queue is at maximum capacity
    QueueFull,
    /// Request with same parameters already exists
    DuplicateRequest(String),
    /// Request not found
    RequestNotFound,
    /// Request is already being processed
    RequestAlreadyProcessing,
    /// Invalid request parameters
    InvalidRequest(String),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::QueueFull => write!(f, "Queue is at maximum capacity"),
            QueueError::DuplicateRequest(id) => write!(f, "Duplicate request already exists: {}", id),
            QueueError::RequestNotFound => write!(f, "Request not found"),
            QueueError::RequestAlreadyProcessing => write!(f, "Request is already being processed"),
            QueueError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for QueueError {}

/// Configuration for queue manager
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueManagerConfig {
    /// Maximum number of concurrent simulations
    pub max_concurrent_simulations: usize,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Default priority for new requests
    pub default_priority: SimulationPriority,
    /// Whether to enable deduplication
    pub enable_deduplication: bool,
    /// Queue processing interval in milliseconds
    pub processing_interval_ms: u64,
}

impl Default for QueueManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_simulations: 3,
            max_queue_size: 1000,
            default_priority: SimulationPriority::Normal,
            enable_deduplication: true,
            processing_interval_ms: 100,
        }
    }
}

/// High-level manager for simulation queues
pub struct QueueManager {
    /// The actual queue
    queue: SimulationQueue,
    /// Configuration
    config: QueueManagerConfig,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new(config: QueueManagerConfig) -> Self {
        Self {
            queue: SimulationQueue::new(config.max_queue_size),
            config,
        }
    }

    /// Add a simulation request to the queue
    pub fn enqueue(&self, request: SimulationRequest) -> Result<(), QueueError> {
        self.queue.enqueue(request)
    }

    /// Get the next simulation request
    pub fn dequeue(&self) -> Option<SimulationRequest> {
        self.queue.dequeue()
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> QueueStats {
        self.queue.get_stats()
    }

    /// Update configuration
    pub fn update_config(&mut self, config: QueueManagerConfig) {
        self.config = config;
        // Note: In a real implementation, you might want to recreate the queue
        // if the max_queue_size changes
    }

    /// Get current configuration
    pub fn get_config(&self) -> &QueueManagerConfig {
        &self.config
    }

    /// Clear all pending requests
    pub fn clear_pending(&self) {
        self.queue.clear_pending();
    }

    /// Cancel a specific request
    pub fn cancel_request(&self, request_id: &str) -> Result<(), QueueError> {
        self.queue.cancel_request(request_id)
    }

    /// Mark a request as completed
    pub fn mark_completed(&self, request_id: &str) {
        self.queue.mark_completed(request_id);
    }

    /// Check if a request is pending
    pub fn is_pending(&self, request_id: &str) -> bool {
        self.queue.is_pending(request_id)
    }

    /// Check if a request is being processed
    pub fn is_processing(&self, request_id: &str) -> bool {
        self.queue.is_processing(request_id)
    }

    /// Get all pending requests
    pub fn get_pending_requests(&self) -> Vec<SimulationRequest> {
        self.queue.get_pending_requests()
    }

    /// Get all processing request IDs
    pub fn get_processing_request_ids(&self) -> Vec<String> {
        self.queue.get_processing_request_ids()
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new(QueueManagerConfig::default())
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

    fn create_test_parameters(iterations: usize) -> ScenarioParameters {
        ScenarioParameters {
            players: vec![create_test_creature("Player1", 10.0, 15.0)],
            encounters: vec![],
            iterations,
        }
    }

    #[test]
    fn test_queue_basic_operations() {
        let queue = SimulationQueue::new(10);
        
        // Initially empty
        assert!(queue.dequeue().is_none());
        assert_eq!(queue.get_stats().pending_count, 0);

        // Add a request
        let request = SimulationRequest::new(create_test_parameters(100), SimulationPriority::Normal);
        queue.enqueue(request).unwrap();

        // Should have one pending
        assert_eq!(queue.get_stats().pending_count, 1);

        // Dequeue it
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.parameters.iterations, 100);

        // Should be empty again
        assert_eq!(queue.get_stats().pending_count, 0);
        assert_eq!(queue.get_stats().processing_count, 1);
    }

    #[test]
    fn test_priority_ordering() {
        let queue = SimulationQueue::new(10);
        
        // Add requests in different order
        let low = SimulationRequest::new(create_test_parameters(10), SimulationPriority::Low);
        let high = SimulationRequest::new(create_test_parameters(20), SimulationPriority::High);
        let normal = SimulationRequest::new(create_test_parameters(30), SimulationPriority::Normal);

        queue.enqueue(low).unwrap();
        queue.enqueue(high).unwrap();
        queue.enqueue(normal).unwrap();

        // Should get high priority first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.parameters.iterations, 20);

        // Then normal
        let second = queue.dequeue().unwrap();
        assert_eq!(second.parameters.iterations, 30);

        // Then low
        let third = queue.dequeue().unwrap();
        assert_eq!(third.parameters.iterations, 10);
    }

    #[test]
    fn test_deduplication() {
        let queue = SimulationQueue::new(10);
        
        let params1 = create_test_parameters(100);
        let params2 = create_test_parameters(100); // Same parameters
        
        let request1 = SimulationRequest::new(params1.clone(), SimulationPriority::Normal);
        let request2 = SimulationRequest::new(params2, SimulationPriority::High);

        // First should succeed
        queue.enqueue(request1).unwrap();

        // Second should fail due to deduplication
        let result = queue.enqueue(request2);
        assert!(matches!(result, Err(QueueError::DuplicateRequest(_))));
    }

    #[test]
    fn test_cancellation() {
        let queue = SimulationQueue::new(10);
        
        let request = SimulationRequest::new(create_test_parameters(100), SimulationPriority::Normal);
        let request_id = request.request_id.clone();
        
        queue.enqueue(request).unwrap();
        
        // Cancel before processing
        assert!(queue.cancel_request(&request_id).is_ok());
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_queue_capacity() {
        let queue = SimulationQueue::new(2);
        
        // Fill to capacity
        queue.enqueue(SimulationRequest::new(create_test_parameters(10), SimulationPriority::Normal)).unwrap();
        queue.enqueue(SimulationRequest::new(create_test_parameters(20), SimulationPriority::Normal)).unwrap();
        
        // Should fail when over capacity
        let result = queue.enqueue(SimulationRequest::new(create_test_parameters(30), SimulationPriority::Normal));
        assert!(matches!(result, Err(QueueError::QueueFull)));
    }

    #[test]
    fn test_request_completion() {
        let queue = SimulationQueue::new(10);
        
        let request = SimulationRequest::new(create_test_parameters(100), SimulationPriority::Normal);
        let request_id = request.request_id.clone();
        
        queue.enqueue(request).unwrap();
        let _dequeued = queue.dequeue().unwrap();
        
        // Should be processing
        assert!(queue.is_processing(&request_id));
        assert!(!queue.is_pending(&request_id));
        
        // Mark as completed
        queue.mark_completed(&request_id);
        
        // Should no longer be processing
        assert!(!queue.is_processing(&request_id));
    }
}