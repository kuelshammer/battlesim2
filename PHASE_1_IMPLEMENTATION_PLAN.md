# Phase 1 Implementation Plan: Core Ontology & Resources

## 🔗 Context
This document details the step-by-step implementation for **Phase 1** of the [Simulation Architecture Redesign](SIMULATION_ARCHITECTURE_REDESIGN.md).

**Goal:** Define the "Currency" of the system. Establish the fundamental data structures (`ResourceLedger`, `ActionCost`, `ActionRequirement`) that will drive the new execution engine.

---

## 🛠️ Step-by-Step Implementation

### Step 1: Define the `Resource` Ontology
**File:** `simulation-wasm/src/model.rs` (or a new `resources.rs` module)

We need a way to uniquely identify resources.

1.  Create `ResourceType` Enum:
    *   `Primary`: Action, BonusAction, Reaction.
    *   `Movement`: Feet (f64).
    *   `SpellSlot(u8)`: Level 1-9.
    *   `ClassResource(String)`: "Rage", "Ki", "SorceryPoints".
    *   `ItemCharge(String)`: "Wand of Magic Missiles".
    *   `HitDice(u8)`: Die size (d6, d8, etc.).

2.  Implement `Hash` and `Eq` for `ResourceType` so it can be used as a HashMap key.

### Step 2: Create the `ResourceLedger`
**File:** `simulation-wasm/src/model.rs`

This struct tracks *what a character has*.

1.  Define `ResourceLedger` struct:
    *   `current: HashMap<ResourceType, f64>`
    *   `max: HashMap<ResourceType, f64>` (for resets)
    *   `reset_rules: HashMap<ResourceType, ResetType>` (ShortRest, LongRest)

2.  Implement Methods:
    *   `pub fn has(&self, resource: &ResourceType, amount: f64) -> bool`
    *   `pub fn consume(&mut self, resource: &ResourceType, amount: f64) -> Result<(), Error>`
    *   `pub fn restore(&mut self, resource: &ResourceType, amount: f64)`
    *   `pub fn reset(&mut self, reset_type: ResetType)`

### Step 3: Define `ActionCost`
**File:** `simulation-wasm/src/model.rs`

What does an action *cost*?

1.  Define `ActionCost` Struct/Enum:
    *   `Discrete(ResourceType, f64)`: e.g., `(SpellSlot(1), 1.0)`
    *   `Variable(ResourceType, f64, f64)`: Min/Max spend (e.g., Lay on Hands 1-20).

### Step 4: Define `ActionRequirement`
**File:** `simulation-wasm/src/model.rs`

Prerequisites for using an action (distinct from cost).

1.  Define `ActionRequirement` Enum:
    *   `ResourceAvailable(ResourceType, f64)`: Check without consuming.
    *   `CombatState(CombatCondition)`: e.g., "EnemyInRange", "IsSurprised".
    *   `StatusEffect(String)`: e.g., "SelfHasCondition(Invisible)".

### Step 5: Unit Tests (Verification)
**File:** `simulation-wasm/src/resources_test.rs`

We must verify the logic *in isolation* before integrating.

1.  **Test Ledger Basic:**
    *   Initialize Ledger with `Action: 1, SpellSlot(1): 4`.
    *   Assert `has(Action, 1)` is true.
    *   `consume(Action, 1)` -> Success.
    *   `has(Action, 1)` -> False.
    *   `consume(Action, 1)` -> Error.

2.  **Test Variable Cost:**
    *   Initialize `LayOnHands: 20`.
    *   Check if `ActionCost::Variable` logic works (backend helper).

3.  **Test Reset:**
    *   Consume resources.
    *   Call `reset(LongRest)`.
    *   Assert values return to max.

---

## ✅ Definition of Done (Phase 1)

Phase 1 is complete when:

1.  [ ] `ResourceType`, `ResourceLedger`, `ActionCost`, and `ActionRequirement` structs are compiled in `simulation-wasm`.
2.  [ ] A new unit test file `resources_test.rs` passes all checks for consumption, restoration, and insufficient resource handling.
3.  [ ] **No changes** are made to the existing `execute_turn` loop yet (this is purely data structure setup).
4.  [ ] The code is committed and builds without warnings.
