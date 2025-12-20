//! Simple example demonstrating Phase 2 background processing system
//! 
//! This example shows the core functionality without complex model dependencies

use simulation_wasm::queue_manager::SimulationQueue;
use simulation_wasm::progress_communication::{ProgressCommunication, ProgressSubscription};
use simulation_wasm::storage_integration::{StorageIntegration, StorageIntegrationConfig};
use simulation_wasm::storage_manager::StorageManager;
use simulation_wasm::storage::{ScenarioParameters, SimulationConfig};
use simulation_wasm::background_simulation::SimulationPriority;
use std::collections::HashMap;

fn main() {
    println!("=== Phase 2 Background Processing Demo ===\n");

    // 1. Initialize the storage system
    println!("1. Initializing storage system...");
    let storage_manager = StorageManager::default();
    let queue = SimulationQueue::new(10);
    let (progress_comm, progress_receiver) = ProgressCommunication::new();
    
    let config = StorageIntegrationConfig {
        max_concurrent_simulations: 2,
        progress_update_interval_ms: 100,
        auto_store_completed: true,
        auto_cleanup: true,
        max_completed_age_seconds: 3600,
    };

    let storage_integration = StorageIntegration::new(
        storage_manager,
        queue,
        progress_comm.clone(),
        config,
    );

    println!("✓ Storage system initialized\n");

    // 2. Subscribe to progress updates
    println!("2. Setting up progress subscriptions...");
    let subscription = ProgressSubscription::new("demo_subscription".to_string())
        .completions_only();
    
    let sub_receiver = storage_integration.progress_comm
        .subscribe(subscription)
        .expect("Failed to subscribe to progress updates");
    
    println!("✓ Progress subscription created\n");

    // 3. Create simple test scenarios
    println!("3. Creating test scenarios...");
    
    let low_priority_scenario = ScenarioParameters {
        players: vec![], // Empty for demo
        encounters: vec![],
        iterations: 50,
        config: SimulationConfig {
            log_enabled: true,
            seed: Some(42),
            options: HashMap::new(),
        },
    };

    let high_priority_scenario = ScenarioParameters {
        players: vec![],
        encounters: vec![],
        iterations: 100,
        config: SimulationConfig {
            log_enabled: true,
            seed: Some(123),
            options: HashMap::new(),
        },
    };

    let critical_priority_scenario = ScenarioParameters {
        players: vec![],
        encounters: vec![],
        iterations: 25,
        config: SimulationConfig {
            log_enabled: true,
            seed: Some(456),
            options: HashMap::new(),
        },
    };

    println!("✓ Created 3 test scenarios\n");

    // 4. Submit simulation requests with different priorities
    println!("4. Submitting simulation requests...");
    
    // Submit requests in order of increasing priority to test queue behavior
    let low_result = storage_integration.submit_simulation_request(
        low_priority_scenario,
        SimulationPriority::Low,
    ).expect("Failed to submit low priority request");

    let high_result = storage_integration.submit_simulation_request(
        high_priority_scenario,
        SimulationPriority::High,
    ).expect("Failed to submit high priority request");

    let critical_result = storage_integration.submit_simulation_request(
        critical_priority_scenario,
        SimulationPriority::Critical,
    ).expect("Failed to submit critical priority request");

    println!("✓ Submitted 3 simulation requests:");
    println!("  - Low priority (50 iterations): {}", low_result.request_id);
    println!("  - High priority (100 iterations): {}", high_result.request_id);
    println!("  - Critical priority (25 iterations): {}", critical_result.request_id);
    println!();

    // 5. Check queue status
    println!("5. Checking queue status...");
    let stats = storage_integration.get_integration_stats();
    println!("Queue Statistics:");
    println!("  - Pending requests: {}", stats.queue_stats.pending_count);
    println!("  - Processing requests: {}", stats.queue_stats.processing_count);
    println!("  - Active simulations: {}", stats.active_simulations);
    println!("  - Max concurrent: {}", stats.max_concurrent_simulations);
    println!("  - Progress subscriptions: {}", stats.progress_subscriptions);
    println!();

    // 6. Process queue and monitor progress
    println!("6. Processing queue and monitoring progress...");
    
    // Process requests from queue
    for _ in 0..3 {
        if let Ok(Some(sim_id)) = storage_integration.process_next_request() {
            println!("✓ Started simulation: {:?}", sim_id);
        }
    }

    // Monitor progress updates for a short time
    println!("Monitoring progress updates (1 second)...");
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < std::time::Duration::from_secs(1) {
        // Check for progress updates
        if let Ok(update) = progress_receiver.try_recv() {
            println!("Progress Update: {} - {:.1}% - {}", 
                update.simulation_id.0,
                update.progress_percentage * 100.0,
                update.current_phase
            );
        }
        
        // Check for subscription updates
        if let Ok(update) = sub_receiver.try_recv() {
            println!("Subscription Update: {} - {:?}", 
                update.simulation_id.0,
                update.update_type
            );
        }
        
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 7. Check final status
    println!("\n7. Final status check...");
    for request_id in [&low_result.request_id, &high_result.request_id, &critical_result.request_id] {
        let status = storage_integration.get_request_status(request_id);
        println!("Request {}: {:?}", request_id, status);
    }

    // 8. Demonstrate cleanup
    println!("\n8. Performing cleanup...");
    let cleanup_result = storage_integration.perform_cleanup()
        .expect("Cleanup failed");
    println!("✓ Cleanup completed:");
    println!("  - Cleaned simulations: {}", cleanup_result.cleaned_simulations);
    println!("  - Cleaned storage: {}", cleanup_result.cleaned_storage);

    println!("\n=== Demo Complete ===");
    println!("The Phase 2 implementation provides:");
    println!("✓ Background simulation processing with progress tracking");
    println!("✓ Priority-based queue management with deduplication");
    println!("✓ Real-time progress communication system");
    println!("✓ Automatic storage integration and cleanup");
    println!("✓ Thread-safe operations and error handling");
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation_wasm::queue_manager::SimulationRequest;
    use simulation_wasm::progress_communication::ProgressUpdateType;

    #[test]
    fn test_queue_priority_ordering() {
        let queue = SimulationQueue::new(5);
        
        let scenario1 = ScenarioParameters {
            players: vec![],
            encounters: vec![],
            iterations: 10,
            config: SimulationConfig::default(),
        };
        
        let scenario2 = ScenarioParameters {
            players: vec![],
            encounters: vec![],
            iterations: 20,
            config: SimulationConfig::default(),
        };
        
        let scenario3 = ScenarioParameters {
            players: vec![],
            encounters: vec![],
            iterations: 30,
            config: SimulationConfig::default(),
        };
        
        // Add requests in different order
        let low_req = SimulationRequest::new(scenario1, SimulationPriority::Low);
        let critical_req = SimulationRequest::new(scenario2, SimulationPriority::Critical);
        let high_req = SimulationRequest::new(scenario3, SimulationPriority::High);
        
        queue.enqueue(low_req).unwrap();
        queue.enqueue(critical_req).unwrap();
        queue.enqueue(high_req).unwrap();
        
        // Should get critical priority first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.priority, SimulationPriority::Critical);
        
        // Then high priority
        let second = queue.dequeue().unwrap();
        assert_eq!(second.priority, SimulationPriority::High);
        
        // Then low priority
        let third = queue.dequeue().unwrap();
        assert_eq!(third.priority, SimulationPriority::Low);
    }

    #[test]
    fn test_progress_communication() {
        let (comm, receiver) = ProgressCommunication::new();
        
        // Subscribe to updates
        let subscription = ProgressSubscription::new("test".to_string());
        let mut sub_receiver = comm.subscribe(subscription).unwrap();
        
        // Send an update
        let sim_id = simulation_wasm::background_simulation::BackgroundSimulationId::new();
        let update = simulation_wasm::progress_communication::ProgressUpdate::new(
            sim_id.clone(),
            ProgressUpdateType::Started,
            0.0,
            "Test",
        );
        
        comm.send_update(update).unwrap();
        
        // Should receive update
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.simulation_id, sim_id);
        assert_eq!(received.current_phase, "Test");
        
        // Subscription should also receive it
        let sub_received = sub_receiver.try_recv().unwrap();
        assert_eq!(sub_received.simulation_id, sim_id);
    }

    #[test]
    fn test_storage_integration() {
        let storage_manager = StorageManager::default();
        let queue = SimulationQueue::new(5);
        let (progress_comm, _) = ProgressCommunication::new();
        let config = StorageIntegrationConfig::default();
        
        let integration = StorageIntegration::new(
            storage_manager,
            queue,
            progress_comm,
            config,
        );
        
        // Submit a request
        let scenario = ScenarioParameters {
            players: vec![],
            encounters: vec![],
            iterations: 10,
            config: SimulationConfig::default(),
        };
        
        let result = integration.submit_simulation_request(
            scenario,
            SimulationPriority::Normal,
        ).unwrap();
        
        assert!(!result.request_id.is_empty());
        
        // Check status
        let status = integration.get_request_status(&result.request_id);
        assert!(matches!(status, 
            simulation_wasm::storage_integration::SimulationRequestStatus::Running { .. } |
            simulation_wasm::storage_integration::SimulationRequestStatus::Queued { .. }
        ));
    }
}