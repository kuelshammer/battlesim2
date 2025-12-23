# Plan: Simulation Optimization (Fast vs Precise)

## Phase 1: Backend & Worker Updates [checkpoint: 8ea3bcd]
Goal: Enable variable iteration counts in the simulation engine.

- [x] Task: Update WASM `run_simulation` to accept `iterations` as a parameter. (a8da9b7)
- [x] Task: Update `SimulationWorker` (Typescript) to support passing `iterations`. (a8da9b7)
- [x] Task: Unit Test - Verify WASM runs correctly with 31 iterations and produces valid deciles/logs. (a8da9b7)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (8ea3bcd)

## Phase 2: Frontend Integration [checkpoint: 8193ea5]
Goal: Implement the UI controls for Fast vs Precise modes.

- [x] Task: Update `useSimulationWorker` hook to expose an interface for triggering specific run counts. (8193ea5)
- [x] Task: Modify `Simulation.tsx` to default to 31 runs (Fast Mode) on auto-simulate. (8193ea5)
- [x] Task: Add "Run Precise Simulation" button to `Simulation.tsx` that triggers 2511 runs. (8193ea5)
- [x] Task: Add visual indicator for current simulation mode (Fast vs Precise). (8193ea5)
- [x] Task: Implement logic to revert to Fast Mode (auto-run) upon any user edit. (8193ea5)
- [x] Task: Performance Tuning - Experiment with increasing Fast Mode iterations (e.g., 51, 101) while maintaining "snappy" feel. (8193ea5)
- [x] Task: Conductor - User Manual Verification 'Phase 2' (8193ea5)
