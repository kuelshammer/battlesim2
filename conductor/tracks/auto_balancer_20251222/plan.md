# Plan: Encounter Auto-Balancer Engine

## Phase 1: Role Detection & Adjustment Framework [checkpoint: c318530]
Goal: Implement the logic to identify monster roles and create numeric adjustment knobs.

- [x] Task: Implement Role Detection (c318530)
    - Add `MonsterRole` enum and detection logic in `simulation-wasm/src/model.rs`.
- [x] Task: Create Adjustment Knobs (c318530)
    - Implement functions to apply percentage-based modifications to damage, HP, and DC in `simulation-wasm/src/creature_adjustment.rs`.

## Phase 2: The Optimization Loop [checkpoint: 43e5a31]
Goal: Implement the iterative Hill Climbing algorithm in Rust.

- [x] Task: Implement the "Safety Clamp" (43e5a31)
    - Loop that nerfs monster damage/DC until the `Safety Grade` is at least B.
- [x] Task: Implement the "Intensity Pump" (43e5a31)
    - Loop that buffs monster HP/Resiliance until the `Intensity Tier` is at least 3.
- [x] Task: Implement the "Slog Filter" (43e5a31)
    - Logic to cap HP buffs if median rounds exceed 8.

## Phase 3: Dice Reconstruction [checkpoint: 4af3d7c]
Goal: Convert adjusted numeric targets back into valid 5e dice notation.

- [x] Task: HP Reconstruction Logic (4af3d7c)
    - Back-calculate Hit Dice count based on target HP, fixed die size, and CON mod.
- [x] Task: Damage Reconstruction Logic (4af3d7c)
    - Back-calculate damage dice (e.g., 15 -> 2d10+4) and handle Multiattack adjustments.

## Phase 4: WASM Bindings & Frontend Models [checkpoint: 2bec118]
Goal: Expose the Auto-Balancer to the web application.

- [x] Task: Create WASM Entry Point (2bec118)
    - Export `auto_adjust_encounter_wasm` in `lib.rs`.
- [x] Task: Update TypeScript Models (2bec118)
    - Define `AdjustmentReport` and updated `Creature` schemas in `src/model/model.ts`.

## Phase 5: UI Implementation
Goal: Add the "Magic Wand" button and results display.

- [x] Task: Add Auto-Adjust button to Encounter cards (6a40a37)
- [ ] Task: Implement "Adjustment Preview" Modal
    - Show side-by-side diff of original vs. balanced monsters.
- [ ] Task: Final Verification & Polishing
