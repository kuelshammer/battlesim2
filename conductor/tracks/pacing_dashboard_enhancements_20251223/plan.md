# Plan: Daily Pacing Dashboard Enhancements

## Phase 1: Data Logic Refactor [checkpoint: ec0e894]
Goal: Refactor pacing data calculation to support the "Grand Total" budget and accurate segment mapping.

- [x] Task: Update `pacingData` in `Simulation.tsx` to calculate `GrandTotalBudget` (Initial + Recovery) and generate structured segment lists. (80bfc3c)
- [x] Task: Unit Test - Verify `pacingData` normalization correctly handles recoveries and multiple encounters. (80bfc3c)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (ec0e894)

## Phase 2: FuelGauge Component Updates
Goal: Implement the new visual requirements in the `FuelGauge` component.

- [ ] Task: Update `FuelGauge.tsx` props and rendering logic to support structured segments, rest dividers, and labels.
- [ ] Task: Update `fuelGauge.module.scss` with styles for alternating colors, labels, and green rest dividers.
- [ ] Task: Visual Verification - Ensure labels and alternating colors are clear even with high encounter counts.
- [ ] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)

## Phase 3: Dashboard Integration & Polish
Goal: Finalize integration and ensure responsiveness for various adventuring day sizes.

- [ ] Task: Verify the "Descent Graph" and "Assistant Summary" remain consistent with the new budget scaling.
- [ ] Task: Performance Check - Ensure the dashboard remains snappy with 10+ timeline items.
- [ ] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)
