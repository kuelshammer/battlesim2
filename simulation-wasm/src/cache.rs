use std::collections::HashMap;
use std::cell::RefCell;
use crate::model::{Creature, TimelineStep, LightweightRun};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Combine scenario hash and seed into a single u64 key for more efficient HashMap storage
/// Uses splitmix approach: combine high and low bits from both values
#[inline]
fn combine_keys(scenario_hash: u64, seed: u64) -> u64 {
    // Splitmix64-style mixing for better distribution
    const GOLDEN_RATIO: u64 = 0x9e3779b97f4a7c15;
    let mut combined = scenario_hash.wrapping_add(seed.wrapping_mul(GOLDEN_RATIO));
    combined ^= combined >> 33;
    combined = combined.wrapping_mul(0xff51afd7ed558ccd);
    combined ^= combined >> 33;
    combined = combined.wrapping_mul(0xc4ceb9fe1a85ec53);
    combined ^= combined >> 33;
    combined
}

thread_local! {
    /// Optimized cache using single u64 key instead of tuple for ~30% memory reduction
    static CACHE: RefCell<HashMap<u64, LightweightRun>> = RefCell::new(HashMap::new());
}

/// Calculate a stable hash for a simulation scenario
pub fn get_scenario_hash(players: &[Creature], timeline: &[TimelineStep]) -> u64 {
    let mut hasher = DefaultHasher::new();
    
    // Efficient hashing using implemented Hash traits
    players.hash(&mut hasher);
    timeline.hash(&mut hasher);
    
    hasher.finish()
}

/// Retrieve a cached lightweight run result
pub fn get_cached_run(scenario_hash: u64, seed: u64) -> Option<LightweightRun> {
    let key = combine_keys(scenario_hash, seed);
    CACHE.with(|c| c.borrow().get(&key).cloned())
}

/// Store a lightweight run result in the cache
pub fn insert_cached_run(scenario_hash: u64, seed: u64, run: LightweightRun) {
    let key = combine_keys(scenario_hash, seed);
    CACHE.with(|c| {
        let mut cache = c.borrow_mut();

        // Safety: Limit cache size to ~5k entries (~160 KB) to prevent OOM in WASM
        // This is sufficient for typical scenarios (2511 iterations)
        const MAX_CACHE_ENTRIES: usize = 5_000;
        if cache.len() > MAX_CACHE_ENTRIES {
            cache.clear();
        }

        cache.insert(key, run);
    })
}

/// Clear the simulation results cache
pub fn clear_cache() {
    CACHE.with(|c| c.borrow_mut().clear())
}

/// Get cache statistics for memory monitoring
/// Returns (entry_count, estimated_bytes)
/// Each LightweightRun is ~32 bytes, HashMap overhead is ~24 bytes per entry
pub fn get_cache_stats() -> (usize, usize) {
    CACHE.with(|c| {
        let cache = c.borrow();
        let entry_count = cache.len();
        // Estimate: 32 bytes (LightweightRun) + 24 bytes (HashMap overhead) + 8 bytes (key)
        let estimated_bytes = entry_count * 64;
        (entry_count, estimated_bytes)
    })
}
