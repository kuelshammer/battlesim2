# Plan: Per-Encounter Decile Logs

## Phase 1: WASM Data Flow & Slicing [checkpoint: 7ad8c01]
Goal: Update the Rust backend to capture, sort, and slice events cumulatively for each encounter.

- [x] Task: Update `DecileStats` or a new struct to hold 11 event lists (logs) per encounter. (7ad8c01)
- [x] Task: Update `run_simulation_with_callback` to store events for all 2511 runs. (7ad8c01)
- [x] Task: Implement the "Cumulative Sorting" logic: for each encounter `i`, sort all runs by the score achieved from encounter `0` to `i`, then extract the 11 representative logs. (7ad8c01)
- [x] Task: Unit Test (Rust) - Verify that the 11 logs for Encounter 2 are correctly sorted by the combined score of E1+E2 and bounded correctly. (7ad8c01)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (7ad8c01)

## Phase 2: Frontend Infrastructure
Goal: Update the TypeScript interfaces and worker to handle the expanded log data.

- [ ] Task: Update Zod schemas in `model.ts` to support the new `decileLogs` mapping within encounter analysis.
- [ ] Task: Update `useSimulationWorker.ts` to store and expose the per-encounter log data.
- [ ] Task: Unit Test (Vitest) - Verify the simulation worker correctly parses the new multi-log structure.
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Modal UI & Navigation
Goal: Implement the log modal with decile navigation and percentile labels.

- [ ] Task: Update the Log Modal in `Simulation.tsx` to include "Worse Run" / "Better Run" buttons.
- [ ] Task: Add state management to the modal to track the currently viewed percentile (defaulting to 50% Median).
- [ ] Task: Update `EventLog.tsx` to display the selected percentile's sliced events.
- [ ] Task: Visual Verification - Confirm that clicking through deciles shows varying combat outcomes (e.g., more misses in low percentiles).
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
