# Plan: Liquid Asset Intensity Scoring

## Phase 1: EHP Point System & TDNW [checkpoint: 64209b3]
Goal: Implement the core asset weighting and the Total Daily Net Worth calculation.

- [x] Task: Define EHP constants and `calculate_ehp` helper in `simulation-wasm/src/resources.rs` or `decile_analysis.rs`. (64209b3)
- [x] Task: Implement TDNW calculation in `analyze_results`. (64209b3)
- [x] Task: Unit Test - Verify EHP values for a standard Fighter and Wizard. (64209b3)
- [x] Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md) (64209b3)

## Phase 2: Dynamic Threshold Logic [checkpoint: 7ee2483]
Goal: Implement the Delta calculation and the $1/N$ scaling rule.

- [x] Task: Update `calculate_run_stats` to return start/end EHP per encounter. (7ee2483)
- [x] Task: Implement `Target Drain` calculation based on number of combat events in timeline. (7ee2483)
- [x] Task: Update `assess_intensity_tier` to use the Delta vs. Target logic. (7ee2483)
- [x] Task: Unit Test - Verify Intensity Tier shift when changing $N$ for the same result data. (7ee2483)
- [x] Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md) (7ee2483)

## Phase 3: UI & Visual Refinement [checkpoint: 7ee2483]
Goal: Replace Stars with Bolts and update the Tier mapping.

- [x] Task: Update `AnalysisComponents.tsx` to render Lightning Bolts (`faBolt`). (7ee2483)
- [x] Task: Implement `(Tier - 1)` bolt count logic in frontend. (7ee2483)
- [x] Task: Update SCSS for yellow lightning styling. (7ee2483)
- [x] Task: Final Verification - Confirm Black Dragon vs. Fighters shows correct Bolts and Grade. (7ee2483)
- [x] Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md) (7ee2483)

## Phase 4: Target Roles & Weighted Adjustments [checkpoint: 7649283]
Goal: Implement narrative pacing via encounter categories.

- [x] Task: Define `TargetRole` enum and update `Encounter` struct in `model.rs`. (7649283)
- [x] Task: Implement weighted target logic in `decile_analysis.rs`. (7649283)
- [x] Task: Add "Target Role" dropdown to `EncounterForm.tsx`. (7649283)
- [x] Task: Update `AutoAdjuster` to use weighted targets from the full timeline context. (7649283)
- [x] Task: Integration Test - Multi-encounter day with mixed roles (Skirmish/Boss). (7649283)
- [x] Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md) (7649283)
