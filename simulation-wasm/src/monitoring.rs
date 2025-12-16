use crate::error_handling::{ErrorLogger, SimulationError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub total_iterations: usize,
    pub successful_iterations: usize,
    pub failed_iterations: usize,
    pub retried_iterations: usize,
    pub error_types: HashMap<String, usize>,
    pub average_execution_time: f64,
    pub memory_usage_peak: usize,
    pub start_time: u64,
    pub end_time: u64,
}

impl SimulationMetrics {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            total_iterations: 0,
            successful_iterations: 0,
            failed_iterations: 0,
            retried_iterations: 0,
            error_types: HashMap::new(),
            average_execution_time: 0.0,
            memory_usage_peak: 0,
            start_time: now,
            end_time: now,
        }
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_iterations == 0 {
            0.0
        } else {
            self.successful_iterations as f64 / self.total_iterations as f64
        }
    }
    
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
    
    pub fn retry_rate(&self) -> f64 {
        if self.failed_iterations == 0 {
            0.0
        } else {
            self.retried_iterations as f64 / self.failed_iterations as f64
        }
    }
    
    pub fn reliability_score(&self) -> f64 {
        // Complex score considering success rate, retries, error types
        let base_score = self.success_rate();
        let retry_penalty = (self.retried_iterations as f64 / self.total_iterations as f64) * 0.1;
        let error_penalty = (self.error_types.len() as f64 * 0.05);
        
        (base_score - retry_penalty - error_penalty).max(0.0)
    }
    
    pub fn mark_completed(&mut self) {
        self.end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.end_time.saturating_sub(self.start_time))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Alert {
    LowSuccessRate(f64),
    HighRetryRate,
    TooManyErrorTypes(usize),
    SlowExecution(f64),
    HighMemoryUsage(usize),
    SimulationStalled,
    UnexpectedErrorPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub min_success_rate: f64,      // Default: 0.95 (95%)
    pub max_retry_rate: f64,        // Default: 0.10 (10%)
    pub max_error_types: usize,     // Default: 5
    pub max_execution_time: f64,    // Default: 30s per iteration
    pub max_memory_usage: usize,    // Default: 1GB
    pub max_stall_time: u64,         // Default: 60s
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_success_rate: 0.95,
            max_retry_rate: 0.10,
            max_error_types: 5,
            max_execution_time: 30.0,
            max_memory_usage: 1024 * 1024 * 1024, // 1GB
            max_stall_time: 60,
        }
    }
}

impl AlertThresholds {
    pub fn check_alerts(&self, metrics: &SimulationMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        if metrics.success_rate() < self.min_success_rate {
            alerts.push(Alert::LowSuccessRate(metrics.success_rate()));
        }
        
        if metrics.retry_rate() > self.max_retry_rate {
            alerts.push(Alert::HighRetryRate);
        }
        
        if metrics.error_types.len() > self.max_error_types {
            alerts.push(Alert::TooManyErrorTypes(metrics.error_types.len()));
        }
        
        if metrics.average_execution_time > self.max_execution_time {
            alerts.push(Alert::SlowExecution(metrics.average_execution_time));
        }
        
        if metrics.memory_usage_peak > self.max_memory_usage {
            alerts.push(Alert::HighMemoryUsage(metrics.memory_usage_peak));
        }
        
        // Check for stall (no progress for too long)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if now > metrics.end_time && (now - metrics.end_time) > self.max_stall_time {
            alerts.push(Alert::SimulationStalled);
        }
        
        alerts
    }
}

pub struct SimulationMonitor {
    metrics: SimulationMetrics,
    thresholds: AlertThresholds,
    start_time: Instant,
    alerts: Vec<Alert>,
    error_logger: ErrorLogger,
}

impl SimulationMonitor {
    pub fn new(thresholds: AlertThresholds) -> Self {
        Self {
            metrics: SimulationMetrics::new(),
            thresholds,
            start_time: Instant::now(),
            alerts: Vec::new(),
            error_logger: ErrorLogger::new(1000),
        }
    }
    
    pub fn start_iteration(&mut self) {
        self.metrics.total_iterations += 1;
    }
    
    pub fn record_success(&mut self) {
        self.metrics.successful_iterations += 1;
    }
    
