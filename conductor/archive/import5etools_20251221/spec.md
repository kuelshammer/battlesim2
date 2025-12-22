# Spec: 5e.tools Monster Import

## Overview
This feature allows users to import monster data directly from 5e.tools JSON files or raw JSON text. This streamlines encounter creation by leveraging the extensive library of creatures provided by the 5e.tools project.

## Technical Goals
- Parse the 5e.tools monster schema.
- Map 5e.tools properties to the internal BattleSim `Creature` and `CreatureState` models.
- Convert 5e.tools actions into BattleSim `Action` objects (attacks, spells, and multiattack).
- Provide a user-friendly UI for importing data (file upload or copy-paste).

## Mapping Requirements

### Core Stats
- **Name:** `name` -> `name`
- **AC:** `ac` (can be complex, need to extract primary value) -> `ac`
- **HP:** `hp.average` -> `maxHp` and `currentHp`
- **Abilities:** `str`, `dex`, `con`, `int`, `wis`, `cha` -> ability scores
- **Saves:** `save` object -> save bonuses

### Actions
- **Multiattack:** Parse `multiattack` descriptions or arrays to determine the creature's attack routine.
- **Attacks:** Parse action descriptions to extract:
    - Accuracy bonus (e.g., "+5 to hit")
    - Damage dice (e.g., "1d8 + 3")
    - Damage type
    - Targets (Ally/Enemy)
- **Spells:** (Initial implementation may focus on standard attacks; spell parsing is a stretch goal).

## UI/UX
- Add an "Import from 5e.tools" button to the Creature Form.
- Open a modal allowing the user to paste JSON or upload a file.
- Show a preview of the mapped creature before finalizing the import.

## Acceptance Criteria
- User can paste a 5e.tools monster JSON and see it populated in the form.
- Core stats (AC, HP, Abilities) are mapped correctly.
- Basic attacks are automatically converted into functional BattleSim actions.
- The simulator can run with the imported creature.
