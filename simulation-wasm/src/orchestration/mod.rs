//! Orchestration module - simulation coordination and state management
//!
//! This module contains:
//! - GUI integration logic
//! - Simulation runners (survey, selection, deep-dive)
//! - Global state management
//! - Complex simulation orchestration logic

pub mod gui;
pub mod runners;
pub mod simulation;
pub mod state;

pub use gui::*;
pub use runners::*;
pub use simulation::*;
pub use state::*;
