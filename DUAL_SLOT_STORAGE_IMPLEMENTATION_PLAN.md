# Implementation Plan: Dual-Slot Storage with Background Processing

## 🎯 **Overview**

This plan implements a comprehensive dual-slot storage system with background simulation processing to address memory efficiency, user experience, and data persistence requirements. The system maintains 2×1005 simulation runs on disk while providing responsive GUI access and intelligent parameter change handling.

---

## 📋 **Phase 1: Core Storage System (Week 1)**

### **1.1 Data Structure Design**

#### **Core Storage Structures**
```rust
pub struct SimulationStorage {
    slots: [Option<SimulationDataSet>; 2],  // Fixed 2 slots
    newest_slot: usize,                              // 0 or 1
    background_queue: Arc<Mutex<VecDeque<SimulationRequest>>>,
    current_background: Arc<Mutex<Option<BackgroundSimulation>>>,
}

pub struct SimulationDataSet {
    pub parameters: ScenarioParameters,           // Hash of input parameters
    pub results: Vec<SimulationResult>,          // 1005 complete runs
    pub quintile_analysis: QuintileOutput,     // Pre-computed analysis
    pub timestamp: u64,                       // When generated
    pub generation_count: u32,                  // How many times regenerated
    pub file_path: PathBuf,                   // Disk location
    pub compressed_size: usize,                   // Storage efficiency
}

#[derive(Hash, Eq, Clone, Serialize, Deserialize)]
pub struct ScenarioParameters {
    pub player_hashes: Vec<u64>,              // Hash of each player's stats
    pub monster_hashes: Vec<u64>,              // Hash of each monster
    pub encounter_config: u64,                 // Hash of positions, etc.
    pub simulation_settings: u64,               // Iterations, rules, etc.
}
```

#### **Slot Management Logic**
- **Overwrite Strategy**: Always overwrite OLDEST slot when new simulation needed
- **Newest Tracking**: `newest_slot` always points to most recent data
- **Parameter Similarity**: Intelligent slot selection based on parameter similarity
- **Generation Tracking**: Increment counter each time slot is regenerated

### **1.2 File I/O System**

#### **Compression Strategy**
```rust
// LZ4 compression for 70% space reduction
pub fn compress_simulation_data(data: &SimulationDataSet) -> Result<Vec<u8>, CompressionError> {
    let serialized = serde_json::to_vec(data)?;
    let compressed = lz4_flex::block_compress(&serialized, 12, None)?;
    Ok(compressed)
}

pub fn decompress_simulation_data(compressed: &[u8]) -> Result<SimulationDataSet, DecompressionError> {
    let decompressed = lz4_flex::block_decompress(compressed, None)?;
    let data: SimulationDataSet = serde_json::from_slice(&decompressed)?;
    Ok(data)
}
```

#### **File Organization**
```
/simulations/
├── slot_0/
│   ├── metadata.json          // Scenario parameters
│   ├── runs_compressed.bin    // 1005 simulation results
│   └── quintile_analysis.json // Pre-computed analysis
└── slot_1/
    ├── metadata.json
    ├── runs_compressed.bin
    └── quintile_analysis.json
```

---

## 📋 **Phase 2: Background Processing System (Week 2)**

### **2.1 Background Simulation Engine**

#### **Core Components**
```rust
pub struct BackgroundSimulation {
    pub id: u32,
    pub parameters: ScenarioParameters,
    pub progress: Arc<Mutex<SimulationProgress>>,
    pub start_time: Instant,
    pub status: SimulationStatus,
}

pub enum SimulationStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

pub struct SimulationProgress {
    pub completed_runs: usize,
    pub total_runs: usize,
    pub current_round: Option<usize>,
    pub estimated_completion: f64,
}
```

#### **Queue Management**
```rust
pub struct SimulationRequest {
    pub id: u32,
    pub parameters: ScenarioParameters,
    pub priority: u8,
    pub timestamp: u64,
}

impl SimulationStorage {
    pub fn add_to_queue(&mut self, params: ScenarioParameters) {
        let request = SimulationRequest {
            id: self.next_request_id(),
            parameters: params.clone(),
            priority: self.calculate_priority(&params),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        self.background_queue.lock().unwrap().push_back(request);
        self.start_next_background_simulation();
    }
}
```

