# Plan: Project Cleanup and Performance Optimization

## Phase 1: Electron Removal [checkpoint: c2b57a5]
Goal: Strip out all abandoned Electron configurations and scripts.

- [x] Task: Clean `package.json` (08ab9d3)
    - Remove `"main"`, `"build"`, and Electron-related dependencies if any.
- [x] Task: Delete build scripts (e47cc2b)
    - Remove `build-electron.sh`.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Electron Removal' (c2b57a5)

## Phase 2: Monster Data Externalization [checkpoint: 08ab9d3]
Goal: Move monster data to a public JSON file to improve performance.

- [x] Task: Convert `monsters.ts` to `monsters.json` (83194)
    - Extract the array and save to `public/data/monsters.json`.
- [x] Task: Refactor Monster Search to fetch data (83223)
    - Update `src/components/creatureForm/monsterForm.tsx` to use `fetch` and `useEffect`.
- [x] Task: Clean up source data (83410)
    - Reduced `src/data/monsters.ts` to only include `DefaultMonsters`.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Monster Data Externalization' (83410)

## Phase 3: Zod Stabilization
Goal: Downgrade Zod to a stable version and restore type safety.

- [x] Task: Downgrade Zod (84396)
    - Run `npm install zod@^3.23`.
- [x] Task: Restore Import Schema (84469)
    - Re-implement Zod validation in `src/model/import/5etools-schema.ts`.
- [x] Task: Verify type safety (0cfb06c)
    - Run `npm run type-check`.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Zod Stabilization' (Protocol in workflow.md)