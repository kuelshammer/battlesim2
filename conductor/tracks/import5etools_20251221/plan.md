# Plan: 5e.tools Monster Import

## Phase 1: Core Data Parsing
Goal: Implement the logic to parse 5e.tools JSON and map basic stats (HP, AC, Abilities).

- [x] Task: Define Zod schemas for 5e.tools monster format (9a88260)
    - Write tests for schema validation
    - Implement schemas in `src/model/import/5etools-schema.ts`
- [x] Task: Implement mapping logic for core stats (865215f)
    - Write tests for mapping HP, AC, and Abilities
    - Implement mapper in `src/model/import/5etools-mapper.ts`
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Core Data Parsing' (Protocol in workflow.md)

## Phase 2: Action Conversion
Goal: Convert 5e.tools actions and multiattack into BattleSim actions.

- [ ] Task: Implement regex-based parsing for standard attacks
    - Write tests for extracting hit bonus and damage from descriptions
    - Implement parsing in `src/model/import/5etools-action-parser.ts`
- [ ] Task: Implement Multiattack mapping
    - Write tests for multiattack routinely logic
    - Integrate with mapper
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Action Conversion' (Protocol in workflow.md)

## Phase 3: UI Integration
Goal: Provide a way for users to use the import feature in the browser.

- [ ] Task: Create Import Modal component
    - Write tests for modal interaction and input handling
    - Implement `src/components/creatureForm/ImportModal.tsx`
- [ ] Task: Integrate Import button into Creature Form
    - Write tests for form population from imported data
    - Update `src/components/creatureForm/creatureForm.tsx`
- [ ] Task: Conductor - User Manual Verification 'Phase 3: UI Integration' (Protocol in workflow.md)
