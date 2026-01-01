use crate::model::SimulationResult;
use crate::user_interaction::ScenarioParameters;
use std::sync::{Arc, Mutex, mpsc, PoisonError};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Unique identifier for a background simulation
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BackgroundSimulationId(pub String);

impl BackgroundSimulationId {
    pub fn new() -> Self {
        Self(format!("sim_{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()))
    }
    
    pub fn from_string(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            Err("Simulation ID string cannot be empty".to_string())
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

/// Priority level for simulation requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum SimulationPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for SimulationPriority {
    fn default() -> Self {
        SimulationPriority::Normal
    }
}

/// Progress information for a running simulation
#[derive(Debug, Clone)]
pub struct SimulationProgress {
    /// Unique identifier for this simulation
    pub simulation_id: BackgroundSimulationId,
    /// Current progress percentage (0.0 to 1.0)
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
}

impl SimulationProgress {
    pub fn new(simulation_id: BackgroundSimulationId, total_iterations: usize) -> Self {
        Self {
            simulation_id,
            progress_percentage: 0.0,
            iterations_completed: 0,
            total_iterations,
            estimated_time_remaining_ms: None,
            current_phase: "Initializing".to_string(),
            messages: Vec::new(),
        }
    }

    pub fn update_progress(&mut self, completed: usize, phase: &str) {
        self.iterations_completed = completed;
        self.progress_percentage = if self.total_iterations > 0 {
            completed as f64 / self.total_iterations as f64
        } else {
            1.0
        };
        self.current_phase = phase.to_string();
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }
}

/// A background simulation that runs in a separate thread
#[cfg(not(target_arch = "wasm32"))]
pub struct BackgroundSimulation {
    /// Unique identifier
    pub id: BackgroundSimulationId,
    /// Simulation parameters
    pub parameters: ScenarioParameters,
    /// Current progress
    pub progress: Arc<Mutex<SimulationProgress>>,
    /// When the simulation was started
    pub start_time: Instant,
    /// Priority level
    pub priority: SimulationPriority,
    /// Whether this simulation should be cancelled
    pub cancellation_requested: Arc<Mutex<bool>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl BackgroundSimulation {
    pub fn new(mut parameters: ScenarioParameters, priority: SimulationPriority) -> Self {
        parameters.iterations = parameters.iterations.max(100);
        let id = BackgroundSimulationId::new();
        let progress = Arc::new(Mutex::new(SimulationProgress::new(id.clone(), parameters.iterations)));

        Self {
            id: id.clone(),
            parameters,
            progress,
            start_time: Instant::now(),
            priority,
            cancellation_requested: Arc::new(Mutex::new(false)),
        }
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        *self.cancellation_requested.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Request cancellation of this simulation
    pub fn request_cancellation(&self) {
        *self.cancellation_requested.lock().unwrap_or_else(PoisonError::into_inner) = true;
    }

    /// Get current progress as a clone
    pub fn get_progress(&self) -> SimulationProgress {
        self.progress.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

/// Engine for managing and running background simulations
#[cfg(not(target_arch = "wasm32"))]
pub struct BackgroundSimulationEngine {
    /// Channel for sending progress updates
    progress_sender: mpsc::Sender<SimulationProgress>,
    /// Channel for receiving completed simulations
    completion_receiver: Arc<Mutex<mpsc::Receiver<BackgroundSimulationResult>>>,
    /// Handle to the worker thread
    worker_handle: Option<thread::JoinHandle<()>>,
}

/// Result of a completed background simulation
#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
pub struct BackgroundSimulationResult {
    /// Simulation identifier
    pub simulation_id: BackgroundSimulationId,
    /// Whether the simulation completed successfully
    pub success: bool,
    /// Results if successful
    pub results: Option<Vec<SimulationResult>>,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl BackgroundSimulationEngine {
    /// Create a new background simulation engine
    pub fn new() -> (Self, mpsc::Receiver<SimulationProgress>) {
        let (progress_sender, progress_receiver) = mpsc::channel();
        let (_completion_sender, completion_receiver) = mpsc::channel();

        let completion_receiver = Arc::new(Mutex::new(completion_receiver));

        let engine = Self {
            progress_sender,
            completion_receiver,
            worker_handle: None,
        };

        (engine, progress_receiver)
    }

    /// Start a background simulation
    pub fn start_simulation(
        &mut self,
        parameters: ScenarioParameters,
        priority: SimulationPriority,
    ) -> Result<BackgroundSimulationId, String> {
        let simulation = BackgroundSimulation::new(parameters.clone(), priority);
        let simulation_id = simulation.id.clone();
        let simulation_id_for_return = simulation_id.clone();

        // Clone necessary data for the thread
        let progress_clone = simulation.progress.clone();
        let cancellation_clone = simulation.cancellation_requested.clone();
        let progress_sender = self.progress_sender.clone();

        // Spawn worker thread
        let handle = thread::spawn(move || {
            Self::run_simulation_worker(
                simulation_id.clone(),
                parameters,
                progress_clone,
                cancellation_clone,
                progress_sender,
            )
        });

        self.worker_handle = Some(handle);

        Ok(simulation_id_for_return)
    }

    /// Worker function that runs the actual simulation
    #[cfg(not(target_arch = "wasm32"))]
    fn run_simulation_worker(
        _simulation_id: BackgroundSimulationId,
        mut parameters: ScenarioParameters,
        progress: Arc<Mutex<SimulationProgress>>,
        cancellation_requested: Arc<Mutex<bool>>,
        progress_sender: mpsc::Sender<SimulationProgress>,
    ) {
        parameters.iterations = parameters.iterations.max(100);
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut error_message: Option<String> = None;
        let mut success = false;

        // Update initial progress
        {
            let mut prog = progress.lock().unwrap_or_else(PoisonError::into_inner);
            prog.update_progress(0, "Starting simulation");
            prog.add_message("Simulation initialized".to_string());
        }
        
        // Send initial progress update
        if let Ok(prog) = progress.lock() {
            let _ = progress_sender.send(prog.clone());
        }

        // Run simulation with progress tracking
        for i in 0..parameters.iterations {
            // Check for cancellation
            if *cancellation_requested.lock().unwrap_or_else(PoisonError::into_inner) {
                error_message = Some("Simulation cancelled by user".to_string());
                break;
            }

            // Update progress periodically
            if i % 10 == 0 || i == parameters.iterations - 1 {
                {
                    let mut prog = progress.lock().unwrap_or_else(PoisonError::into_inner);
                    prog.update_progress(i, format!("Running iteration {}/{}", i + 1, parameters.iterations).as_str());
                    
                    // Estimate time remaining
                    if i > 0 {
                        let elapsed = start_time.elapsed().as_millis() as u64;
                        let avg_time_per_iteration = elapsed / i as u64;
                        let remaining_iterations = parameters.iterations.saturating_sub(i);
                        prog.estimated_time_remaining_ms = Some(avg_time_per_iteration * remaining_iterations as u64);
                    }
                }
                
                // Send progress update
                if let Ok(prog) = progress.lock() {
                    let _ = progress_sender.send(prog.clone());
                }
            }

            // Run a single simulation iteration
            match Self::run_single_iteration(&parameters) {
                Ok(result) => results.push(result),
                Err(e) => {
                    error_message = Some(format!("Iteration {} failed: {}", i, e));
                    break;
                }
            }
        }

        // Create final result
        let _execution_time = start_time.elapsed().as_millis() as u64;
        
        if error_message.is_none() {
            success = true;
            
            // Update final progress
            {
                let mut prog = progress.lock().unwrap_or_else(PoisonError::into_inner);
                prog.update_progress(parameters.iterations, "Storing results");
                prog.add_message("Simulation completed successfully".to_string());
            }
            
            // Send final progress update
            if let Ok(prog) = progress.lock() {
                let _ = progress_sender.send(prog.clone());
            }

            // Simulation completed - no storage needed
        }

        // Note: In a real implementation, we'd send this through a completion channel
        // For now, the progress updates and storage integration are the main focus
        
        // Update final progress with completion status
        {
            let mut prog = progress.lock().unwrap_or_else(PoisonError::into_inner);
            if success {
                prog.current_phase = "Completed".to_string();
                prog.progress_percentage = 1.0;
            } else {
                prog.current_phase = "Failed".to_string();
                if let Some(err) = &error_message {
                    prog.add_message(format!("Error: {}", err));
                }
            }
        }
        
        // Send final progress update
        if let Ok(prog) = progress.lock() {
            let _ = progress_sender.send(prog.clone());
        }
    }

    /// Run a single simulation iteration
    fn run_single_iteration(parameters: &ScenarioParameters) -> Result<SimulationResult, String> {
        // Use the existing simulation engine
        let runs = crate::run_event_driven_simulation_rust(
            parameters.players.clone(),
            parameters.timeline.clone(),
            1, // Single iteration
            false, // log_enabled
            None,
        );
        
        runs.into_iter()
            .next()
            .map(|run| run.result)
            .ok_or_else(|| "No simulation result generated".to_string())
    }

    /// Check for completed simulations (non-blocking)
    pub fn try_receive_completed(&self) -> Option<BackgroundSimulationResult> {
        if let Ok(receiver) = self.completion_receiver.try_lock() {
            match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Wait for a completed simulation (blocking)
    pub fn wait_for_completed(&self) -> BackgroundSimulationResult {
        let receiver = self.completion_receiver.lock().unwrap_or_else(PoisonError::into_inner);
        receiver.recv().unwrap()
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
    fn test_background_simulation_creation() {
        let parameters = ScenarioParameters {
            players: vec![create_test_creature("Player1", 10.0, 15.0)],
            timeline: vec![],
            iterations: 100,
        };

        let simulation = BackgroundSimulation::new(parameters, SimulationPriority::High);
        
        assert_eq!(simulation.priority, SimulationPriority::High);
        assert_eq!(simulation.parameters.iterations, 100);
        assert!(!simulation.is_cancelled());
    }

    #[test]
    fn test_simulation_progress() {
        let id = BackgroundSimulationId::new();
        let mut progress = SimulationProgress::new(id.clone(), 100);
        
        assert_eq!(progress.progress_percentage, 0.0);
        assert_eq!(progress.iterations_completed, 0);
        
        progress.update_progress(50, "Running");
        assert_eq!(progress.progress_percentage, 0.5);
        assert_eq!(progress.iterations_completed, 50);
        assert_eq!(progress.current_phase, "Running");
    }

    #[test]
    fn test_cancellation() {
        let parameters = ScenarioParameters {
            players: vec![create_test_creature("Player1", 10.0, 15.0)],
            timeline: vec![],
            iterations: 100,
        };

        let simulation = BackgroundSimulation::new(parameters, SimulationPriority::Normal);
        
        assert!(!simulation.is_cancelled());
        simulation.request_cancellation();
        assert!(simulation.is_cancelled());
    }
}