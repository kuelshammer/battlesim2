# Plan: Simulation Optimization (Fast vs Precise)

## Phase 1: Backend & Worker Updates
Goal: Enable variable iteration counts in the simulation engine.

- [x] Task: Update WASM `run_simulation` to accept `iterations` as a parameter. (a8da9b7)
- [x] Task: Update `SimulationWorker` (Typescript) to support passing `iterations`. (a8da9b7)
- [x] Task: Unit Test - Verify WASM runs correctly with 31 iterations and produces valid deciles/logs. (a8da9b7)
- [ ] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: Frontend Integration
Goal: Implement the UI controls for Fast vs Precise modes.

- [ ] Task: Update `useSimulationWorker` hook to expose an interface for triggering specific run counts.
- [ ] Task: Modify `Simulation.tsx` to default to 31 runs (Fast Mode) on auto-simulate.
- [ ] Task: Add "Run Precise Simulation" button to `Simulation.tsx` that triggers 2511 runs.
- [ ] Task: Add visual indicator for current simulation mode (Fast vs Precise).
- [ ] Task: Implement logic to revert to Fast Mode (auto-run) upon any user edit.
- [ ] Task: Performance Tuning - Experiment with increasing Fast Mode iterations (e.g., 51, 101) while maintaining "snappy" feel.
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)
