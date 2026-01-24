pub mod action_cache;
pub mod action_resolver;
pub mod actions;
pub mod aggregation;
pub mod analysis;
pub mod api;
pub mod cleanup;
pub mod context;
pub mod dice;
pub mod enums;
pub mod events;
pub mod execution;
pub mod model;
pub mod orchestration;
pub mod reactions;
pub mod resolvers;
pub mod resources;
pub mod rng;
pub mod sorting;
pub mod targeting;
pub mod validation; // New module for requirement validation
                    // Re-export decile_analysis for backward compatibility
pub mod decile_analysis {
    pub use crate::analysis::*;
}

pub mod auto_balancer;
pub mod background_simulation; // Background simulation engine
pub mod combat_stats;
pub mod config;
pub mod creature_adjustment;
pub mod dice_reconstruction;
pub mod display_manager; // Display mode management
pub mod encounter_balancer;
pub mod enhanced_validation; // Comprehensive validation
pub mod error_handling; // Enhanced error handling system
pub mod intensity_calculation;

pub mod monitoring; // Success metrics and monitoring
pub mod percentile_analysis;
pub mod progress_communication; // Progress communication system
pub mod progress_ui; // Progress UI components
pub mod queue_manager; // Queue management system
pub mod recovery; // Error recovery mechanisms
pub mod safe_aggregation; // Safe aggregation functions

pub mod strategic_assessment;
pub mod user_interaction; // User interaction flows // Configuration system


pub mod cache;

pub mod memory_guardrails; // Memory safety protections for large simulations
pub mod seed_selection; // Seed selection algorithms for Two-Pass
pub mod simulation; // Core simulation execution functions
pub mod two_pass; // Two-Pass deterministic re-simulation system
pub mod utils; // Utility functions for simulation results
pub mod wasm_api; // WASM bindings and JavaScript interface

// Re-export commonly used functions for external access
pub use api::runner::{
    run_single_event_driven_simulation, run_single_lightweight_simulation, run_survey_pass,
};
pub use seed_selection::select_interesting_seeds_with_tiers;
pub use two_pass::{run_simulation_with_rolling_stats, run_simulation_with_three_tier};

// Re-export WASM API functions for backward compatibility
pub use wasm_api::*;

// Re-export orchestration modules for internal use
pub use orchestration::*;
