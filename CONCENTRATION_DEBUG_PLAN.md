# Debugging Plan: Concentration & Buff Display Issues

## Problem Description

![User Screenshot](/Users/max/.gemini/antigravity/brain/b60f5142-74a1-4f1e-96df-918832237f9d/uploaded_image_1764530429248.png)

**Symptoms**:
1. ✅ Concentration icon (brain 🧠) correctly removed when Acolyte dies
2. ❌ Hover tooltip shows Acolyte still "concentrating" on Bless
3. ❌ Bless buffs (+1d4) not displayed on targets
4. ❌ Same issue with Bane (-1d4) from second caster

**Hypothesis**: Data is being cleaned in aggregation, but GUI is reading from a different field OR aggregation is incomplete.

---

## Testing Strategy

We'll test in **3 layers** to isolate the issue:

### Layer 1: Raw WASM Output (Bypass GUI)
### Layer 2: Single vs Aggregated Runs
### Layer 3: Frontend Data Mapping

---

## Layer 1: Raw WASM Output Analysis

**Goal**: Verify WASM is producing correct data

### Test 1.1: Console Log Raw Aggregated Results

**File**: `src/components/simulation/simulation.tsx`

Add logging BEFORE any processing:

```typescript
const aggregatedRounds = await wasm.aggregate_simulation_results(
  JSON.stringify(rawResults)
);
console.log('🔍 RAW AGGREGATED RESULTS:', JSON.parse(aggregatedRounds));
```

**What to check**:
- For each round where Acolyte is dead (`HP < 0.5`):
  - `concentrating_on`: Should be `null`
  - `buffs`: Should be empty object `{}`
- For Fighter (target):
  - `buffs`: Should NOT contain Bless if Acolyte is dead

**Expected**: Clean data from Rust
**If fails**: Bug is in aggregation (Rust)
**If passes**: Bug is in GUI mapping (TypeScript)

---

### Test 1.2: Create Rust Debug Example

**File**: `simulation-wasm/examples/test_concentration_display.rs` (NEW)

Create a minimal test that:
1. Acolyte casts Bless on Fighter
2. Run 10 simulations
3. Aggregate results
4. Print EVERY round's state for Acolyte and Fighter

```rust
use simulation_wasm::*;

fn main() {
    let caster = /* Acolyte with Bless */;
    let target = /* Fighter */;
    let enemy = /* Goblin */;
    
    let results = simulation::run_monte_carlo(&[caster, target], &[enemy], 10);
    let aggregated = aggregation::aggregate_results(&results);
    
    for (i, round) in aggregated.iter().enumerate() {
        println!("\n=== ROUND {} ===", i + 1);
        for c in &round.team1 {
            println!("{}: HP={:.1}, Concentrating={:?}, Buffs={:?}", 
                c.creature.name, 
                c.final_state.current_hp,
                c.final_state.concentrating_on,
                c.final_state.buffs.keys().collect::<Vec<_>>()
            );
        }
    }
}
```

**What to check**:
- When Acolyte HP < 0.5:
  - `concentrating_on` = `None`
  - `buffs` keys = empty
- Fighter:
  - No "bless" key in buffs after Acolyte dies

**Run**: `cargo run --example test_concentration_display`

---

## Layer 2: Single Run vs Aggregated Comparison

**Goal**: Determine if aggregation logic is correctly handling the cleanup

### Test 2.1: Compare Single Run to Aggregated

Modify dev server to log BOTH:

```typescript
// Single run (first result)
const firstRun = JSON.parse(rawResults)[0];
console.log('📊 FIRST RUN (Round where Acolyte dies):', 
  firstRun.rounds[roundWhereDead]);

// Aggregated
const aggregated = await wasm.aggregate_simulation_results(
  JSON.stringify(rawResults)
);
console.log('📊 AGGREGATED (Same round):', 
  JSON.parse(aggregated)[roundWhereDead]);
```

**Compare**:
1. **Single run**: Acolyte should have clean state (our fix works mid-combat)
2. **Aggregated**: Should match single run

