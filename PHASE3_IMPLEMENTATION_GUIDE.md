# Phase 3 Implementation Guide: Error Handling & Reliability

## 🚀 Quick Start Implementation

This guide provides step-by-step instructions for implementing the Phase 3 error handling and reliability improvements.

### Step 1: Update Module Imports

Add the new modules to your `lib.rs`:

```rust
pub mod error_handling; // Enhanced error handling system
pub mod enhanced_validation; // Comprehensive validation
pub mod recovery; // Error recovery mechanisms
pub mod safe_aggregation; // Safe aggregation functions
pub mod monitoring; // Success metrics and monitoring
```

### Step 2: Replace Dangerous unwrap() Calls

#### Critical Priority (Immediate)

**File: `aggregation.rs`**
```rust
// Replace line 371:
// OLD: let last_encounter = result.last().unwrap();
// NEW:
let last_encounter = result.last().ok_or_else(|| {
    SimulationError::EmptyResult("No encounters found in simulation result".to_string())
})?;
```

**File: `combat_stats.rs`**
```rust
// Replace line 180:
// OLD: self.stats_cache.get(creature_id).unwrap()
// NEW:
self.stats_cache.get(creature_id).ok_or_else(|| {
    SimulationError::InvalidCombatant(format!("Combatant '{}' not found in stats cache", creature_id))
})?
```

**File: `context.rs`**
```rust
// Replace line 889:
// OLD: let combatant = context.get_combatant("player1").unwrap();
// NEW:
let combatant = context.get_combatant("player1").ok_or_else(|| {
    SimulationError::InvalidCombatant("Combatant 'player1' not found".to_string())
})?
```

### Step 3: Add Error Logging to Simulation Functions

**File: `lib.rs` - Update `run_single_event_driven_simulation`:**

```rust
fn run_single_event_driven_simulation(
    players: &[Creature], 
    encounters: &[Encounter], 
    _log_enabled: bool
) -> (SimulationResult, Vec<crate::events::Event>) {
    let context = ErrorContext::new("simulation".to_string(), 0, 0);
    
    // Add validation at start
    if let Err(validation_error) = validate_scenario_for_edge_cases(&Scenario {
        players: players.to_vec(),
        encounters: encounters.to_vec(),
    }) {
        log_simulation_error(
            SimulationError::ValidationFailed(format!("Scenario validation failed: {:?}", validation_error)),
            context.clone()
        );
        // Return empty result with error logging
        return (vec![], vec![]);
    }
    
    // ... existing simulation code ...
    
    // Validate result before returning
    if let Err(result_error) = validate_single_result(&result, 0) {
        log_simulation_error(result_error, context);
        // Return empty result as fallback
        return (vec![], vec![]);
    }
    
    (result, all_events)
}
```

### Step 4: Implement Retry Logic

**File: `lib.rs` - Add retry wrapper:**

```rust
#[wasm_bindgen]
pub fn run_simulation_with_reliability(
    players: JsValue, 
    encounters: JsValue, 
    iterations: usize
) -> Result<JsValue, JsValue> {
    let players: Vec<Creature> = serde_wasm_bindgen::from_value(players)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse players: {}", e)))?;
    let encounters: Vec<Encounter> = serde_wasm_bindgen::from_value(encounters)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encounters: {}", e)))?;

    // Use enhanced reliability with retry logic
    let results = run_simulation_with_retry(&players, &encounters, iterations, 3)
        .map_err(|e| JsValue::from_str(&format!("Simulation failed: {}", e)))?;

    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);

    serde::Serialize::serialize(&results, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}
```

### Step 5: Add Monitoring to Quintile Analysis

**File: `lib.rs` - Update `run_quintile_analysis_wasm`:**

