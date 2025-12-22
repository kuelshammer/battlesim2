# Spec: Refactor 5e.tools Import to Custom Creature Flow

## Overview
Currently, the 5e.tools import lives in the "Monster Search" section, which expects a completed, valid monster statblock. This causes friction when users want to import partial data (like race definitions) or customize an imported monster before adding it.

This track moves the Import functionality to the **Custom Creature Form**, transforming it from a "Direct Add" into a "Pre-fill" feature.

## Goals
- Improve UX by allowing users to tweak imported data before the creature is added to the simulation.
- Support partial imports (e.g., lineages/races) by letting the user fill in the missing combat stats (HP/AC) via the UI.
- Simplify the Monster Search UI by focusing strictly on the externalized JSON database.

## Acceptance Criteria
- An "Import from 5e.tools" button exists within the Custom Creature creation flow.
- Pasting a JSON successfully populates the Custom Form fields.
- The user can edit any field (Name, HP, AC, Actions) after the import.
- The generic "Add Monster" search no longer includes the "Import" button.
