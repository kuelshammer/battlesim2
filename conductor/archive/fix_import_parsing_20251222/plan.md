# Plan: Robust 5e.tools JSON Parsing

## Phase 1: Logic & Tests
Goal: Create a robust cleaning utility for JSON strings.

- [x] Task: Implement `cleanJsonInput` utility (a643a61)
    - Write tests for various "dirty" inputs (leading characters, markdown, etc.)
    - Implement utility in `src/model/import/utils.ts`
- [x] Task: Update `ImportModal` to use the cleaning utility (a643a61)
    - Update `src/components/creatureForm/ImportModal.tsx`
- [x] Task: Improve schema and mapper for Abjurer-style fields (source, nested type) (f185ec5)
    - Update `src/model/import/5etools-schema.ts`
    - Update `src/model/import/5etools-mapper.ts`
- [x] Task: Conductor - User Manual Verification 'Phase 1: Logic & Tests' (f185ec5)
