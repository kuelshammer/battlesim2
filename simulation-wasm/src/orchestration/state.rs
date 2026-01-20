//! Global state management for WASM interface
//!
//! Centralizes all global/static state used by the WASM API.

use std::sync::Mutex;

/// Global storage for last simulation events (thread-safe)
/// Since storage is removed, this is kept for API compatibility but always returns None
pub static LAST_SIMULATION_EVENTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Store simulation events for later retrieval
pub fn store_simulation_events(events: Vec<String>) {
    if let Ok(mut events_guard) = LAST_SIMULATION_EVENTS.lock() {
        *events_guard = Some(events);
    }
}

/// Retrieve stored simulation events
pub fn get_stored_simulation_events() -> Option<Vec<String>> {
    match LAST_SIMULATION_EVENTS.lock() {
        Ok(events_guard) => events_guard.clone(),
        Err(_) => None,
    }
}
