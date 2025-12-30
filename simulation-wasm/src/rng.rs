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

/// A wrapper around the thread-local RNG that ensures state advancement
///
/// This struct implements `RngCore` and delegates all calls to the 
/// thread-local `RNG` storage. This ensures that if a seed is set, 
/// the same RNG state is shared and advanced across all calls within the thread.
pub struct ThreadLocalRng;

impl RngCore for ThreadLocalRng {
    fn next_u32(&mut self) -> u32 {
        RNG.with(|rng_opt| {
            let mut opt = rng_opt.borrow_mut();
            match opt.as_mut() {
                Some(rng) => rng.next_u32(),
                None => thread_rng().next_u32(),
            }
        })
    }

    fn next_u64(&mut self) -> u64 {
        RNG.with(|rng_opt| {
            let mut opt = rng_opt.borrow_mut();
            match opt.as_mut() {
                Some(rng) => rng.next_u64(),
                None => thread_rng().next_u64(),
            }
        })
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        RNG.with(|rng_opt| {
            let mut opt = rng_opt.borrow_mut();
            match opt.as_mut() {
                Some(rng) => rng.fill_bytes(dest),
                None => thread_rng().fill_bytes(dest),
            }
        })
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        RNG.with(|rng_opt| {
            let mut opt = rng_opt.borrow_mut();
            match opt.as_mut() {
                Some(rng) => rng.try_fill_bytes(dest),
                None => thread_rng().try_fill_bytes(dest),
            }
        })
    }
}

/// Get a thread-local Rng instance
///
/// Returns a `ThreadLocalRng` wrapper that provides deterministic sequences
/// if `seed_rng()` was called, or falls back to `thread_rng()`.
///
/// # Returns
/// A type implementing `Rng` that can be used for random number generation
pub fn get_rng() -> ThreadLocalRng {
    ThreadLocalRng
}
