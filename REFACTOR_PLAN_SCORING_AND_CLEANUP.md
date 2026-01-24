# Refactoring Plan: Scoring Logic Cleanup & Codebase Hygiene

## Overview
This document outlines the architectural cleanup required following the implementation of the "Director's Score" and "CoinFlip Archetype". The goal is to reduce technical debt by removing dead code and modularizing the simulation analysis engine.

---

## 1. Dead Code Removal (`resolution.rs`) ✅ COMPLETE

**Target:** `simulation-wasm/src/resolution.rs`
**Current State:** 1,471 LOC. Marked as DEPRECATED.
**Impact:**
- Confuses developers (two resolution systems).
- Bloats compilation time.
- Increases WASM binary size (if dead code elimination fails).

### Execution Plan
1. **Unlink:** Remove `pub mod resolution;` from `simulation-wasm/src/lib.rs`.
2. **Delete:** Remove `simulation-wasm/src/resolution.rs`.
3. **Verify:** Run `cargo check` to ensure no active modules depend on it.

**Status:** ✅ **COMPLETE** - File has been removed. No `resolution.rs` files exist in the codebase.

### Bead Task
```bash
bd create --title="Remove deprecated resolution.rs" --type=task --priority=3 --description="Remove the 1500 LOC deprecated file 'resolution.rs' and unlink it from lib.rs. Verify with cargo check."
```

---

## 2. Modularization (`decile_analysis.rs`) ✅ COMPLETE

**Target:** `simulation-wasm/src/decile_analysis.rs`
**Current State:** ~1,050 LOC "God Module".
**Responsibilities:**
- Type Definitions (`EncounterArchetype`, `Vitals`)
- Game Design Logic (Scoring, Pacing, Archetypes)
- Statistical Math (Deciles, Medians)
- UI Data Mapping (Visualization structs)

### New Architecture
Refactored into a clean `analysis` module structure:

```text
simulation-wasm/src/analysis/
├── mod.rs             # Re-exports for clean public API (132 LOC)
├── types.rs           # Structs & Enums (Vitals, Archetypes, Tiers) (342 LOC)
├── narrative.rs       # The "Director" logic (Scoring, Pacing, Labeling)
├── statistics.rs      # The "Mathematician" logic (Percentiles, Aggregation)
└── visualization.rs   # The "Presenter" logic (Frontend mapping)
```

**Total:** ~1,796 LOC (well-distributed, modular)

**Backward Compatibility:** `lib.rs` re-exports `decile_analysis` alias for existing code.

### Execution Plan
1. **Create Directory:** `simulation-wasm/src/analysis/`.
2. **Migrate Types:** Move Enums and Structs to `types.rs`.
3. **Migrate Logic:**
    - Move `assess_archetype`, `calculate_day_pacing` to `narrative.rs`.
    - Move `run_decile_analysis` to `statistics.rs`.
    - Move `extract_combatant_visualization` to `visualization.rs`.
4. **Update Imports:** Fix `lib.rs`, `api/wasm.rs`, and `auto_balancer.rs`.

**Status:** ✅ **COMPLETE** - Refactored into modular structure. All imports updated.

### Bead Task
```bash
bd create --title="Refactor decile_analysis.rs into modular analysis package" --type=task --priority=3 --description="Split the 1000+ LOC decile_analysis.rs into analysis/types.rs, narrative.rs, statistics.rs, and visualization.rs to separate concerns."
```

---

## 3. Frontend Schema Hygiene (Optional) ⏸️ DEFERRED

**Target:** `src/model/model.ts`
**Current State:** 704 LOC.
**Analysis:** While large, it serves as a good "Single Source of Truth" for Zod schemas. Splitting it now might introduce circular dependency risks for little gain.

**Decision:** **DEFER** until >1,000 LOC.

**Status:** ⏸️ **DEFERRED** - Currently 704 LOC, well under the 1,000 LOC threshold. No action needed at this time.

---

## Summary

| Task | Status | Notes |
|------|--------|-------|
| Remove deprecated resolution.rs | ✅ COMPLETE | File removed, no dependencies found |
| Refactor decile_analysis.rs | ✅ COMPLETE | Modularized into analysis/ package |
| Frontend Schema Hygiene | ⏸️ DEFERRED | 704 LOC, under 1,000 threshold |

**Next Steps:**
- Documentation updates to reflect new modular structure
- Continue monitoring `src/model/model.ts` size as the codebase grows
