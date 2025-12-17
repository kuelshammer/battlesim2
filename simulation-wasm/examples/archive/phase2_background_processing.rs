//! Example demonstrating the Phase 2 dual-slot storage system with background processing
//! 
//! This example shows:
//! 1. Background simulation engine with progress tracking
//! 2. Queue management with priority handling
//! 3. Progress communication system
//! 4. Storage integration layer

use simulation_wasm::background_simulation::SimulationPriority;
use simulation_wasm::queue_manager::SimulationQueue;
use simulation_wasm::progress_communication::{ProgressCommunication, ProgressSubscription};
use simulation_wasm::storage_integration::{StorageIntegration, StorageIntegrationConfig};
use simulation_wasm::storage_manager::StorageManager;
use simulation_wasm::storage::{ScenarioParameters, SimulationConfig};
use simulation_wasm::model::{Creature, Action, AtkAction, DiceFormula};
use std::thread;
use std::time::Duration;

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
        actions: vec![
            Action::Atk(AtkAction {
                base: ActionBase {
                    id: "sword_attack".to_string(),
                    name: "Sword Attack".to_string(),
                    description: None,
                },
                range: "5".to_string(),
                attack_bonus: Some(5),
                damage: Some("1d8+3".to_string()),
                damage_type: Some("slashing".to_string()),
                crit_damage: None,
                crit_on: None,
                max_targets: None,
                target: "enemy".to_string(),
                to_hit: None,
                on_hit: None,
                on_crit: None,
                on_miss: None,
                cost: vec![],
                requirements: vec![],
            })
        ],
        triggers: vec![],
        spell_slots: None,
        class_resources: None,
        hit_dice: None,
        con_modifier: None,
    }
}

fn create_test_scenario(iterations: usize) -> ScenarioParameters {
    ScenarioParameters {
        players: vec![
            create_test_creature("Fighter", 50.0, 16.0),
            create_test_creature("Cleric", 30.0, 14.0),
        ],
        encounters: vec![
            // This would normally contain actual monster data
            // For simplicity, we're using empty encounters
        ],
        iterations,
        config: SimulationConfig {
            log_enabled: true,
            seed: Some(42),
            options: std::collections::HashMap::new(),
        },
    }
}

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

    // 3. Submit simulation requests with different priorities
    println!("3. Submitting simulation requests...");
    
    let low_priority_scenario = create_test_scenario(50);
    let high_priority_scenario = create_test_scenario(100);
    let critical_priority_scenario = create_test_scenario(25);

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

    // 4. Check queue status
    println!("4. Checking queue status...");
    let stats = storage_integration.get_integration_stats();
    println!("Queue Statistics:");
    println!("  - Pending requests: {}", stats.queue_stats.pending_count);
    println!("  - Processing requests: {}", stats.queue_stats.processing_count);
    println!("  - Active simulations: {}", stats.active_simulations);
    println!("  - Max concurrent: {}", stats.max_concurrent_simulations);
    println!("  - Progress subscriptions: {}", stats.progress_subscriptions);
    println!();

    // 5. Process queue and monitor progress
    println!("5. Processing queue and monitoring progress...");
    
    // Process requests from queue
    for _ in 0..3 {
        if let Ok(Some(sim_id)) = storage_integration.process_next_request() {
            println!("✓ Started simulation: {:?}", sim_id);
        }
    }

    // Monitor progress updates for a short time
    println!("Monitoring progress updates (2 seconds)...");
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < Duration::from_secs(2) {
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
        
        thread::sleep(Duration::from_millis(100));
    }

    // 6. Check final status
    println!("\n6. Final status check...");
    for request_id in [&low_result.request_id, &high_result.request_id, &critical_result.request_id] {
        let status = storage_integration.get_request_status(request_id);
        println!("Request {}: {:?}", request_id, status);
    }

    // 7. Demonstrate cleanup
    println!("\n7. Performing cleanup...");
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
    use simulation_wasm::background_simulation::{BackgroundSimulationEngine, BackgroundSimulationId};
    use simulation_wasm::queue_manager::SimulationRequest;
    use simulation_wasm::progress_communication::ProgressUpdateType;

    #[test]
    fn test_background_simulation_creation() {
        let storage_manager = StorageManager::default();
        let (engine, _receiver) = BackgroundSimulationEngine::new(storage_manager);
        
        // Test that engine was created successfully
        assert_eq!(engine.get_integration_stats().active_simulations, 0);
    }

    #[test]
    fn test_queue_priority_ordering() {
        let queue = SimulationQueue::new(5);
        
        let scenario1 = create_test_scenario(10);
        let scenario2 = create_test_scenario(20);
        let scenario3 = create_test_scenario(30);
        
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
        let sub_receiver = comm.subscribe(subscription).unwrap();
        
        // Send an update
        let sim_id = BackgroundSimulationId::new();
        let update = simulation_wasm::progress_communication::ProgressUpdate::new(
            sim_id.clone(),
            ProgressUpdateType::Started,
            0.0,
            "Test",
        );
        
        comm.send_update(update).unwrap();
        
        // Should receive the update
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
        let scenario = create_test_scenario(10);
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