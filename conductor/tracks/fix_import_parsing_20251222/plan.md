# Plan: Robust 5e.tools JSON Parsing

## Phase 1: Logic & Tests
Goal: Create a robust cleaning utility for JSON strings.

- [x] Task: Implement `cleanJsonInput` utility (a643a61)
    - Write tests for various "dirty" inputs (leading characters, markdown, etc.)
    - Implement utility in `src/model/import/utils.ts`
- [x] Task: Update `ImportModal` to use the cleaning utility (a643a61)
    - Update `src/components/creatureForm/ImportModal.tsx`
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Logic & Tests' (Protocol in workflow.md)
