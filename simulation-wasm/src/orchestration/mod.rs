//! Orchestration module - simulation coordination and state management
//!
//! This module contains:
//! - GUI integration logic
//! - Simulation runners (survey, selection, deep-dive)
//! - Global state management
//! - Complex simulation orchestration logic
//! - Auto-balance and encounter adjustment logic

pub mod balancer;
pub mod gui;
pub mod runners;
pub mod simulation;
pub mod state;

pub use balancer::*;
pub use gui::*;
pub use runners::*;
pub use simulation::*;
pub use state::*;