```rust
#[wasm_bindgen]
pub fn run_quintile_analysis_wasm(results: JsValue, scenario_name: &str, _party_size: usize) -> Result<JsValue, JsValue> {
    // Add debug logging
    console::log_1(&"=== Enhanced Quintile Analysis with Monitoring ===".into());
    
    let mut results: Vec<SimulationResult> = serde_wasm_bindgen::from_value(results)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse results: {}", e)))?;
    
    // Set up monitoring
    let thresholds = AlertThresholds::default();
    let mut monitor = SimulationMonitor::new(thresholds);
    
    console::log_1(&format!("Received {} simulation results", results.len()).into());
    
    // Validate results before processing
    match validate_simulation_results(&results) {
        Ok(validation_report) => {
            console::log_1(&format!("Validation passed: {:.1}% success rate", 
                validation_report.success_rate() * 100.0).into());
        }
        Err(validation_error) => {
            console::log_1(&format!("Validation failed: {}", validation_error).into());
            return Err(JsValue::from_str(&format!("Validation failed: {}", validation_error)));
        }
    }
    
    // Sort results by score from worst to best performance
    results.sort_by(|a, b| {
        let score_a = crate::aggregation::calculate_score(a);
        let score_b = crate::aggregation::calculate_score(b);
        score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Calculate party size from first result (use actual data instead of parameter)
    let actual_party_size = if let Some(first_result) = results.first() {
        if let Some(first_encounter) = first_result.first() {
            first_encounter.rounds.first()
                .map(|first_round| first_round.team1.len())
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    console::log_1(&format!("Calculated party size: {}", actual_party_size).into());
    
    let output = quintile_analysis::run_quintile_analysis(&results, scenario_name, actual_party_size);
    
    // Finalize monitoring
    let final_metrics = monitor.finalize();
    let report = MonitoringReport::generate(&monitor);
    
    console::log_1(&format!("Analysis completed. Health status: {}", 
        report.health_status.as_str()).into());
    
    if !report.alerts.is_empty() {
        console::log_1(&format!("Generated {} alerts", report.alerts.len()).into());
        for recommendation in &report.recommendations {
            console::log_1(&recommendation.clone().into());
        }
    }
    
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(false);
        
    serde::Serialize::serialize(&output, &serializer)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize quintile analysis: {}", e)))
}
```

## 🔧 Testing Implementation

### Run the Test Suite

```bash
cd simulation-wasm
cargo test reliability_tests --lib
```

### Manual Testing

1. **Test Empty Result Handling:**
```javascript
// In browser console or test file
const emptyResults = [];
run_quintile_analysis_wasm(emptyResults, "test", 0)
    .catch(error => console.log("Expected error:", error));
```

2. **Test High-Stress Simulation:**
```javascript
// Create a complex scenario with many combatants
const complexScenario = {
    players: [/* many players */],
    encounters: [/* many monsters */]
};

run_simulation_with_reliability(complexScenario.players, complexScenario.encounters, 1000)
    .then(results => console.log("Results:", results.length))
    .catch(error => console.log("Error:", error));
```

## 📊 Monitoring Integration

### Frontend Integration

Add monitoring display to your React components:

```typescript
// src/components/simulation/MonitoringPanel.tsx
import React, { useState, useEffect } from 'react';

interface MonitoringData {
  healthStatus: 'Healthy' | 'Degraded' | 'Critical';
  successRate: number;
  alerts: string[];
  recommendations: string[];
}

export const MonitoringPanel: React.FC = () => {
  const [monitoring, setMonitoring] = useState<MonitoringData | null>(null);
  
  useEffect(() => {
    // Fetch monitoring data from simulation results
    // This would be integrated with your existing simulation flow
  }, []);
  
  if (!monitoring) return null;
  
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Healthy': return '#4CAF50';
      case 'Degraded': return '#FF9800';
      case 'Critical': return '#F44336';
      default: return '#9E9E9E';
    }
  };
  
  return (
    <div className="monitoring-panel">
      <h3>Simulation Health</h3>
      <div style={{ color: getStatusColor(monitoring.healthStatus) }}>
        Status: {monitoring.healthStatus}
      </div>
      <div>Success Rate: {(monitoring.successRate * 100).toFixed(1)}%</div>
      
      {monitoring.alerts.length > 0 && (
        <div>
          <h4>Alerts</h4>
          <ul>
            {monitoring.alerts.map((alert, i) => (
              <li key={i}>{alert}</li>
            ))}
          </ul>
        </div>
      )}
      
      {monitoring.recommendations.length > 0 && (
        <div>
          <h4>Recommendations</h4>
          <ul>
            {monitoring.recommendations.map((rec, i) => (
              <li key={i}>{rec}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
```

