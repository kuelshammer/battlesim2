# Plan: Encounter Auto-Balancer Engine

## Phase 1: Role Detection & Adjustment Framework
Goal: Implement the logic to identify monster roles and create numeric adjustment knobs.

- [~] Task: Implement Role Detection
    - Add `MonsterRole` enum and detection logic in `simulation-wasm/src/model.rs`.
- [ ] Task: Create Adjustment Knobs
    - Implement functions to apply percentage-based modifications to damage, HP, and DC in `simulation-wasm/src/creature_adjustment.rs`.

## Phase 2: The Optimization Loop
Goal: Implement the iterative Hill Climbing algorithm in Rust.

- [ ] Task: Implement the "Safety Clamp"
    - Loop that nerfs monster damage/DC until the `Safety Grade` is at least B.
- [ ] Task: Implement the "Intensity Pump"
    - Loop that buffs monster HP/Resiliance until the `Intensity Tier` is at least 3.
- [ ] Task: Implement the "Slog Filter"
    - Logic to cap HP buffs if median rounds exceed 8.

## Phase 3: Dice Reconstruction
Goal: Convert adjusted numeric targets back into valid 5e dice notation.

- [ ] Task: HP Reconstruction Logic
    - Back-calculate Hit Dice count based on target HP, fixed die size, and CON mod.
- [ ] Task: Damage Reconstruction Logic
    - Back-calculate damage dice (e.g., 15 -> 2d10+4) and handle Multiattack adjustments.

## Phase 4: WASM Bindings & Frontend Models
Goal: Expose the Auto-Balancer to the web application.

- [ ] Task: Create WASM Entry Point
    - Export `auto_adjust_encounter_wasm` in `lib.rs`.
- [ ] Task: Update TypeScript Models
    - Define `AdjustmentReport` and updated `Creature` schemas in `src/model/model.ts`.

## Phase 5: UI Implementation
Goal: Add the "Magic Wand" button and results display.

- [ ] Task: Add Auto-Adjust button to Encounter cards
- [ ] Task: Implement "Adjustment Preview" Modal
    - Show side-by-side diff of original vs. balanced monsters.
- [ ] Task: Final Verification & Polishing
