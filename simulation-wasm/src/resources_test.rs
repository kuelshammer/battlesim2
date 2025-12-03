use super::*;

#[test]
fn test_ledger_basic_consumption() {
    let mut ledger = ResourceLedger::new();
    let action_resource = ResourceType::Action;
    
    // Register Action: Max 1, reset on any rest
    ledger.register_resource(action_resource.clone(), 1.0, None);
    
    assert!(ledger.has(&action_resource, 1.0));
    assert!(!ledger.has(&action_resource, 2.0));
    
    // Consume
    assert!(ledger.consume(&action_resource, 1.0).is_ok());
    assert!(!ledger.has(&action_resource, 1.0));
    
    // Over-consume
    assert!(ledger.consume(&action_resource, 1.0).is_err());
}

#[test]
fn test_ledger_restoration() {
    let mut ledger = ResourceLedger::new();
    let spell_slot = ResourceType::SpellSlot(1);
    
    ledger.register_resource(spell_slot.clone(), 4.0, Some(ResetType::LongRest));
    
    // Consume all
    let _ = ledger.consume(&spell_slot, 4.0);
    assert!(!ledger.has(&spell_slot, 1.0));
    
    // Restore 1
    ledger.restore(&spell_slot, 1.0);
    assert!(ledger.has(&spell_slot, 1.0));
    
    // Try to restore beyond max
    ledger.restore(&spell_slot, 10.0);
    let current = *ledger.current.get(&spell_slot).unwrap();
    assert_eq!(current, 4.0); // Should cap at max
}

#[test]
fn test_ledger_reset() {
    let mut ledger = ResourceLedger::new();
    let sr_resource = ResourceType::ClassResource("Ki".to_string());
    let lr_resource = ResourceType::SpellSlot(1);
    
    ledger.register_resource(sr_resource.clone(), 5.0, Some(ResetType::ShortRest));
    ledger.register_resource(lr_resource.clone(), 4.0, Some(ResetType::LongRest));
    
    // Empty both
    let _ = ledger.consume(&sr_resource, 5.0);
    let _ = ledger.consume(&lr_resource, 4.0);
    
    // Short Rest
    ledger.reset(ResetType::ShortRest);
    
    assert!(ledger.has(&sr_resource, 5.0)); // Should be full
    assert!(!ledger.has(&lr_resource, 1.0)); // Should still be empty
    
    // Empty SR again
    let _ = ledger.consume(&sr_resource, 5.0);
    
    // Long Rest
    ledger.reset(ResetType::LongRest);
    
    assert!(ledger.has(&sr_resource, 5.0)); // Both should be full
    assert!(ledger.has(&lr_resource, 4.0));
}

#[test]
fn test_variable_cost_logic() {
    // Simulating how we'd handle Variable cost in the engine
    let mut ledger = ResourceLedger::new();
    let hp_pool = ResourceType::HP; // Using HP as a pool
    
    ledger.register_resource(hp_pool.clone(), 20.0, Some(ResetType::LongRest));
    
    let cost = ActionCost::Variable(hp_pool.clone(), 1.0, 20.0);
    
    // Scenario: User wants to spend 5
    let desired_amount = 5.0;
    
    if let ActionCost::Variable(r, min, max) = &cost {
        assert_eq!(r, &hp_pool);
        assert!(desired_amount >= *min);
        assert!(desired_amount <= *max);
        
        assert!(ledger.has(r, desired_amount));
        assert!(ledger.consume(r, desired_amount).is_ok());
    }
    
    // Remaining should be 15
    assert!(ledger.has(&hp_pool, 15.0));
    assert!(!ledger.has(&hp_pool, 15.1));
}
