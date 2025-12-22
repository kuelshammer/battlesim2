# Plan: Efficiency-Based Performance Scoring

## Phase 1: Foundation
Goal: Prepare the model and aggregation logic for complex scoring.

- [ ] Task: Update `SimulationResult` model
    - Add `score: Option<f64>` to `SimulationRunData` in `simulation-wasm/src/model.rs`.
- [ ] Task: Extend `calculate_score` logic
    - Create a new `calculate_efficiency_score` function in `aggregation.rs` that takes both `SimulationResult` and `Vec<Event>`.

## Phase 2: Algorithm Implementation
Goal: Implement the weighted resource penalty system.

- [ ] Task: Implement Resource Penalty Calculation
    - Spell Slots: `15 * (level ^ 1.6)`
    - Long Rest Class Features: 40 points
    - Short Rest Class Features: 20 points
    - Consumables/Potions: 20 points
- [ ] Task: Integrate Scoring in Simulation Loop
    - Update `run_single_event_driven_simulation` in `lib.rs` to compute and store the score.

## Phase 3: Integration & Verification
Goal: Ensure the system sorts runs correctly by efficiency.

- [ ] Task: Update Sorting Logic
    - Use the pre-computed `score` for all decile analysis sorting.
- [ ] Task: Verify Efficiency-Based Sorting
    - Create a test case where a run with fewer resources spent scores higher than one with more resources spent, despite identical HP.
- [ ] Task: Conductor - User Manual Verification
