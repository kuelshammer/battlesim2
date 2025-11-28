# Implementation Plan - Battlesim Monte Carlo Upgrade

This plan outlines the steps to replace the deterministic TypeScript simulation backend with a Rust/WASM-based Monte Carlo simulation while preserving the existing User Interface.

## User Review Required

> [!IMPORTANT]
> **UI Behavior Change**: The "Luck" slider currently adjusts the simulation continuously (e.g., changing a d20 roll from 10.5 to 11.0). In the new system, the slider will select one of 5 "representative" outcomes from thousands of simulations.
> - **0-20% (Bad Luck)**: Shows a simulation where players rolled poorly/monsters rolled well.
> - **40-60% (Average)**: Shows the median outcome.
> - **80-100% (Good Luck)**: Shows a simulation where players dominated.
>
> The UI components themselves will remain identical, but the data displayed will be a "real" possible fight log rather than a mathematical average.

## Proposed Changes

### 1. Rust/WASM Setup
- Initialize a new Rust library in `simulation-wasm/`.
- Configure `Cargo.toml` with `wasm-bindgen`, `serde`, `serde_json`, `rand`, and `getrandom` (for WASM compatibility).
- Set up build scripts to compile Rust to WASM.

### 2. Data Modeling (Rust)
- Mirror the TypeScript interfaces from `src/model/model.ts` and `src/model/enums.ts` into Rust structs.
- Use `serde` to ensure strict JSON compatibility so the frontend data can be passed directly to Rust.
- **Key Structs**: `Creature`, `Action`, `Combattant`, `Encounter`, `SimulationResult`.

### 3. Simulation Logic Port
- Port `src/model/dice.ts`:
    - Implement a true RNG dice roller to replace the weighted average logic.
    - Support dice expressions (e.g., `2d8 + 4`).
- Port `src/model/simulation.ts`:
    - Implement the battle loop: Initiative (if applicable, or sticking to current turn order), Actions, Targeting, Buffs/Debuffs.
    - **Logic Adaptation**: Ensure `check_action_condition`, `get_next_target`, etc., work with concrete RNG values instead of averages.

### 4. Monte Carlo Engine
- Implement `run_monte_carlo(iterations: usize)`:
    - Run the full simulation loop `iterations` times (e.g., 1000).
    - Store the full result (logs, stats) of each run.
- Implement Scoring & Sorting:
    - Score formula: `10 * sum(player_hp) - sum(monster_hp)`.
    - Sort all runs by score.
- Implement Quintile Selection:
    - Based on the requested "Luck" input (0.0 - 1.0), select the specific run from the sorted list.
    - Example: Luck 0.5 -> Select run at index 500 (Median).

### 5. Frontend Integration
- Update `package.json` to include the WASM package (or load it dynamically).
- Modify `src/components/simulation/simulation.tsx`:
    - Replace the `runSimulation` import from TS with the WASM import.
    - Manage the async nature of WASM loading (if necessary).

## Verification Plan

### Automated Tests (Rust)
- **Unit Tests**: Verify dice rolling distribution and modifier logic.
- **Logic Tests**: Verify action conditions (e.g., "Heal only if ally < 50% HP") trigger correctly.

### Manual Verification
- **Regression Testing**: Compare a simple "1 Goblin vs 1 Fighter" scenario in the old vs. new system to ensure basic mechanics (AC, HP, Damage) are consistent.
- **Monte Carlo Check**: Run a simulation and verify that "Bad Luck" results actually show missed attacks/failed saves, and "Good Luck" results show crits/successful saves.
- **Aggregation Verification**: Verify that aggregated action labels correctly display target names (e.g., "Attack on Goblin 1") instead of just "Attack on".

## Concentration Implementation Plan

### 1. Data Model Updates
- [x] **`Buff` Schema**: Add `concentration: boolean` (default `false`).
- [x] **`Creature` Schema**: Add `conSaveBonus: number` (optional, fallback to generic `saveBonus`).
- [x] **`CreatureState` Schema**: Add `concentratingOn: string | null` (stores the ID of the active concentration effect).

### 2. Simulation Logic Updates (`simulation.rs`)
- **Helper Function: `break_concentration(caster_id: &str)`**:
    - Identify the effect the caster is concentrating on.
    - Iterate through all combatants (allies & enemies).
    - Remove any buff where `source == caster_id` AND `buff_id == concentrating_on`.
    - Clear `concentratingOn` state on the caster.
    - Log the event.

- **Casting Logic (`Action::Buff` / `Action::Debuff`)**:
    - Check if the new spell requires concentration.
    - If yes:
        - Check if caster is already concentrating.
        - If so, call `break_concentration` (old spell ends).
        - Set `concentratingOn` to the new spell's ID.

- **Damage Logic (`Action::Atk`)**:
    - When a creature takes damage:
        - Check if they are concentrating (`concentratingOn.is_some()`).
        - Calculate DC: `max(10, damage / 2)`.
        - Roll Save: `d20 + con_save_bonus`.
        - If fail: Call `break_concentration`.

- **Incapacitation Logic**:
    - If a creature drops to 0 HP, call `break_concentration`.

### 3. Frontend Updates
- **Creature Editor**: Add input for `Constitution Save Bonus`.
- **Action Editor**: Add checkbox for `Concentration` in Buff/Debuff effects.
- **UI Display**: Show a "Concentrating" indicator on the combatant card.
