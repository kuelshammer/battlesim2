# Plan: Human-Readable Combat Logs

## Phase 1: Engine Data Structure Updates (Rust)
Goal: Ensure the Rust simulation engine emits all necessary data (individual die rolls, modifiers, advantage context) in the JSON events.

- [x] Task: Create reproduction test case for missing roll data (Red) c56fb85
- [x] Task: Update `AttackEvent` and `DamageEvent` structs to include detailed roll info (Green) c56fb85
- [x] Task: Refactor existing logging logic to populate new fields (Refactor) c56fb85
- [x] Task: Conductor - User Manual Verification 'Engine Data Structure Updates (Rust)' (Protocol in workflow.md)

## Phase 2: Log Formatting Logic (TypeScript/JS)
Goal: Create a shared formatting library/module that transforms the enriched JSON events into strings.

- [x] Task: Create test suite for `LogFormatter` (Red)
- [x] Task: Implement `LogFormatter.toSummary()` (Green)
- [x] Task: Implement `LogFormatter.toDetails()` (Green)
- [x] Task: Handle unit naming and fallback logic (Green)
- [x] Task: Conductor - User Manual Verification 'Log Formatting Logic (TypeScript/JS)' (Protocol in workflow.md)

## Phase 3: Frontend UI Implementation
Goal: Update the Web UI to display the new "Compact Card" and "Detailed Modal" using the formatter.

- [x] Task: Create `CombatLogCard` component tests (Red)
- [x] Task: Implement `CombatLogCard` (Compact view) (Green)
- [x] Task: Create `CombatLogModal` component (Detailed view) (Green)
- [x] Task: Integrate components into the main simulation view (Green)
- [x] Task: Conductor - User Manual Verification 'Frontend UI Implementation' (Protocol in workflow.md)

## Phase 4: File Logging Integration
Goal: Update the file logger to include the formatted human-readable strings.

- [x] Task: Create tests for file logging output (Red) 18f9bc3
- [x] Task: Implement FileLogger using LogFormatter (Green) a4bbf4d
- [ ] Task: Conductor - User Manual Verification 'File Logging Integration' (Protocol in workflow.md)
