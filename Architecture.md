# Architecture: Battlesim2 Simulation Engine & UI

## Overview

Battlesim2 is a high-fidelity D&D 5e combat simulator built with a Rust/WASM backend and a React/Next.js frontend. The architecture focuses on deterministic, event-driven simulation capable of performing large-scale batch runs for statistical analysis (1% bucketing).

## Core Architecture

### 1. Event-Driven Backend (Rust/WASM)
The simulation engine uses an event-driven model to ensure a "Single Source of Truth" for all state changes.

- **TurnContext**: Central repository for encounter state, including combatant vitality, active effects (buffs/debuffs), and resource ledgers.
- **ActionResolver**: Pure logic component that translates high-level actions (Attacks, Spells) into a sequence of atomic events.
- **Unified HP Management**: All damage and healing must pass through `TurnContext::apply_damage` and `TurnContext::apply_healing`, ensuring that `DamageTaken`, `HealingApplied`, and `UnitDied` events are consistently emitted.
- **ResourceLedger**: Tracks discrete (Spell Slots, Ki) and variable resources with standardized reset rules (Short Rest, Long Rest).

### 2. Two-Pass Simulation System
To handle the high memory cost of detailed logs in batch simulations (e.g., 10,100 runs):
- **Phase 1 (Survey)**: Executes lightweight simulations tracking only final outcomes and scores.
- **Phase 2 (Seed Selection)**: Uses 1% bucketing (100 buckets, each representing 101 runs) to identify median seeds for each percentile.
- **Phase 3 (Re-simulation)**: Re-runs specific "interesting" seeds with full event collection for detailed UI visualization.

### 3. Frontend Architecture (React/TypeScript)
The UI is designed around "Intentional Minimalism" and "Arcane Aesthetics."

- **State Management**: Uses `useStoredState` for persistent configuration and `useSimulationWorker` to offload heavy computations to background threads.
- **Visual Language**: 
    - **Arcane Vitals Vellum**: A high-fidelity, glassmorphic resource panel for combatants featuring staggered animations and bespoke typography (`Cinzel Decorative`, `Crimson Pro`).
    - **Interactive Grimoire**: Detailed combat logs and 1% granularity dashboards providing staggered, readable breakdowns of complex simulations.
- **Accessibility**: Comprehensive ARIA support and keyboard navigation (Escape to close modals, tab-indexed controls).

## Implementation Status

### Completed
- [x] **Phase 1: Core Integrity**: Unified HP modification and event emission.
- [x] **Phase 2: GUI Editors**: Full-featured editors for players, monsters, and adventuring days.
- [x] **Phase 3: EventLog & Two-Pass**: Detailed logging with memory-efficient re-simulation.
- [x] **Phase 4: Arcane UI Refactor**: Implementation of the "Arcane Vitals Vellum" and custom font integration.
- [x] **Phase 5: D&D Rule Fidelity**: Implementation of actual dice rolling (replacing averages) and critical hit logic.

### In Progress / Next
- [~] **Phase 6: Advanced AI**: Integration of smart targeting (lowest HP, concentration breaking) and resource-aware decision making.
- [ ] **Phase 7: Strategy Builder**: UI for user-defined "Gambits" and priority-based action selection.

## Technology Stack
- **Backend**: Rust, WASM (`wasm-bindgen`), `serde`, `proptest`.
- **Frontend**: Next.js, React, TypeScript, SCSS Modules, `framer-motion`.
- **Typography**: Google Fonts (`Cinzel Decorative`, `Crimson Pro`).
