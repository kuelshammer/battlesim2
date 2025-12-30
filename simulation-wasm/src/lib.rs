pub mod dice;
pub mod rng;
pub mod actions;
pub mod targeting;
pub mod enums;
pub mod model;
pub mod aggregation;
pub mod cleanup;
pub mod resolution;
pub mod resources;
pub mod events;
pub mod context;
pub mod reactions;
pub mod execution;
pub mod action_resolver;
pub mod validation; // New module for requirement validation
pub mod utilities;
pub mod decile_analysis;
pub mod combat_stats;
pub mod scoring_test;
pub mod creature_adjustment;
pub mod adjustment_test;
pub mod auto_balancer;
pub mod dice_reconstruction;
pub mod intensity_calculation;
#[cfg(test)]
mod intensity_test;
pub mod error_handling; // Enhanced error handling system
pub mod enhanced_validation; // Comprehensive validation
pub mod recovery; // Error recovery mechanisms
pub mod safe_aggregation; // Safe aggregation functions
pub mod monitoring; // Success metrics and monitoring
pub mod background_simulation; // Background simulation engine
pub mod queue_manager; // Queue management system
pub mod progress_communication; // Progress communication system
pub mod display_manager; // Display mode management
pub mod progress_ui; // Progress UI components
pub mod user_interaction; // User interaction flows
pub mod config; // Configuration system
pub mod storage; // Stub storage module
pub mod storage_manager; // Stub storage manager module
pub mod cache;
pub mod log_reproduction_test;
pub mod utils; // Utility functions for simulation results
pub mod seed_selection; // Seed selection algorithms for Two-Pass
pub mod simulation; // Core simulation execution functions
pub mod two_pass; // Two-Pass deterministic re-simulation system
pub mod memory_guardrails; // Memory safety protections for large simulations
pub mod wasm_api; // WASM bindings and JavaScript interface

// Re-export commonly used functions for external access
pub use simulation::{run_single_event_driven_simulation, run_single_lightweight_simulation, run_survey_pass};
pub use seed_selection::select_interesting_seeds_with_tiers;
pub use two_pass::{run_simulation_with_rolling_stats, run_simulation_with_three_tier};

// Re-export WASM API functions for backward compatibility
pub use wasm_api::*;