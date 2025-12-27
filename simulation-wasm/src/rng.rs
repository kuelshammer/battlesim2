//! Deterministic RNG management for simulation tests
//!
//! This module provides thread-local RNG storage that can be optionally seeded
//! for deterministic simulation results. This enables:
//! - Reproducible tests
//! - Reliable snapshot testing
//! - Faster regression tests (fewer iterations needed)
//!
//! # Usage
//!
//! ```rust
//! use simulation_wasm::rng;
//! use rand::Rng;
//!
//! // Seed the RNG for deterministic results
//! rng::seed_rng(42);
//!
//! // Use the seeded RNG
//! let mut rng = rng::get_rng();
//! let roll = rng.gen_range(1..=20);
//!
//! // Clear the seed when done
//! rng::clear_rng();
//! ```

use rand::prelude::*;
use std::cell::RefCell;

/// Thread-local RNG storage
///
/// When seeded, this provides deterministic random numbers.
/// When None, falls back to entropy-based RNG.
thread_local! {
    static RNG: RefCell<Option<StdRng>> = RefCell::new(None);
}

/// Seed the thread-local RNG with the given seed value
///
/// After calling this, all `get_rng()` calls will return deterministic
/// random numbers based on this seed.
pub fn seed_rng(seed: u64) {
    RNG.with(|rng| {
        *rng.borrow_mut() = Some(StdRng::seed_from_u64(seed));
    });
}

/// Clear any seeded RNG, reverting to entropy-based randomness
///
/// Call this after running a seeded simulation to avoid affecting
/// subsequent operations.
pub fn clear_rng() {
    RNG.with(|rng| {
        *rng.borrow_mut() = None;
    });
}

/// Get a thread-local Rng instance
///
/// If a seed has been set via `seed_rng()`, returns a clone of the seeded RNG.
/// Otherwise, returns a new entropy-based RNG (like `thread_rng()`).
///
/// # Returns
/// A type implementing `Rng` that can be used for random number generation
pub fn get_rng() -> impl Rng {
    // We need to clone the seeded RNG out of the RefCell
    // This is safe because we're creating a new instance each time
    RNG.with(|rng_opt| {
        match &*rng_opt.borrow() {
            Some(seeded) => {
                // Clone the seeded RNG for this use
                seeded.clone()
            }
            None => {
                // Fall back to entropy-based RNG if no seed set
                StdRng::from_entropy()
            }
        }
    })
}
