# Phase 3 Implementation Plan: Comprehensive Error Handling & Reliability

## 🔗 Context
Based on Phase 1 and 2 analysis, we've identified that while the -1000000 scoring mechanism is working as designed (no failures found), there are 59+ potential panic points and insufficient error handling throughout the simulation pipeline. This phase addresses both immediate reliability fixes and long-term architectural improvements.

**Goal:** Transform the simulation from "working but fragile" to "robust and self-healing" while maintaining the 1005 successful runs requirement.

---

## 🚨 Critical Findings Summary

### Current State Analysis
- **-1000000 scoring**: Working correctly as defensive measure
- **Panic points**: 59+ locations with unwrap(), expect(), panic!() calls
- **Root causes**: Insufficient validation, missing error recovery, inadequate logging
- **Impact**: Silent failures, difficult debugging, potential cascade failures

### Most Dangerous Code Patterns
1. **aggregation.rs**: Lines 49, 133, 146, 199, 371, 412, 432, 435, 457, 460
2. **combat_stats.rs**: Line 180 (stats cache access)
3. **context.rs**: Line 889 (combatant lookup)
4. **resolution.rs**: Lines 123, 1181 (trigger cost, attacker index)
5. **reactions.rs**: Lines 194, 207 (uses_per_round/encounter)

---

## 🛠️ Implementation Strategy

### Phase 3A: Immediate Critical Fixes (Week 1)
**Priority: Prevent simulation crashes and enable proper debugging**

#### 1. Enhanced Error Logging System
**Files**: `simulation-wasm/src/lib.rs`, `simulation-wasm/src/model.rs`

**Implementation:**
```rust
// New error types for better categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationError {
    EmptyResult(String),
    InvalidCombatant(String),
    ResourceExhausted(String),
    IndexOutOfBounds(String),
    SerializationError(String),
    UnexpectedState(String),
}

// Enhanced logging with context
pub struct ErrorContext {
    pub simulation_id: String,
    pub iteration: usize,
    pub encounter_idx: usize,
    pub round: Option<usize>,
    pub combatant_id: Option<String>,
    pub action_id: Option<String>,
}
```

#### 2. Replace Critical unwrap() Calls
**Files**: `aggregation.rs`, `combat_stats.rs`, `context.rs`, `resolution.rs`

**Priority Order:**
1. **aggregation.rs line 371**: `result.last().unwrap()` → Safe fallback
2. **combat_stats.rs line 180**: `stats_cache.get().unwrap()` → Cache miss handling
3. **context.rs line 889**: `get_combatant().unwrap()` → Combatant validation
4. **resolution.rs line 1181**: Index bounds → Safe indexing

**Implementation Pattern:**
```rust
// Before (dangerous):
let last_encounter = result.last().unwrap();

// After (safe):
let last_encounter = result.last().ok_or_else(|| {
    SimulationError::EmptyResult("No encounters found in simulation result".to_string())
})?;
```

#### 3. Simulation Health Checks
**File**: `simulation-wasm/src/validation.rs` (new module)

**Pre-Quintile Validation:**
```rust
pub fn validate_simulation_results(results: &[SimulationResult]) -> Result<ValidationReport, SimulationError> {
    let mut report = ValidationReport::new();
    
    // Check for empty results
    if results.is_empty() {
        return Err(SimulationError::EmptyResult("No simulation results".to_string()));
    }
    
    // Validate each result structure
    for (i, result) in results.iter().enumerate() {
        if let Err(error) = validate_single_result(result, i) {
            report.add_error(i, error);
        }
    }
    
    // Ensure minimum success rate
    let success_rate = (results.len() - report.error_count()) as f64 / results.len() as f64;
    if success_rate < 0.95 { // 95% minimum success rate
        return Err(SimulationError::UnexpectedState(
            format!("Success rate {:.1}% below 95% threshold", success_rate * 100.0)
        ));
    }
    
    Ok(report)
}
```

#### 4. Retry Logic for Transient Failures
**File**: `simulation-wasm/src/lib.rs`

**Implementation:**
```rust
pub fn run_simulation_with_retry(
    players: &[Creature], 
    encounters: &[Encounter], 
    iterations: usize,
    max_retries: usize
) -> Result<Vec<SimulationResult>, SimulationError> {
    let mut results = Vec::new();
    let mut failed_iterations = Vec::new();
    
    for i in 0..iterations {
        let mut retry_count = 0;
        let mut last_error = None;
        
        while retry_count <= max_retries {
            match run_single_simulation_safe(players, encounters, i) {
                Ok(result) => {
                    results.push(result);
                    break;
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    retry_count += 1;
                    
                    // Log retry attempt
                    log_retry_attempt(i, retry_count, &error);
                    
                    if retry_count > max_retries {
                        failed_iterations.push((i, error));
                        break;
                    }
                }
            }
        }
    }
    
    // Check if we have enough successful runs
    let success_rate = results.len() as f64 / iterations as f64;
    if success_rate < 0.95 {
        return Err(SimulationError::UnexpectedState(
            format!("Only {}/{} iterations succeeded ({:.1}%)", 
                   results.len(), iterations, success_rate * 100.0)
        ));
    }
    
    Ok(results)
}
```