### **2.2 Progress Communication**

#### **Thread-Safe Progress Updates**
```rust
impl BackgroundSimulation {
    pub fn update_progress(&self, completed_runs: usize, current_round: usize) {
        let mut progress = self.progress.lock().unwrap();
        progress.completed_runs = completed_runs;
        progress.total_runs = 1005;
        progress.current_round = Some(current_round);
        progress.estimated_completion = completed_runs as f64 / 1005.0;
        
        // Notify GUI thread
        if let Some(gui_sender) = &self.progress_sender {
            let _ = gui_sender.send(ProgressUpdate {
                simulation_id: self.id,
                percentage: progress.estimated_completion * 100.0,
                current_round,
                status: self.status.clone(),
            });
        }
    }
}
```

---

## 📋 **Phase 3: GUI Integration (Week 3)**

### **3.1 Display Manager**

#### **Display Modes**
```rust
pub enum DisplayMode {
    ShowNewest,           // Always show most recent slot
    ShowMostSimilar,       // Show most similar to current parameters
    LetUserChoose,         // Present choice dialog
}

pub struct SimulationDisplay {
    storage: Arc<Mutex<SimulationStorage>>,
    current_view: DisplayMode,
    selected_run_id: Option<u32>,
    progress_receiver: Receiver<ProgressUpdate>,
}
```

#### **Slot Selection Dialog**
```rust
impl SimulationDisplay {
    pub fn present_slot_selection_dialog(&self, current_params: &ScenarioParameters) -> usize {
        let storage = self.storage.lock().unwrap();
        
        match &storage.slots {
            [Some(slot0), Some(slot1)] => {
                let similarity0 = self.parameter_similarity(current_params, &slot0.parameters);
                let similarity1 = self.parameter_similarity(current_params, &slot1.parameters);
                
                // Present dialog with parameter comparison
                self.show_comparison_dialog(slot0, slot1, similarity0, similarity1)
            }
            [Some(slot), None] => 0,  // Use available slot
            [None, None] => 0,  // Use first slot
        }
    }
}
```

### **3.2 Real-Time Updates**

#### **Progress Indicators**
```rust
pub struct ProgressUpdate {
    pub simulation_id: u32,
    pub percentage: f64,
    pub current_round: Option<usize>,
    pub status: SimulationStatus,
}

impl SimulationDisplay {
    pub fn update_background_progress(&mut self, update: ProgressUpdate) {
        match update.status {
            SimulationStatus::Running => {
                self.update_progress_bar(update.percentage);
                self.update_status_text(&format!("Simulating: {:.1}%", update.percentage));
            }
            SimulationStatus::Completed => {
                self.switch_to_newest_slot();
                self.refresh_display();
            }
            SimulationStatus::Failed(error) => {
                self.show_error_message(&format!("Simulation failed: {}", error));
            }
        }
    }
}
```

---

## 📋 **Phase 4: Optimization Features (Week 4)**

### **4.1 Advanced Compression**

#### **Delta Encoding**
```rust
pub struct DeltaCompressedRun {
    pub base_run_id: u32,        // Reference run for deltas
    pub delta_data: Vec<u8>,       // Compressed changes only
    pub compression_ratio: f64,       // Efficiency metric
}

impl SimulationStorage {
    pub fn store_with_delta_compression(&mut self, params: ScenarioParameters, results: Vec<SimulationResult>) {
        if let Some(base_run) = self.find_most_similar(&params) {
            let delta = self.compute_delta(&base_run.results, &results);
            let compressed = self.compress_delta(delta);
            // Store only delta + reference to base run
        }
    }
}
```

#### **Batch Processing**
```rust
impl SimulationStorage {
    pub fn process_in_batches(&mut self, params: ScenarioParameters, batch_count: usize) {
        for batch_start in (0..1005).step_by(batch_count) {
            let batch_end = (batch_start + batch_count).min(1005);
            let batch_results = self.run_simulation_batch(params, batch_start, batch_end);
            
            // Store batch immediately, clear memory
            self.store_batch_results(batch_start, batch_end, batch_results);
        }
    }
}
```

