//! Global state management for WASM interface
//!
//! Centralizes all global/static state used by the WASM API.

use std::sync::{Mutex, OnceLock};
use crate::storage_manager::StorageManager;

/// Global storage for last simulation events (thread-safe)
pub static LAST_SIMULATION_EVENTS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Global storage manager for WASM interface
static STORAGE_MANAGER: OnceLock<Mutex<StorageManager>> = OnceLock::new();

/// Initialize or get the global storage manager
pub fn get_storage_manager() -> &'static Mutex<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| Mutex::new(StorageManager::default()))
}

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