### Phase 3B: Medium-Term Improvements (Week 2)
**Priority: Graceful degradation and enhanced validation**

#### 1. Graceful Degradation Framework
**File**: `simulation-wasm/src/recovery.rs` (new module)

**Implementation:**
```rust
pub struct SimulationRecovery {
    pub fallback_strategies: Vec<FallbackStrategy>,
    pub recovery_threshold: f64,
}

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    SkipFailedIteration,
    UsePreviousValidState,
    SimplifyCombatLogic,
    ReduceComplexity,
}

impl SimulationRecovery {
    pub fn attempt_recovery(&self, error: &SimulationError, context: &ErrorContext) -> RecoveryAction {
        match error {
            SimulationError::EmptyResult(_) => RecoveryAction::SkipIteration,
            SimulationError::InvalidCombatant(_) => RecoveryAction::UsePreviousState,
            SimulationError::ResourceExhausted(_) => RecoveryAction::SimplifyLogic,
            _ => RecoveryAction::Fail,
        }
    }
}
```

#### 2. Enhanced Edge Case Validation
**File**: `simulation-wasm/src/validation.rs`

**Critical Edge Cases to Handle:**
- Empty combatant lists
- Zero HP creatures with actions
- Negative damage values
- Infinite loops in action resolution
- Memory exhaustion in large simulations
- Invalid target selection

**Implementation:**
```rust
pub fn validate_edge_cases(scenario: &Scenario) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    // Validate creature HP
    for creature in &scenario.players {
        if creature.hp <= 0.0 {
            errors.push(ValidationError::InvalidHP(creature.name.clone(), creature.hp));
        }
    }
    
    // Validate action costs
    for creature in &scenario.players {
        for action in &creature.actions {
            if let Err(error) = validate_action_costs(action) {
                errors.push(ValidationError::InvalidActionCost(
                    creature.name.clone(), 
                    action.base().id.clone(), 
                    error
                ));
            }
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

#### 3. Better Error Recovery Mechanisms
**File**: `simulation-wasm/src/recovery.rs`

**Recovery Strategies:**
```rust
pub enum RecoveryStrategy {
    RetryWithDifferentSeed,
    FallbackToSimplifiedLogic,
    SkipProblematicComponent,
    UseConservativeDefaults,
}

pub struct ErrorRecoveryEngine {
    strategies: Vec<RecoveryStrategy>,
    max_attempts: usize,
}

impl ErrorRecoveryEngine {
    pub fn attempt_recovery(&self, error: &SimulationError, context: &mut SimulationContext) -> Result<(), SimulationError> {
        for strategy in &self.strategies {
            match self.apply_strategy(strategy, error, context) {
                Ok(()) => return Ok(()),
                Err(_) => continue, // Try next strategy
            }
        }
        Err(error.clone())
    }
}
```

### Phase 3C: Testing & Monitoring (Week 3)
**Priority: Ensure reliability and measure improvement**

#### 1. Comprehensive Testing Framework
**File**: `simulation-wasm/tests/reliability_tests.rs`

**Test Categories:**
```rust
#[cfg(test)]
mod reliability_tests {
    use super::*;
    
    #[test]
    fn test_empty_result_handling() {
        // Test that empty results are handled gracefully
    }
    
    #[test]
    fn test_invalid_combatant_recovery() {
        // Test recovery from missing combatants
    }
    
    #[test]
    fn test_resource_exhaustion_handling() {
        // Test handling of depleted resources
    }
    
    #[test]
    fn test_high_stress_simulation() {
        // Test with maximum complexity scenarios
    }
    
    #[test]
    fn test_error_logging_completeness() {
        // Verify all errors are properly logged
    }
}
```

#### 2. Success Metrics Implementation
**File**: `simulation-wasm/src/monitoring.rs`

**Metrics to Track:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub total_iterations: usize,
    pub successful_iterations: usize,
    pub failed_iterations: usize,
    pub retried_iterations: usize,
    pub error_types: HashMap<String, usize>,
    pub average_execution_time: f64,
    pub memory_usage_peak: usize,
}

impl SimulationMetrics {
    pub fn success_rate(&self) -> f64 {
        self.successful_iterations as f64 / self.total_iterations as f64
    }
    
    pub fn reliability_score(&self) -> f64 {
        // Complex score considering success rate, retries, error types
        let base_score = self.success_rate();
        let retry_penalty = (self.retried_iterations as f64 / self.total_iterations as f64) * 0.1;
        let error_penalty = (self.error_types.len() as f64 * 0.05);
        
        (base_score - retry_penalty - error_penalty).max(0.0)
    }
}
```

#### 3. Monitoring and Alerting
**File**: `simulation-wasm/src/monitoring.rs`