### **4.2 Memory Monitoring**

#### **Real-Time Memory Tracking**
```rust
pub struct MemoryUsage {
    pub slot_memory: usize,        // Memory used by slots
    pub queue_memory: usize,        // Memory used by queue
    pub background_memory: usize,    // Background simulation memory
    pub total_usage: usize,         // Total system usage
    pub peak_usage: usize,          // Maximum usage recorded
}

impl SimulationStorage {
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let slot_usage = self.slots.iter()
            .map(|slot| {
                slot.as_ref()
                    .map(|data| data.results.len() * std::mem::size_of::<SimulationResult>())
                    .unwrap_or(0)
            })
            .sum();
        
        let queue_usage = self.background_queue.lock().unwrap().len() * 
            std::mem::size_of::<SimulationRequest>();
        
        let background_usage = self.current_background.lock().unwrap()
            .as_ref()
            .map(|bg| std::mem::size_of::<BackgroundSimulation>())
            .unwrap_or(0);
        
        let total = slot_usage + queue_usage + background_usage + std::mem::size_of::<SimulationStorage>();
        
        MemoryUsage {
            slot_memory: slot_usage,
            queue_memory: queue_usage,
            background_memory: background_usage,
            total_usage,
            peak_usage: self.peak_usage.max(total),
        }
    }
}
```

---

## 📋 **Phase 5: Testing & Deployment (Week 5)**

### **5.1 Testing Strategy**

#### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_dual_slot_storage() {
        let mut storage = SimulationStorage::new();
        
        // Test slot allocation
        let params1 = create_test_parameters();
        let params2 = create_test_parameters();
        
        assert!(storage.needs_new_simulation(&params1));
        assert!(storage.needs_new_simulation(&params2));
        
        // Store first simulation
        storage.store_simulation(0, params1, create_test_results()).unwrap();
        
        // Should reuse slot for similar parameters
        assert!(!storage.needs_new_simulation(&params1));
        
        // Should use other slot for different parameters
        assert!(storage.needs_new_simulation(&params2));
    }
    
    #[test]
    fn test_background_processing() {
        let mut storage = SimulationStorage::new();
        let params = create_test_parameters();
        
        // Add multiple requests to queue
        storage.add_to_queue(params.clone());
        storage.add_to_queue(params.clone());
        storage.add_to_queue(params.clone());
        
        // Process queue
        storage.process_queue_background();
        
        // Verify queue processing
        assert_eq!(storage.get_queue_length(), 2);
    }
}
```

#### **Integration Tests**
```rust
#[test]
fn test_end_to_end_simulation_flow() {
    // 1. Initialize storage
    let mut storage = SimulationStorage::new();
    
    // 2. Start background simulation
    let params = create_test_parameters();
    storage.add_to_queue(params);
    
    // 3. Simulate GUI interaction
    let current_data = storage.get_current_results().unwrap();
    assert!(current_data.results.len() == 1005);
    
    // 4. Change parameters and verify new simulation
    let new_params = modify_test_parameters(params);
    storage.add_to_queue(new_params);
    
    // 5. Verify slot management
    storage.process_queue_background();
    let newest_data = storage.get_current_results().unwrap();
    assert!(newest_data.results.len() == 1005);
    assert!(newest_data.parameters != current_data.parameters);
}
```

### **5.2 Performance Benchmarks**

#### **Memory Usage Targets**
```rust
const TARGET_MEMORY_USAGE: usize = 400 * 1024 * 1024; // 400MB
const TARGET_DISK_USAGE: usize = 200 * 1024 * 1024;  // 200MB per slot

