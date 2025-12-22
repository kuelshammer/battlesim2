# Plan: Project Cleanup and Performance Optimization

## Phase 1: Electron Removal
Goal: Strip out all abandoned Electron configurations and scripts.

- [x] Task: Clean `package.json` (08ab9d3)
    - Remove `"main"`, `"build"`, and Electron-related dependencies if any.
- [x] Task: Delete build scripts (e47cc2b)
    - Remove `build-electron.sh`.
- [~] Task: Conductor - User Manual Verification 'Phase 1: Electron Removal' (Protocol in workflow.md)

## Phase 2: Monster Data Externalization
Goal: Move monster data to a public JSON file to improve performance.

- [ ] Task: Convert `monsters.ts` to `monsters.json`
    - Extract the array and save to `public/data/monsters.json`.
- [ ] Task: Refactor Monster Search to fetch data
    - Update `src/components/creatureForm/monsterForm.tsx` to use `fetch` and `useEffect`.
- [ ] Task: Clean up source data
    - Delete `src/data/monsters.ts`.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Monster Data Externalization' (Protocol in workflow.md)

## Phase 3: Zod Stabilization
Goal: Downgrade Zod to a stable version and restore type safety.

- [ ] Task: Downgrade Zod
    - Run `npm install zod@^3.23`.
- [ ] Task: Restore Import Schema
    - Re-implement Zod validation in `src/model/import/5etools-schema.ts`.
- [ ] Task: Verify type safety
    - Run `npm run type-check`.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Zod Stabilization' (Protocol in workflow.md)