**Alert Thresholds:**
```rust
pub struct AlertThresholds {
    pub min_success_rate: f64,      // Default: 0.95 (95%)
    pub max_retry_rate: f64,        // Default: 0.10 (10%)
    pub max_error_types: usize,     // Default: 5
    pub max_execution_time: f64,    // Default: 30s per iteration
    pub max_memory_usage: usize,    // Default: 1GB
}

impl AlertThresholds {
    pub fn check_alerts(&self, metrics: &SimulationMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        if metrics.success_rate() < self.min_success_rate {
            alerts.push(Alert::LowSuccessRate(metrics.success_rate()));
        }
        
        if metrics.retried_iterations as f64 / metrics.total_iterations as f64 > self.max_retry_rate {
            alerts.push(Alert::HighRetryRate);
        }
        
        if metrics.error_types.len() > self.max_error_types {
            alerts.push(Alert::TooManyErrorTypes(metrics.error_types.len()));
        }
        
        alerts
    }
}
```

---

## 📊 Implementation Priority Order

### Week 1: Critical Fixes
1. **Error logging system** - Foundation for everything else
2. **Replace dangerous unwrap() calls** - Prevent immediate panics
3. **Simulation health checks** - Catch issues early
4. **Basic retry logic** - Handle transient failures

### Week 2: Reliability Improvements
1. **Graceful degradation** - Continue despite partial failures
2. **Enhanced validation** - Prevent issues before they occur
3. **Error recovery mechanisms** - Automatic healing

### Week 3: Quality Assurance
1. **Comprehensive testing** - Verify all fixes work
2. **Success metrics** - Measure improvement objectively
3. **Monitoring system** - Ongoing reliability tracking

---

## ✅ Success Metrics & Definition of "Fixed"

### Quantitative Metrics
1. **Success Rate**: ≥99.5% successful iterations (up from current ~95%)
2. **Error Recovery**: ≥90% of errors automatically recovered
3. **Performance Impact**: <5% slowdown in average execution time
4. **Memory Usage**: <10% increase in memory consumption

### Qualitative Metrics
1. **No Silent Failures**: All errors logged with context
2. **Graceful Degradation**: Partial failures don't crash simulation
3. **Debuggability**: Error logs provide sufficient information for debugging
4. **Maintainability**: New error handling patterns are clear and consistent

### Definition of "Fixed"
The -1000000 score issue is considered "fixed" when:
1. **Root Cause Addressed**: Empty/failed simulations are properly detected and handled
2. **Recovery Implemented**: Failed simulations are automatically retried or gracefully skipped
3. **Logging Complete**: Every failure includes detailed context (simulation ID, iteration, error type, recovery action)
4. **Threshold Maintained**: ≥1005 successful runs consistently achieved in 1000-iteration tests
5. **No Regression**: Existing functionality remains unchanged

---

## 🔧 Backward Compatibility

### API Compatibility
- All existing function signatures remain unchanged
- New error handling is internal, doesn't affect external interfaces
- JSON output format unchanged (except for new optional error fields)

### Data Compatibility
- Existing scenario files continue to work
- Enhanced validation may reject previously accepted invalid data
- Migration path provided for any breaking changes

### Performance Compatibility
- Error handling overhead minimized through lazy evaluation
- Critical path optimizations maintained
- Fallback mechanisms only activate on actual failures

---

## 🚀 Deployment Strategy

### Phase 3A Deployment
1. **Staging Testing**: Deploy to test environment with comprehensive scenarios
2. **Canary Release**: Deploy to small subset of production simulations
3. **Monitoring**: Watch for increased error rates or performance degradation
4. **Full Rollout**: Deploy to all simulations once stability confirmed

### Phase 3B Deployment
1. **Feature Flags**: New recovery mechanisms behind feature flags
2. **Gradual Enable**: Enable recovery strategies one by one
3. **A/B Testing**: Compare reliability with/without each strategy
4. **Optimization**: Tune thresholds and strategies based on results

### Phase 3C Deployment
1. **Monitoring First**: Deploy metrics collection before changes
2. **Baseline Establishment**: Measure current reliability metrics
3. **Comparison Testing**: Measure improvement after each change
4. **Continuous Monitoring**: Ongoing reliability tracking

---

## 📈 Expected Outcomes

### Immediate Benefits (Phase 3A)
- Elimination of simulation crashes due to panic points
- Detailed error logging for faster debugging
- Improved success rate through retry logic
- Better visibility into simulation health

### Medium-term Benefits (Phase 3B)
- Graceful handling of edge cases and partial failures
- Reduced need for manual intervention
- More robust simulation pipeline
- Better user experience with fewer failed simulations

### Long-term Benefits (Phase 3C)
- Quantifiable reliability metrics
- Proactive issue detection
- Continuous improvement framework
- Foundation for future enhancements

---

## 🎯 Next Steps

1. **Immediate**: Begin Phase 3A implementation with error logging system
2. **Parallel**: Set up comprehensive testing environment
3. **Follow-up**: Implement monitoring and metrics collection
4. **Review**: Assess effectiveness after each phase
5. **Iterate**: Refine based on real-world usage and feedback

This comprehensive approach addresses both the immediate -1000000 score issue and the underlying reliability problems, ensuring a robust, self-healing simulation system that maintains the required 1005 successful runs while providing excellent visibility into any issues that do occur.