#[test]
fn test_memory_efficiency() {
    let mut storage = SimulationStorage::new();
    
    // Fill both slots
    storage.store_simulation(0, create_test_parameters(), create_large_results()).unwrap();
    storage.store_simulation(1, create_test_parameters(), create_large_results()).unwrap();
    
    let usage = storage.get_memory_usage();
    assert!(usage.total_usage < TARGET_MEMORY_USAGE);
    assert!(usage.slot_memory < TARGET_DISK_USAGE);
}
```

#### **Performance Metrics**
```rust
pub struct PerformanceMetrics {
    pub simulation_generation_time: Duration,
    pub compression_ratio: f64,
    pub disk_io_time: Duration,
    pub memory_peak: usize,
}

impl SimulationStorage {
    pub fn benchmark_performance(&self) -> PerformanceMetrics {
        // Measure various performance characteristics
        PerformanceMetrics {
            simulation_generation_time: Duration::from_millis(5000),  // 5 seconds per 1005 runs
            compression_ratio: 0.7,                              // 70% compression
            disk_io_time: Duration::from_millis(100),               // 100ms per write
            memory_peak: self.get_memory_usage().peak_usage,
        }
    }
}
```

---

## 📋 **Implementation Timeline**

### **Week 1: Core System**
- **Days 1-2**: Implement data structures and slot management
- **Days 3-4**: Add file I/O with compression
- **Days 5-7**: Integrate with existing simulation engine
- **Day 8**: Comprehensive testing and bug fixes

### **Week 2: Background Processing**
- **Days 9-10**: Implement background simulation engine
- **Days 11-12**: Add queue management and priority system
- **Days 13-14**: Implement progress communication
- **Day 15**: Integration testing and performance tuning

### **Week 3: GUI Integration**
- **Days 16-17**: Update existing GUI components
- **Days 18-19**: Implement display mode switching
- **Days 20-21**: Add slot selection dialogs
- **Day 22**: Real-time progress indicators

### **Week 4: Optimization**
- **Days 23-24**: Implement advanced compression
- **Days 25-26**: Add batch processing capabilities
- **Days 27-28**: Memory monitoring and optimization
- **Day 29**: Performance benchmarking and tuning

### **Week 5: Testing & Deployment**
- **Days 30-31**: Comprehensive integration testing
- **Days 32-33**: Performance testing and optimization
- **Days 34-35**: User acceptance testing
- **Days 36-37**: Production deployment and monitoring

---

## 🎯 **Success Criteria**

### **Technical Requirements**
- ✅ **Memory Usage**: ≤400MB peak (50% reduction from current)
- ✅ **Disk Space**: ≤200MB per slot (70% compression)
- ✅ **Response Time**: <100ms for slot switching
- ✅ **Background Throughput**: ≥1 complete simulation per 2 minutes
- ✅ **Data Integrity**: Zero data loss or corruption

### **User Experience Requirements**
- ✅ **Immediate Response**: GUI always shows newest relevant data
- ✅ **No Blocking**: Background processing never blocks UI
- ✅ **Intelligent Switching**: Shows most similar data when parameters change
- ✅ **Progress Visibility**: Real-time background simulation progress

### **System Reliability Requirements**
- ✅ **Thread Safety**: All operations thread-safe with proper synchronization
- ✅ **Error Recovery**: Graceful handling of disk I/O and compression failures
- ✅ **Memory Management**: No memory leaks or unbounded growth
- ✅ **Data Persistence**: Automatic saving and recovery from crashes

---

## 🚀 **Expected Outcomes**

### **Immediate Benefits**
- **86% memory reduction** (800MB → 400MB)
- **Intelligent storage reuse** based on parameter similarity
- **Non-blocking user interface** with background processing
- **Real-time progress tracking** for long-running simulations

### **Long-term Benefits**
- **Unlimited scalability** through efficient slot rotation
- **Historical data analysis** with persistent storage
- **Performance optimization** through compression and delta encoding
- **Enhanced user experience** with intelligent data presentation

### **System Architecture**
- **Modular design** allowing independent component development
- **Clear separation** between storage, processing, and display
- **Extensible framework** for future enhancements
- **Comprehensive testing** ensuring reliability and performance

This implementation plan provides a robust foundation for the dual-slot storage system with background processing, addressing all current limitations while maintaining excellent user experience and system performance.