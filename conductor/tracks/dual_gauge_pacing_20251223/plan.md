# Plan: Dual-Gauge Pacing System (Vitality & Power)

## Phase 1: WASM & Logic Updates [checkpoint: bc071a5]
Goal: Implement the math for split metrics in the Rust backend and export dual timelines.

- [x] Task: Update `DecileStats` and `AggregateOutput` structs in `decile_analysis.rs` to include `vitality_timeline` and `power_timeline`. (bc071a5)
- [x] Task: Implement metric calculation logic in `intensity_calculation.rs`: (bc071a5)
    - Vitality: `(HP + HD_Value) / Total`.
    - Power: `(Slot_Value + Feature_Points) / Total`.
- [x] Task: Update `calculate_run_stats` in `decile_analysis.rs` to populate both timelines per simulation step. (bc071a5)
- [x] Task: Unit Test (Rust) - Verify Vitality and Power calculations match the 5e statistical averages specified. (bc071a5)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (bc071a5)

## Phase 2: Frontend Data & Infrastructure [checkpoint: 4a952b9]
Goal: Update the TypeScript bridge and processing utilities to support the dual-metric data.

- [x] Task: Update TypeScript interfaces in `model.ts` and `usesimulationworker.ts` to include the new dual timeline fields. (4a952b9)
- [x] Task: Refactor `pacingUtils.ts` to generate `vitalitySegments` and `powerSegments` for the UI. (4a952b9)
- [x] Task: Unit Test (Vitest) - Ensure `calculatePacingData` correctly parses the dual timelines from WASM. (4a952b9)
- [x] Task: Conductor - User Manual Verification 'Phase 2' (4a952b9)

## Phase 3: Visual Implementation [checkpoint: c7ecabd]
Goal: Update the dashboard and cards to render the dual-gauge system.

- [x] Task: Update `DescentGraph.tsx` to render two lines (Red/Blue) with corresponding uncertainty areas. (c7ecabd)
- [x] Task: Update `FuelGauge.tsx` to display parallel bars for Vitality and Power in both Plan and Reality rows. (c7ecabd)
- [x] Task: Update `AssistantSummary.tsx` and `EncounterResult.tsx` to display the dynamic "State Labels" (e.g., Glass Cannon). (c7ecabd)
- [x] Task: Visual Verification - Ensure the dual-line graph is readable even with many encounters. (c7ecabd)
- [x] Task: Conductor - User Manual Verification 'Phase 3' (c7ecabd)

## Phase 4: Smart Auto-Balancer [checkpoint: final]
Goal: Leverage the split metrics to make the auto-adjuster more tactically aware.

- [x] Task: Refactor `auto_balancer.rs` heuristic to use the Vitality/Power ratio for choosing between HP vs Damage adjustments. (final)
- [x] Task: Integration Test - Verify the balancer correctly chooses to nerf damage (instead of HP) when Vitality is dangerously low but Power is high. (final)
- [x] Task: Conductor - User Manual Verification 'Phase 4' (final)

---
**Track Completed.**
Calculated independent percentiles for vitality and power risk areas to ensure statistical accuracy.
