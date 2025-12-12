# Legacy Code Removal Guide

## Objective
Remove the deprecated "legacy" simulation engine (primarily located in `src/simulation/mod.rs` or `src/simulation.rs`) while ensuring the project remains compile-steady and fully functional. The new "event-driven" simulation engine (`src/execution.rs`, `src/action_resolver.rs`, etc.) MUST replace it entirely.

## Guidelines for the AI Agent
1.  **Safety First:** Do not delete code until all references to it have been removed.
2.  **Compile & Test Often:** After EVERY change (even small ones), run `cargo check` and `cargo test`.
    *   If a step breaks compilation, **UNDO IT IMMEDIATELY** and rethink the approach.
3.  **Regular Commits:** Commit your progress after every successful check. Use descriptive messages.
4.  **No Logic Changes:** The goal is refactoring/cleanup, not behavior modification. The external API (`run_simulation_wasm`) must essentially do the same thing (run a simulation), but using the *new* engine under the hood.

---

## Phase 1: Redirect WASM Entry Point

The public function `run_simulation_wasm` in `src/lib.rs` currently calls the legacy engine (`simulation::run_monte_carlo`). We must redirect this to use the new event-driven engine.

### Step 1.1: Analyze Interfaces
*   Compare `simulation::run_monte_carlo` outputs vs `run_event_driven_simulation_rust`.
*   Note that `run_simulation_wasm` takes `JsValue` inputs and returns `JsValue`.

### Step 1.2: Refactor `run_simulation_wasm`
*   **Goal:** Modify `run_simulation_wasm` in `src/lib.rs` to call `run_event_driven_simulation_rust` instead of `simulation::run_monte_carlo`.
*   **Action:**
    1.  Parse inputs (Players, Encounters) - *Already done, keep as is.*
    2.  Call `run_event_driven_simulation_rust(players, encounters, iterations, false)`.
    3.  Map the results to the expected return format.
*   **Verification:** Run `cargo check`. If compilation flows, proceed.

### Step 1.3: Verify CLI
*   Ensure `src/bin/sim_cli.rs` is NOT using `simulation` module anymore. (It should be using `run_event_driven_simulation_rust` already, but verify imports).

## Phase 2: Decouple Dependencies

Now that the main entry point no longer calls legacy code, we can remove the module link.

### Step 2.1: Remove Module Usage in `lib.rs`
*   In `src/lib.rs`, find `pub mod simulation;`.
*   Comment it out: `// pub mod simulation;`.
*   **Verification:** Run `cargo check`.
    *   **Failure Expected:** You might find other modules (`tests`, benchmarks) referencing `crate::simulation`.
    *   **Fix:** If tests rely on legacy simulation, mark them as `#[ignore]` or update them to use the new engine. If specific internal helper functions are needed, consider moving them to a shared utility module *before* deleting.

### Step 2.2: Fix Tests
*   Run `cargo test`.
*   If `tests/` directory contains integration tests using the legacy engine, refactor them to use `run_event_driven_simulation_rust`.

## Phase 3: Delete Legacy Code

Once `cargo check` passes with `mod simulation` commented out, the code is dead.

### Step 3.1: Delete Files
*   Delete `src/simulation/` directory.
*   Delete `src/simulation.rs` (if it exists as a file instead of a dir).

### Step 3.2: Remove Module Declaration
*   Remove the commented-out line `// pub mod simulation;` from `src/lib.rs`.

### Step 3.3: Final Verification
*   Run `cargo clean`.
*   Run `cargo build`.
*   Run `cargo test`.
*   Run `cargo run --bin sim_cli -- log test_bless_bane.json` (or similar) to ensure the CLI still works.

## Phase 4: Cleanup & Optimization

### Step 4.1: Remove Unused Imports
*   Check `src/lib.rs` and other files for unused imports (e.g., `use crate::simulation::...`).
*   The compiler warnings will guide you.

### Step 4.2: Audit Data Models
*   Check `src/model.rs`. Are there structs or fields that were ONLY used by the legacy simulation?
    *   *Example:* Old action resolution structs that aren't used by `action_resolver.rs`.
    *   *Caution:* Only delete if you are 100% sure. If in doubt, leave them.

---

## Final Output
A project structure where `src/simulation` does not exist, and all simulation runs (WASM and CLI) use the new event-driven architecture.
