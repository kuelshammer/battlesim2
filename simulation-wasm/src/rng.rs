//! Deterministic RNG management for simulation tests
//!
//! This module provides thread-local RNG storage that can be optionally seeded
//! for deterministic simulation results. It also supports "Forced Rolls" for 
//! testing specific sequences.

use rand::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    static RNG: RefCell<Option<StdRng>> = const { RefCell::new(None) };
    static CURRENT_SEED: RefCell<u64> = const { RefCell::new(0) };
    static FORCED_ROLLS: RefCell<VecDeque<(u32, u32)>> = const { RefCell::new(VecDeque::new()) }; // (sides, value)
}

/// Seed the thread-local RNG with the given seed value
pub fn seed_rng(seed: u64) {
    RNG.with(|rng| {
        *rng.borrow_mut() = Some(StdRng::seed_from_u64(seed));
    });
    CURRENT_SEED.with(|s| {
        *s.borrow_mut() = seed;
    });
}

/// Get the current seed value that was last used to seed the RNG
pub fn get_current_seed() -> u64 {
    CURRENT_SEED.with(|s| *s.borrow())
}

/// Clear any seeded RNG and forced rolls
pub fn clear_rng() {
    RNG.with(|rng| {
        *rng.borrow_mut() = None;
    });
    clear_forced_rolls();
}

/// Force the next roll of a specific die size to return a specific value
pub fn force_roll(sides: u32, value: u32) {
    FORCED_ROLLS.with(|f| {
        f.borrow_mut().push_back((sides, value));
    });
}

/// Helper to force multiple d20 rolls
pub fn force_d20_rolls(rolls: Vec<u32>) {
    for r in rolls {
        force_roll(20, r);
    }
}

/// Clear all forced rolls
pub fn clear_forced_rolls() {
    FORCED_ROLLS.with(|f| {
        f.borrow_mut().clear();
    });
}

/// Roll a d20, respecting forced rolls if any
pub fn roll_d20() -> u32 {
    roll_dice(20)
}

/// Roll a die with N sides, respecting forced rolls if any
pub fn roll_dice(sides: u32) -> u32 {
    // Check if there is a forced roll for this die size
    let forced = FORCED_ROLLS.with(|f| {
        let mut queue = f.borrow_mut();
        // Look for the first forced roll that matches this die size
        if let Some(pos) = queue.iter().position(|&(s, _)| s == sides) {
            return Some(queue.remove(pos).unwrap().1);
        }
        None
    });

    if let Some(val) = forced {
        return val;
    }
    
    let mut rng = get_rng();
    rng.gen_range(1..=sides)
}

/// A wrapper around the thread-local RNG that ensures state advancement
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

pub fn get_rng() -> ThreadLocalRng {
    ThreadLocalRng
}