## 🚨 Alert Configuration

### Custom Alert Thresholds

```rust
// Customize thresholds for your specific needs
let custom_thresholds = AlertThresholds {
    min_success_rate: 0.98,      // Require 98% success rate
    max_retry_rate: 0.05,        // Max 5% retry rate
    max_error_types: 3,          // Max 3 different error types
    max_execution_time: 15.0,    // Max 15s per iteration
    max_memory_usage: 512 * 1024 * 1024, // 512MB
    max_stall_time: 30,          // 30s stall detection
};

let monitor = SimulationMonitor::new(custom_thresholds);
```

## 📈 Success Metrics

### Key Performance Indicators

1. **Success Rate**: Target ≥99.5% (up from ~95%)
2. **Error Recovery**: Target ≥90% automatic recovery
3. **Performance Impact**: Target <5% slowdown
4. **Memory Usage**: Target <10% increase

### Monitoring Dashboard

Create a simple dashboard to track these metrics:

```typescript
// src/components/simulation/ReliabilityDashboard.tsx
export const ReliabilityDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState({
    successRate: 0,
    errorRate: 0,
    retryRate: 0,
    reliabilityScore: 0,
    totalIterations: 0,
    averageExecutionTime: 0
  });
  
  return (
    <div className="reliability-dashboard">
      <div className="metric">
        <h4>Success Rate</h4>
        <div className="value">{(metrics.successRate * 100).toFixed(1)}%</div>
      </div>
      
      <div className="metric">
        <h4>Error Rate</h4>
        <div className="value">{(metrics.errorRate * 100).toFixed(1)}%</div>
      </div>
      
      <div className="metric">
        <h4>Retry Rate</h4>
        <div className="value">{(metrics.retryRate * 100).toFixed(1)}%</div>
      </div>
      
      <div className="metric">
        <h4>Reliability Score</h4>
        <div className="value">{(metrics.reliabilityScore * 100).toFixed(1)}%</div>
      </div>
      
      <div className="metric">
        <h4>Avg Execution Time</h4>
        <div className="value">{metrics.averageExecutionTime.toFixed(2)}s</div>
      </div>
    </div>
  );
};
```

## 🔄 Deployment Checklist

### Pre-Deployment
- [ ] All tests pass (`cargo test`)
- [ ] No new compiler warnings
- [ ] Performance benchmarks meet targets
- [ ] Error logging works correctly
- [ ] Monitoring dashboard displays data

### Post-Deployment
- [ ] Monitor error rates for 24 hours
- [ ] Check performance impact
- [ ] Verify alert system works
- [ ] Collect user feedback on reliability

### Rollback Plan
If issues are detected:
1. Disable new error handling via feature flags
2. Revert to original aggregation functions
3. Monitor system stability
4. Investigate and fix issues before re-enabling

## 🎯 Expected Outcomes

### Immediate Benefits (Week 1)
- ✅ Elimination of simulation crashes
- ✅ Detailed error logging for debugging
- ✅ Improved success rate through retry logic
- ✅ Better visibility into simulation health

### Medium-term Benefits (Week 2)
- ✅ Graceful handling of edge cases
- ✅ Reduced manual intervention
- ✅ More robust simulation pipeline
- ✅ Better user experience

### Long-term Benefits (Week 3)
- ✅ Quantifiable reliability metrics
- ✅ Proactive issue detection
- ✅ Continuous improvement framework
- ✅ Foundation for future enhancements

This implementation provides a comprehensive solution to the -1000000 score issue while establishing a robust foundation for ongoing reliability improvements.