# Plan: Daily Pacing Dashboard

## Phase 1: Backend Data & Step Tracking [checkpoint: 879fc82]
Goal: Ensure the simulation result contains the necessary time-series data for the graph.

- [x] Task: Update `DecileStats` to store a `resource_timeline` (array of EHP % after each step) in `decile_analysis.rs`. (879fc82)
- [x] Task: Update `analyze_results` to populate the timeline for each decile. (879fc82)
- [x] Task: Unit Test - Verify that a 3-encounter day returns a timeline with 4 points (Start, E1, E2, E3). (879fc82)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md) (879fc82)

## Phase 2: UI Components (Gauges & Badges) [checkpoint: e43aacd]
Goal: Implement the core pacing visuals.

- [x] Task: Create `DeltaBadge` component with 5-zone logic. (a8da9b7)
- [x] Task: Create `FuelGauge` component (Plan vs. Reality segments). (0bfa02a)
- [x] Task: Integrate components into `Simulation.tsx` and `EncounterResult.tsx`. (e049873)
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: The Descent Graph (SVG)
Goal: Visualize attrition and risk.

- [x] Task: Implement `DescentGraph.tsx` using raw SVG. (b76a7ed)
- [x] Task: Add shaded region for 25th–75th percentiles. (b76a7ed)
- [x] Task: Add dotted "Plan" line calculation. (b76a7ed)
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)

## Phase 4: Natural Language Summary & Polishing
Goal: Add the "Assistant" voice and final styling.

- [ ] Task: Implement `AssistantSummary` logic.
- [ ] Task: Add cumulative drift calculation to `DeltaBadge`.
- [ ] Task: Final Verification & Polishing.
- [ ] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)
