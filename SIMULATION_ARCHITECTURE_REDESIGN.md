# Simulation Architecture Redesign: The "Action Phase" System

## 1. Philosophy & Goal

**"Build Your Own logic."**

The goal is **not** to hardcode every D&D 5e rule (spell components, carrying capacity, etc.). Instead, we aim to provide a **flexible, generic engine** that allows a user to "program" their character's strategy via the GUI.

If a user wants to simulate "Counterspell," they should be able to configure a Reaction with:
*   **Trigger:** `Event::EnemyCastsSpell`
*   **Requirement:** `Range <= 60`
*   **Cost:** `Reaction` + `Spell Slot`

The backend should simply interpret these generic constraints without knowing what "Counterspell" actually is. Strategy is defined by the user ordering their actions (priority list) and configuring smart triggers.

## 2. Problem Statement

The current simulation engine is too rigid and relies on "Magic Numbers" and hardcoded logic, preventing users from defining complex custom behaviors.

### Core Issues

1.  **Muddled `ActionSlot` Definition:**
    *   `ActionSlot` mixes **Cost** (Action, Bonus) with **Triggers** (WhenReducedTo0HP).
    *   *Limitation:* Users cannot create a custom action that costs a Reaction but triggers on a specific event (e.g., "Attack of Opportunity").

2.  **Rigid Execution Loop:**
    *   The engine strictly follows "Sort -> Pick 1 Action -> Pick 1 Bonus".
    *   *Limitation:* Users cannot define a workflow like "Attack -> If Hit, use Bonus Action (GWM) -> If Miss, use Bonus Action (Pommel Strike)."

3.  **Lack of "Events":**
    *   Triggers are hardcoded in Rust (`resolution.rs`).
    *   *Limitation:* A user cannot add a "Sentinel Feat" effect because there is no way to tell the engine "Stop enemy movement on Hit" without editing source code.

4.  **Disconnected Resource Tracking:**
    *   Currently, `remaining_uses` is often tied to the Action ID itself (e.g., "Bless: 2/day").
    *   *Limitation:* It fails to model **Shared Resources** (e.g., Casting *Bless* should deplete a slot available for *Shield*) or **Granular Pools** (e.g., *Lay on Hands* has 20 points, not "3 uses").

---

## 3. Proposed Architecture

We will transition to a **Phase-Based Execution** model driven by **Generic Constraints**.

### A. The New Data Model

We need to separate **Cost**, **Timing**, and **Requirements**.

#### 1. `ActionCost` (The "Price")
What does this action consume? Configurable list.
*   `Action`, `BonusAction`, `Reaction`
*   `Movement` (amount)
*   `Resource` (Slot Level 1, Rage Charge, Superiority Die)
*   `Free`

#### 2. `ActionTiming` (The "When")
When can this be used?
*   `MainPhase`: Active use during turn.
*   `Reaction`: In response to an Event.
*   `Free`: Anytime conditions are met (e.g., dropping a weapon).

#### 3. `ActionRequirement` (The "Prerequisites")
A list of generic checks the user can add to any action.
*   `TargetCondition(IsEnemy, IsAlive)`
*   `SelfCondition(HasResource: Rage)`
*   `LastEvent(Type: MeleeHit, Value > 10)`
*   `CustomTag(TargetHasTag: "Marked")`

### B. The Turn Manager (The "Loop")

The User defines a **Priority List** of actions. The engine simply runs a loop:

1.  **Start Phase:** Reset counters.
2.  **Action Loop:**
    *   Iterate through User's Action List from Top to Bottom.
    *   Check **Requirements** & **Cost** for each.
    *   Execute the **first valid action**.
    *   Repeat until no actions are valid or explicit "End Turn".
3.  **End Phase:** Cleanup.

*Why this works:* It puts the strategy in the User's hands. If they want to prioritize "Heal" over "Attack", they just drag "Heal" to the top of the list. The engine doesn't need to "decide."

### C. The Trigger System (The "Bus")

Characters have a **Reaction Priority List**.

*   **Events:** The engine emits generic events: `Event::ActionStarted`, `Event::AttackHit`, `Event::DamageTaken`, `Event::UnitDyed`.
*   **Listeners:** When an event occurs, the engine pauses and checks the active character's (and others') Reaction Lists.
*   **Resolution:** If a Reaction's **Trigger** matches the Event and **Requirements** are met, it executes.

### D. The Resource Manager

We introduce a centralized **Resource Ledger** separate from Actions.

#### 1. Resource Types
*   **Discrete Pools (Shared):**
    *   *Example:* Spell Slots. A character has `{ Level1: 4, Level2: 3 }`.
    *   *Usage:* Action "Bless" defines `Cost: { Type: "SpellSlot", Level: 1 }`. Using it decrements the `Level1` pool.
    *   *Impact:* Casting "Bless" automatically prevents casting "Shield" if the pool reaches 0.
*   **Granular Pools (Variable):**
    *   *Example:* Lay on Hands (HP Pool), Sorcery Points.
    *   *Usage:* Action "Lay on Hands" defines `Cost: { Type: "Pool", ID: "LayOnHands", Amount: "Variable" }`.
    *   *Runtime:* The Action (or AI) specifies the amount to spend (e.g., `5`). The Ledger subtracts `5` from the total (e.g., `20 -> 15`).

#### 2. Planning & Validation (Frontend)
Since resources are shared, the Frontend can validate the User's Strategy **before** simulation:
*   **Simulation Preview:** The UI calculates total potential cost.
    *   *Scenario:* User adds "Bless" (Lvl 1) and "Shield" (Lvl 1) to their plan. User only has 1 Slot.
    *   *Feedback:* The UI warns: "Potential Conflict: Actions require 2 Level 1 Slots, but only 1 is available."
