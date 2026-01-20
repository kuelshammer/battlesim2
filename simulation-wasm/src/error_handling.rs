use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationError {
    EmptyResult(String),
    InvalidCombatant(String),
    ResourceExhausted(String),
    IndexOutOfBounds(String),
    SerializationError(String),
    UnexpectedState(String),
    ValidationFailed(String),
    RetryExhausted(String),
}

impl std::fmt::Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::EmptyResult(msg) => write!(f, "Empty result: {}", msg),
            SimulationError::InvalidCombatant(msg) => write!(f, "Invalid combatant: {}", msg),
            SimulationError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            SimulationError::IndexOutOfBounds(msg) => write!(f, "Index out of bounds: {}", msg),
            SimulationError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SimulationError::UnexpectedState(msg) => write!(f, "Unexpected state: {}", msg),
            SimulationError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            SimulationError::RetryExhausted(msg) => write!(f, "Retry exhausted: {}", msg),
        }
    }
}

impl std::error::Error for SimulationError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub simulation_id: String,
    pub iteration: usize,
    pub encounter_idx: usize,
    pub round: Option<usize>,
    pub combatant_id: Option<String>,
    pub action_id: Option<String>,
    pub timestamp: u64,
}

impl ErrorContext {
    pub fn new(simulation_id: String, iteration: usize, encounter_idx: usize) -> Self {
        Self {
            simulation_id,
            iteration,
            encounter_idx,
            round: None,
            combatant_id: None,
            action_id: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn with_round(mut self, round: usize) -> Self {
        self.round = Some(round);
        self
    }

    pub fn with_combatant(mut self, combatant_id: String) -> Self {
        self.combatant_id = Some(combatant_id);
        self
    }

    pub fn with_action(mut self, action_id: String) -> Self {
        self.action_id = Some(action_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub error: SimulationError,
    pub context: ErrorContext,
    pub recovery_attempted: bool,
    pub recovery_successful: bool,
    pub stack_trace: Option<String>,
}

impl ErrorLog {
    pub fn new(error: SimulationError, context: ErrorContext) -> Self {
        Self {
            error,
            context,
            recovery_attempted: false,
            recovery_successful: false,
            stack_trace: None,
        }
    }

    pub fn with_recovery(mut self, attempted: bool, successful: bool) -> Self {
        self.recovery_attempted = attempted;
        self.recovery_successful = successful;
        self
    }

    pub fn with_stack_trace(mut self, trace: String) -> Self {
        self.stack_trace = Some(trace);
        self
    }
}

pub struct ErrorLogger {
    logs: Vec<ErrorLog>,
    max_logs: usize,
}

impl ErrorLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::with_capacity(max_logs),
            max_logs,
        }
    }

    pub fn log_error(&mut self, error: SimulationError, context: ErrorContext) {
        let log = ErrorLog::new(error, context);

        #[cfg(debug_assertions)]
        eprintln!("SIMULATION_ERROR: {}", log.error);

        if self.logs.len() < self.max_logs {
            self.logs.push(log);
        } else {
            // Rotate logs if at capacity
            self.logs.remove(0);
            self.logs.push(log);
        }
    }

    pub fn log_recovery_attempt(
        &mut self,
        error: SimulationError,
        context: ErrorContext,
        successful: bool,
    ) {
        let log = ErrorLog::new(error, context).with_recovery(true, successful);

        #[cfg(debug_assertions)]
        eprintln!(
            "RECOVERY_{}: {}",
            if successful { "SUCCESS" } else { "FAILED" },
            log.error
        );

        if self.logs.len() < self.max_logs {
            self.logs.push(log);
        } else {
            self.logs.remove(0);
            self.logs.push(log);
        }
    }

    pub fn get_logs(&self) -> &[ErrorLog] {
        &self.logs
    }

    pub fn get_error_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();

        for log in &self.logs {
            let error_type = match &log.error {
                SimulationError::EmptyResult(_) => "EmptyResult",
                SimulationError::InvalidCombatant(_) => "InvalidCombatant",
                SimulationError::ResourceExhausted(_) => "ResourceExhausted",
                SimulationError::IndexOutOfBounds(_) => "IndexOutOfBounds",
                SimulationError::SerializationError(_) => "SerializationError",
                SimulationError::UnexpectedState(_) => "UnexpectedState",
                SimulationError::ValidationFailed(_) => "ValidationFailed",
                SimulationError::RetryExhausted(_) => "RetryExhausted",
            };

            *summary.entry(error_type.to_string()).or_insert(0) += 1;
        }

        summary
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }
}

// Global error logger instance
static GLOBAL_ERROR_LOGGER: OnceLock<Mutex<ErrorLogger>> = OnceLock::new();

pub fn get_global_logger() -> &'static Mutex<ErrorLogger> {
    GLOBAL_ERROR_LOGGER.get_or_init(|| Mutex::new(ErrorLogger::new(1000)))
}

pub fn log_simulation_error(error: SimulationError, context: ErrorContext) {
    if let Ok(mut logger) = get_global_logger().lock() {
        logger.log_error(error, context);
    }
}

pub fn log_recovery_attempt(error: SimulationError, context: ErrorContext, successful: bool) {
    if let Ok(mut logger) = get_global_logger().lock() {
        logger.log_recovery_attempt(error, context, successful);
    }
}

// Convenience functions for common error scenarios
pub fn log_empty_result(context: ErrorContext, details: &str) {
    log_simulation_error(
        SimulationError::EmptyResult(format!("Empty simulation result: {}", details)),
        context,
    );
}

pub fn log_invalid_combatant(context: ErrorContext, combatant_id: &str, details: &str) {
    log_simulation_error(
        SimulationError::InvalidCombatant(format!("Combatant '{}': {}", combatant_id, details)),
        context.with_combatant(combatant_id.to_string()),
    );
}

pub fn log_index_out_of_bounds(context: ErrorContext, index: usize, bounds: usize) {
    log_simulation_error(
        SimulationError::IndexOutOfBounds(format!("Index {} out of bounds (0..{})", index, bounds)),
        context,
    );
}

pub fn log_unexpected_state(context: ErrorContext, state: &str) {
    log_simulation_error(
        SimulationError::UnexpectedState(format!("Unexpected state: {}", state)),
        context,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_logger() {
        let mut logger = ErrorLogger::new(10);
        let context = ErrorContext::new("test_sim".to_string(), 0, 0);

        logger.log_error(
            SimulationError::EmptyResult("Test error".to_string()),
            context.clone(),
        );

        assert_eq!(logger.get_logs().len(), 1);

        let summary = logger.get_error_summary();
        assert_eq!(summary.get("EmptyResult"), Some(&1));
    }

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new("test_sim".to_string(), 5, 2)
            .with_round(3)
            .with_combatant("player1".to_string())
            .with_action("attack".to_string());

        assert_eq!(context.simulation_id, "test_sim");
        assert_eq!(context.iteration, 5);
        assert_eq!(context.encounter_idx, 2);
        assert_eq!(context.round, Some(3));
        assert_eq!(context.combatant_id, Some("player1".to_string()));
        assert_eq!(context.action_id, Some("attack".to_string()));
    }
}
