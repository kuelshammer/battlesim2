# Spec: Redundant Short Rest Cleanup

## Overview
Remove the deprecated `shortRest` checkbox from the Encounter UI and data models. The project has transitioned to a timeline-based system where Short Rests are discrete events between combat encounters.

## Requirements
1. **Remove UI Checkbox:** Remove the "The players get a short rest" checkbox from `EncounterForm.tsx`.
2. **Clean TypeScript Model:** Remove the `shortRest` field from `EncounterSchema` in `model.ts`.
3. **Clean Rust Model:** Remove the `short_rest` field from the `Encounter` struct in `model.rs`.
4. **Ensure Backend Compatibility:** Verify that the backend logic (which already uses standalone timeline events) doesn't rely on the per-encounter flag.
