# Spec: Project Cleanup and Performance Optimization

## Overview
This track focuses on technical debt reduction and performance improvements. We will remove unused Electron configurations, optimize the handling of the massive monster dataset, and stabilize the schema validation library.

## Goals

### 1. Electron Cleanup (Priority 1)
- Remove `"main": "electron.js"` and the `"build"` block from `package.json`.
- Delete `build-electron.sh`.
- Ensure the project is strictly configured as a Next.js web application.

### 2. Monster Data Optimization (Priority 2)
- Move the 25,000+ line monster database from `src/data/monsters.ts` to a static JSON file in `public/data/monsters.json`.
- Update the monster search logic to fetch this JSON asynchronously.
- Goal: Improve IDE performance and decrease initial bundle size.

### 3. Zod Stabilization (Priority 3)
- Downgrade `zod` from the experimental `v4.1.13` to the stable `v3.x`.
- Restore formal schema validation in the 5e.tools import logic.
- Ensure type safety across the model layer.

## Acceptance Criteria
- `package.json` no longer contains Electron-specific fields or scripts.
- `src/data/monsters.ts` is deleted or significantly reduced.
- The "Add Monster" search works correctly, loading data from the public JSON.
- `zod` is at version `^3.23.x`.
- The 5e.tools import remains functional with the restored Zod schema.
