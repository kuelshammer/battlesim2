# Plan: Timeline-Based Short Rests

## Phase 1: Model Refactoring
Goal: Transition from "Short Rest After" checkbox to a discrete timeline event.

- [x] Task: Create `TimelineEvent` model (cc81010)
    - Define a new union type `TimelineEvent` (`Encounter | ShortRest`) in `src/model/model.ts`.
    - Update `AdventuringDay` type to use `TimelineEvent[]`.
- [ ] Task: Update Rust model for timeline
    - Implement `TimelineStep` enum in `simulation-wasm/src/model.rs`.
    - Update simulation entry points to accept a vector of timeline steps.

## Phase 2: UI Implementation
Goal: Add the "Short Rest Card" to the Adventuring Day Editor.

- [ ] Task: Update `AdventuringDayForm.tsx`
    - Remove the "Short Rest After" checkbox.
    - Add an "Add Short Rest" button.
    - Render a distinct card/item for Short Rests in the list.
- [ ] Task: Implement drag-and-drop or ordering
    - Ensure short rests can be placed between any two encounters.

## Phase 3: Simulation & Scoring Logic
Goal: Update the engine to process timeline steps sequentially.

- [ ] Task: Update `run_single_event_driven_simulation`
    - Iterate through `TimelineStep`s.
    - If combat: run simulation, update state, calculate score for *just* that combat.
    - If rest: apply recovery logic (Hit Dice, resource resets) and generate a "RestEvent" for scoring.
- [ ] Task: Implement Hit Die scoring penalty
    - Add a 15-point penalty per Hit Die spent during a rest event.
- [ ] Task: Conductor - User Manual Verification
