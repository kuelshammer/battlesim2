# Plan: Redundant Short Rest Cleanup

## Phase 1: Frontend Cleanup [checkpoint: 7649283]
Goal: Remove the redundant field and checkbox from the web app.

- [x] Task: Remove shortRest checkbox from `EncounterForm.tsx` (7649283)
- [x] Task: Remove shortRest field from `EncounterSchema` in `src/model/model.ts` (7649283)

## Phase 2: Backend Cleanup [checkpoint: 7649283]
Goal: Remove the redundant field from the Rust backend.

- [x] Task: Remove short_rest field from `Encounter` struct in `simulation-wasm/src/model.rs` (7649283)
- [x] Task: Verify successful build and tests (7649283)