    pub fn record_failure(&mut self, error: &SimulationError) {
        self.metrics.failed_iterations += 1;
        
        // Track error types
        let error_type = match error {
            SimulationError::EmptyResult(_) => "EmptyResult",
            SimulationError::InvalidCombatant(_) => "InvalidCombatant",
            SimulationError::ResourceExhausted(_) => "ResourceExhausted",
            SimulationError::IndexOutOfBounds(_) => "IndexOutOfBounds",
            SimulationError::SerializationError(_) => "SerializationError",
            SimulationError::UnexpectedState(_) => "UnexpectedState",
            SimulationError::ValidationFailed(_) => "ValidationFailed",
            SimulationError::RetryExhausted(_) => "RetryExhausted",
        };
        
        *self.metrics.error_types.entry(error_type.to_string()).or_insert(0) += 1;
    }
    
    pub fn record_retry(&mut self) {
        self.metrics.retried_iterations += 1;
    }
    
    pub fn update_execution_time(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if self.metrics.total_iterations > 0 {
            self.metrics.average_execution_time = elapsed / self.metrics.total_iterations as f64;
        }
    }
    
    pub fn update_memory_usage(&mut self, usage: usize) {
        if usage > self.metrics.memory_usage_peak {
            self.metrics.memory_usage_peak = usage;
        }
    }
    
    pub fn check_alerts(&mut self) -> &[Alert] {
        self.update_execution_time();
        let new_alerts = self.thresholds.check_alerts(&self.metrics);
        
        // Only add new alerts that weren't already present
        for alert in new_alerts {
            if !self.alerts.iter().any(|existing| std::mem::discriminant(existing) == std::mem::discriminant(&alert)) {
                self.alerts.push(alert);
            }
        }
        
        &self.alerts
    }
    
    pub fn get_metrics(&self) -> &SimulationMetrics {
        &self.metrics
    }
    
    pub fn get_metrics_mut(&mut self) -> &mut SimulationMetrics {
        &mut self.metrics
    }
    
    pub fn finalize(&mut self) -> SimulationMetrics {
        self.metrics.mark_completed();
        self.update_execution_time();
        self.metrics.clone()
    }
    
    pub fn get_error_summary(&self) -> HashMap<String, usize> {
        self.error_logger.get_error_summary()
    }
    
    pub fn is_healthy(&self) -> bool {
        self.alerts.is_empty() && self.metrics.success_rate() >= self.thresholds.min_success_rate
    }
    
    pub fn get_health_status(&self) -> HealthStatus {
        if self.is_healthy() {
            HealthStatus::Healthy
        } else if self.metrics.success_rate() >= 0.8 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Critical
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "Healthy",
            HealthStatus::Degraded => "Degraded",
            HealthStatus::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub metrics: SimulationMetrics,
    pub alerts: Vec<Alert>,
    pub health_status: HealthStatus,
    pub error_summary: HashMap<String, usize>,
    pub recommendations: Vec<String>,
}

impl MonitoringReport {
    pub fn generate(monitor: &mut SimulationMonitor) -> Self {
        let metrics = monitor.get_metrics().clone();
        let alerts = monitor.check_alerts().to_vec();
        let health_status = monitor.get_health_status();
        let error_summary = monitor.get_error_summary();
        let recommendations = Self::generate_recommendations(&metrics, &alerts);
        
        Self {
            metrics,
            alerts,
            health_status,
            error_summary,
            recommendations,
        }
    }
    