**If single run clean but aggregated dirty**: Aggregation bug
**If both dirty**: Combat simulation bug (our fix didn't work)

---

## Layer 3: Frontend Data Mapping

**Goal**: Verify GUI correctly reads WASM data

### Test 3.1: Check Tooltip Data Source

**File**: Find where hover tooltip gets `concentrating` data

Search for:
```bash
grep -r "concentrat" src/components/
```

**What to check**:
- Is it reading from `combatant.concentrating_on`?
- Or from `combatant.initial_state.concentrating_on`?
- Or somewhere else?

**Hypothesis**: Tooltip might be reading `initial_state` instead of `final_state`

---

### Test 3.2: Check Buff Display Logic

**File**: Find where buffs (+1d4) are displayed

Search for:
```bash
grep -r "buffs" src/components/
grep -r "d4" src/components/
```

**What to check**:
- Does it iterate over `combatant.final_state.buffs`?
- Or `combatant.buffs`?
- Is there a filter that might hide valid buffs?

---

## Layer 4: Edge Case Testing

### Test 4.1: Verify Concentration Clearing Logic

Check if aggregation is correctly identifying dead casters:

**File**: `simulation-wasm/src/aggregation.rs:202`

Add debug logging:
```rust
for c in t1.iter_mut().chain(t2.iter_mut()) {
    if c.final_state.current_hp < 0.5 {
        eprintln!("🪦 DEAD: {} HP={:.1}, Had Concentration: {:?}", 
            c.creature.name, 
            c.final_state.current_hp,
            c.final_state.concentrating_on
        );
        if c.final_state.concentrating_on.is_some() {
            c.final_state.concentrating_on = None;
        }
        dead_source_ids.insert(c.id.clone());
    }
}
```

Run example and check logs for "🪦 DEAD" messages.

---

### Test 4.2: Verify Buff Removal Pass

Check if buffs are actually being removed:

**File**: `simulation-wasm/src/aggregation.rs:228`

Already has debug logging. Enable with:
```bash
RUST_LOG=debug cargo run --example test_concentration_display
```

Look for:
```
AGGREGATION: PASS1: Removing buff bless from Fighter (source acolyte-0-0 is dead)
```

If you DON'T see these messages, the cleanup isn't triggering.

---

## Systematic Execution Plan

### Phase 1: Quick Checks (5 minutes)
1. Open browser console
2. Run simulation with Acolyte + Fighter + Goblin
3. Check console for raw aggregated JSON
4. Look at `concentrating_on` and `buffs` fields

**Decision Point**:
- If clean in console → GUI bug (go to Phase 3)
- If dirty in console → Rust bug (go to Phase 2)

---

### Phase 2: Rust Verification (10 minutes)
1. Create `test_concentration_display.rs` example
2. Run with `cargo run --example test_concentration_display`
3. Examine printed output for dead rounds
4. Check if `concentrating_on` is `None` and `buffs` is empty

**Decision Point**:
- If clean → Aggregation works, GUI problem
- If dirty → Combat simulation or aggregation problem

---

### Phase 3: Frontend Debug (15 minutes)
1. Search codebase for tooltip component
2. Find where it reads `concentrating_on`
3. Verify it's reading `final_state` not `initial_state`
4. Check buff display component reads correct field

**Fix**: Update component to read correct data field

---

### Phase 4: Deep Dive (if needed)
1. Enable Rust debug logging
2. Run simulation with logging
3. Trace buff cleanup messages
4. Verify dead_source_ids contains correct IDs

---

## Expected Outcomes

### Scenario A: Console shows clean data
**Root Cause**: Frontend mapping issue
**Fix**: Update tooltip/buff components to read `final_state`

### Scenario B: Console shows dirty data, single run clean
**Root Cause**: Aggregation not applying cleanup
**Fix**: Debug aggregation cleanup passes

### Scenario C: Both console and single run dirty
**Root Cause**: Combat simulation not removing buffs
**Fix**: Debug `remove_all_buffs_from_source` calls

---

## Quick Start Command

```bash
# Run this first to see WASM output
cd simulation-wasm
cargo run --example test_buff_cleanup 2>&1 | grep "Concentrating\|Buffs"
```

This will immediately show if Rust is producing clean data.
