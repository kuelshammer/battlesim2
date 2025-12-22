# Plan: 5e.tools Monster Import

## Phase 1: Core Data Parsing [checkpoint: 0d1d260]
Goal: Implement the logic to parse 5e.tools JSON and map basic stats (HP, AC, Abilities).

- [x] Task: Define Zod schemas for 5e.tools monster format (9a88260)
    - Write tests for schema validation
    - Implement schemas in `src/model/import/5etools-schema.ts`
- [x] Task: Implement mapping logic for core stats (865215f)
    - Write tests for mapping HP, AC, and Abilities
    - Implement mapper in `src/model/import/5etools-mapper.ts`
- [x] Task: Conductor - User Manual Verification 'Phase 1: Core Data Parsing' (0d1d260)

## Phase 2: Action Conversion [checkpoint: 528efba]
Goal: Convert 5e.tools actions and multiattack into BattleSim actions.

- [x] Task: Implement regex-based parsing for standard attacks (8b865cc)
    - Write tests for extracting hit bonus and damage from descriptions
    - Implement parsing in `src/model/import/5etools-action-parser.ts`
- [x] Task: Implement Multiattack mapping (0bd955a)
    - Write tests for multiattack routinely logic
    - Integrate with mapper
- [x] Task: Conductor - User Manual Verification 'Phase 2: Action Conversion' (528efba)

## Phase 3: UI Integration
Goal: Provide a way for users to use the import feature in the browser.

- [x] Task: Create Import Modal component (c2da424)
    - Write tests for modal interaction and input handling
    - Implement `src/components/creatureForm/ImportModal.tsx`
- [x] Task: Integrate Import button into Creature Form (c2da424)
    - Write tests for form population from imported data
    - Update `src/components/creatureForm/creatureForm.tsx`
- [x] Task: Conductor - User Manual Verification 'Phase 3: UI Integration' (c2da424)