    fn generate_recommendations(metrics: &SimulationMetrics, alerts: &[Alert]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if metrics.success_rate() < 0.95 {
            recommendations.push(format!(
                "Success rate ({:.1}%) below target. Consider reviewing scenario complexity.",
                metrics.success_rate() * 100.0
            ));
        }
        
        if metrics.retry_rate() > 0.1 {
            recommendations.push(format!(
                "High retry rate ({:.1}%). Consider investigating transient failures.",
                metrics.retry_rate() * 100.0
            ));
        }
        
        if metrics.error_types.len() > 5 {
            recommendations.push(format!(
                "Too many error types ({}). Consider comprehensive error handling review.",
                metrics.error_types.len()
            ));
        }
        
        if metrics.average_execution_time > 30.0 {
            recommendations.push(format!(
                "Slow execution time ({:.1}s per iteration). Consider performance optimization.",
                metrics.average_execution_time
            ));
        }
        
        for alert in alerts {
            match alert {
                Alert::LowSuccessRate(rate) => {
                    recommendations.push(format!(
                        "Critical: Success rate ({:.1}%) is critically low.",
                        rate * 100.0
                    ));
                }
                Alert::HighRetryRate => {
                    recommendations.push("Warning: High retry rate detected.".to_string());
                }
                Alert::TooManyErrorTypes(count) => {
                    recommendations.push(format!(
                        "Warning: {} different error types detected.",
                        count
                    ));
                }
                Alert::SlowExecution(time) => {
                    recommendations.push(format!(
                        "Performance: Execution time ({:.1}s) exceeds threshold.",
                        time
                    ));
                }
                Alert::HighMemoryUsage(usage) => {
                    recommendations.push(format!(
                        "Memory: Usage ({:.1}MB) exceeds threshold.",
                        *usage as f64 / (1024.0 * 1024.0)
                    ));
                }
                Alert::SimulationStalled => {
                    recommendations.push("Critical: Simulation appears to be stalled.".to_string());
                }
                Alert::UnexpectedErrorPattern => {
                    recommendations.push("Warning: Unexpected error pattern detected.".to_string());
                }
            }
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulation_metrics() {
        let mut metrics = SimulationMetrics::new();
        
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.failure_rate(), 1.0);
        assert_eq!(metrics.reliability_score(), 0.0);
        
        metrics.total_iterations = 100;
        metrics.successful_iterations = 95;
        metrics.failed_iterations = 5;
        metrics.retried_iterations = 2;
        
        assert_eq!(metrics.success_rate(), 0.95);
        assert_eq!(metrics.failure_rate(), 0.05);
        assert_eq!(metrics.retry_rate(), 0.4);
        assert!(metrics.reliability_score() > 0.79); // Allow for floating point precision
    }
    
    #[test]
    fn test_alert_thresholds() {
        let thresholds = AlertThresholds::default();
        
        let mut metrics = SimulationMetrics::new();
        metrics.total_iterations = 100;
        metrics.successful_iterations = 90; // 90% success rate
        metrics.error_types.insert("EmptyResult".to_string(), 5);
        metrics.error_types.insert("InvalidCombatant".to_string(), 5);
        metrics.error_types.insert("ResourceExhausted".to_string(), 5);
        metrics.error_types.insert("IndexOutOfBounds".to_string(), 5);
        metrics.error_types.insert("SerializationError".to_string(), 5);
        metrics.error_types.insert("UnexpectedState".to_string(), 5);
        
        let alerts = thresholds.check_alerts(&metrics);
        
        // Should have alert for low success rate
        assert!(alerts.iter().any(|a| matches!(a, Alert::LowSuccessRate(_))));
        
        // Should have alert for too many error types
        assert!(alerts.iter().any(|a| matches!(a, Alert::TooManyErrorTypes(_))));
    }
    
    #[test]
    fn test_simulation_monitor() {
        let thresholds = AlertThresholds::default();
        let mut monitor = SimulationMonitor::new(thresholds);
        
        // Simulate some iterations
        for i in 0..100 {
            monitor.start_iteration();
            if i < 95 {
                monitor.record_success();
            } else {
                monitor.record_failure(&SimulationError::EmptyResult("Test".to_string()));
            }
        }
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_iterations, 100);
        assert_eq!(metrics.successful_iterations, 95);
        assert_eq!(metrics.failed_iterations, 5);
        assert_eq!(metrics.success_rate(), 0.95);
        
        let health = monitor.get_health_status();
        assert!(matches!(health, HealthStatus::Healthy));
    }
    
    #[test]
    fn test_monitoring_report() {
        let thresholds = AlertThresholds::default();
        let mut monitor = SimulationMonitor::new(thresholds);
        
        // Add some data
        for i in 0..100 {
            monitor.start_iteration();
            if i < 90 {
                monitor.record_success();
            } else {
                monitor.record_failure(&SimulationError::EmptyResult("Test".to_string()));
            }
        }
        
        let report = MonitoringReport::generate(&mut monitor);
        
        assert_eq!(report.metrics.total_iterations, 100);
        assert_eq!(report.metrics.successful_iterations, 90);
        assert!(matches!(report.health_status, HealthStatus::Degraded));
        assert!(!report.recommendations.is_empty());
    }
}