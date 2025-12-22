# Plan: Auto-Balancer Backend Improvements

## Phase 1: Fixing Action Support [checkpoint: b315c1e]
Goal: Ensure all monster abilities are adjustable.

- [x] Task: Implement Template Action Support (implemented in previous turn)
- [x] Task: Add Unit Tests for DC Template Adjustments (b315c1e)

## Phase 2: Tuning the Optimization Loop [checkpoint: 5a9c2d1]
Goal: Improve the quality and accuracy of the balancing results.

- [x] Task: Implement "Broken-to-Safe" escalation (5a9c2d1)
- [x] Task: Refine Role Detection Logic (5a9c2d1)
- [x] Task: Change DC Adjustment Granularity (5a9c2d1)

## Phase 3: Final Verification [checkpoint: 7ee2483]
Goal: Confirm the fix for the Black Dragon scenario.

- [x] Task: Create Integration Test for Black Dragon vs. Fighters (7ee2483)
