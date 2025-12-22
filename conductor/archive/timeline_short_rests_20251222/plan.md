# Plan: Timeline-Based Short Rests

## Phase 1: Model Refactoring [checkpoint: fdceff1]
Goal: Transition from "Short Rest After" checkbox to a discrete timeline event.

- [x] Task: Create `TimelineEvent` model (fdceff1)
- [x] Task: Update Rust model for timeline (fdceff1)

## Phase 2: UI Implementation [checkpoint: fdceff1]
Goal: Add the "Short Rest Card" to the Adventuring Day Editor.

- [x] Task: Update `AdventuringDayForm.tsx` (fdceff1)
- [x] Task: Implement drag-and-drop or ordering (fdceff1)

## Phase 3: Simulation & Scoring Logic [checkpoint: fdceff1]
Goal: Update the engine to process timeline steps sequentially.

- [x] Task: Update `run_single_event_driven_simulation` (fdceff1)
- [x] Task: Implement Hit Die scoring penalty (fdceff1)
- [x] Task: Conductor - User Manual Verification (fdceff1)