# Plan: Refactor 5e.tools Import to Custom Creature Flow

## Phase 1: Preparation
Goal: Prepare the Custom Form to receive imported data.

- [x] Task: Update `Creature` model for optional fields (c8b8445)
    - Ensure fields like `hp` and `ac` can handle partial initialization if needed.
- [x] Task: Create `ImportButton` component (2ceba6b)
    - Extract the import button logic to be reusable or move it to `CustomForm`.

## Phase 2: Integration
Goal: Connect the Import Modal to the Custom Form.

- [x] Task: Add Import trigger to `CustomForm.tsx` (123121d)
    - Add a "Pre-fill from 5e.tools JSON" button at the top of the custom creature form.
- [x] Task: Implement pre-fill logic (123121d)
    - When JSON is imported, map it to the form's state using `mapMonster5eToCreature`.
- [x] Task: Remove Import from `MonsterForm.tsx` (123121d)
    - Clean up the old entry point to prevent confusion.

## Phase 3: Verification
Goal: Ensure the flow is intuitive and functional.

- [x] Task: Verify pre-fill functionality (a164c54)
    - Paste a partial JSON (like a race) and confirm the form populates available fields.
- [x] Task: Conductor - User Manual Verification (a164c54)