*   **Upcasting Logic:** The backend can support `AllowUpcast: true`. If Lvl 1 slots are empty, it automatically consumes a Lvl 2 slot (if configured).

---

## 4. Implementation Plan (Revised)

This is a significant refactor. We must build from the bottom up: **Data -> Resources -> Events -> Execution**.

### Phase 1: Core Ontology (Resources & Costs) [The Foundation]
*   **[👉 DETAILED PLAN: Phase 1 Implementation](PHASE_1_IMPLEMENTATION_PLAN.md)**
*   **Goal:** Define the "Currency" of the system.
*   **Tasks:**
    *   Create `ResourceLedger` struct: A `HashMap<String, f64>` (e.g., `"SpellSlot_1": 4.0`, `"HP_Pool": 20.0`).
    *   Create `ActionCost` enum: `{ Type: Resource | Action | Bonus | Reaction, Amount: f64 }`.
    *   Create `ActionRequirement` enum.
    *   **Why First?** The execution engine cannot be written if it doesn't know how to check/deduct costs.

### Phase 2: Action Data Structure & Frontend Creation [The New Standard]
*   **[👉 DETAILED PLAN: Phase 2 Implementation](PHASE_2_IMPLEMENTATION_PLAN.md)**
*   **Goal:** Define the *new* action data structures in both backend and frontend, enabling users to immediately create flexible, programmable actions via the GUI. Old monster/spell JSON data will **not** be automatically migrated and will require manual re-creation or updating.
*   **Tasks:**
    *   **Backend (Rust):**
        *   Finalize the definition of `Action`, `ActionBase`, `ActionCost`, `ActionRequirement`, `ActionTag` structs/enums based on Phase 1's ontology.
        *   Implement `serde` for these new, cleaner structures.
        *   Remove or deprecate `actionSlot` from `ActionBase` and related types.
    *   **Frontend (Typescript):**
        *   Update TypeScript types (`src/model/model.ts`, `src/model/enums.ts`) to match the new backend structures.
        *   Modify the action creation/editing GUI components (`src/components/creatureForm/actionForm.tsx`, `src/data/actions.ts` for templates) to allow users to specify `costs`, `requirements`, and `tags` for new actions.
    *   **Old JSON Handling:**
        *   Explicitly communicate that existing `.json` files for monsters and pre-defined actions (e.g., in `src/data/monsters.ts`, `src/data/actions.ts`, `kaelen_battle.json`) will become incompatible. Users will need to manually re-create or edit these using the new flexible UI.
        *   This simplifies the backend deserialization, as no complex adapter for old `actionSlot` values is needed.
*   **Why Second?** This sets up the new action definition standard. The backend can now load actions in the new format, and the GUI can build them. We accept temporary data loss for immediate flexibility.

### Phase 3: Event Bus & Context [The Nervous System]
*   **Goal:** Define the state that actions interact with.
*   **Tasks:**
    *   Define `Event` enum: `AttackHit`, `DamageTaken`, `SpellCast`.
    *   Create `TurnContext` struct:
        *   `ledger: ResourceLedger`
        *   `history: Vec<Event>`
        *   `active_effects: Vec<Effect>`
    *   Implement "Event Broadcasting" (just the collection/dispatch mechanism, not the listeners yet).

### Phase 4: The Execution Engine [The Brain]
*   **Goal:** Rewrite `execute_turn` to use the Phase-Based Loop.
*   **Tasks:**
    *   Implement the **Option Generation** step: `get_actions()` now checks `Ledger` vs `Action.costs` and `Context` vs `Action.requirements`.
    *   Implement the **Loop**:
        1.  `options = generate_options(context)`
        2.  `choice = ai.select(options)`
        3.  `context.ledger.deduct(choice.cost)`
        4.  `result = resolve(choice)`
        5.  `context.emit(result.events)` -> **Trigger Reaction Check**
    *   **Reaction Check:** Pause loop, check other characters' generic triggers against the new Event.

### Phase 5: Frontend Adaptation [The Face]
*   **Goal:** Expose the flexibility to the user.
*   **Tasks:**
    *   Update `ActionTemplates` to use the new schema.
    *   UI: Show "Resources" panel (Spell Slots, Pools).
    *   UI: "Strategy" builder (Ordering actions, adding Requirements).

---

## 5. Risks & Mitigations

*   **Infinite Loops:** The AI might keep selecting "Free" actions (like dropping a weapon).
    *   *Mitigation:* Limit "Free" actions per turn or ensure they strictly change state (prevent toggling).
*   **Performance:** Event bus might be slow if not optimized.
    *   *Mitigation:* Direct function calls for common triggers (like basic attacks), use Events only for conditional interrupts (Shield, Counterspell).
*   **Complexity:** Debugging "Why did the AI do that?" becomes harder.
    *   *Mitigation:* Enhanced logging. "AI Actions Available: [A, B, C]. Selected A because..."

## 6. Audit Findings (Hardcoded Dependencies)

A search of the codebase revealed specific hardcoded dependencies on `ActionSlot` integers that must be refactored:

*   **Pre-Combat Logic:** `simulation-wasm/src/simulation.rs` checks for `actionSlot == -3` to execute pre-combat actions. This should be replaced by a `Timing::PreCombat` check.
*   **Reaction Logic:** `simulation-wasm/src/resolution.rs` checks `if cost_slot == (ActionSlot::Reaction as i32)`. This should use `ActionCost::Reaction`.
*   **Data Files:** `src/data/actions.ts`, `src/data/monsters.ts`, and `kaelen_battle.json` contain thousands of hardcoded integer values (0, 1, 2, -1, -3). A migration script or backward-compatibility layer in the deserializer will be required.