# Data Flow & State Transitions

**Purpose**: Request lifecycles and state transitions for debugging/integration.

---

## Table of Contents

1. [Simulation Request Flow](#simulation-request-flow)
2. [Auto-Balance Request Flow](#auto-balance-request-flow)
3. [State Management Architecture](#state-management-architecture)
4. [Event Flow](#event-flow)
5. [Replay System](#replay-system)
6. [State Transition Diagrams](#state-transition-diagrams)
7. [Type Coercion & Validation](#type-coercion--validation)
8. [Error Handling Flow](#error-handling-flow)
9. [Performance Optimization Points](#performance-optimization-points)

---

## Simulation Request Flow

### Overview Diagram

```
User Action → Frontend State → WebWorker → WASM Backend → Simulation → Results → UI Update
```

### Phase 1: Initiation

```
┌─────────────────────────────────────────────────────────────────────┐
│                        USER ACTION                                  │
│  Edit player/monster → useAutoSimulation detects change             │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ (500ms debounce)
┌─────────────────────────────────────────────────────────────────────┐
│                  useSimulationWorker.runSimulation()                │
│  - Generates unique genId                                           │
│  - Creates WorkerMessage:                                           │
│    { type: 'START_SIMULATION', players, timeline, maxK, genId }     │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ postMessage
┌─────────────────────────────────────────────────────────────────────┐
│              WebWorker (simulation.worker.controller.ts)            │
│  - Receives START_SIMULATION message                                │
│  - Initializes WASM module (if not already)                         │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ wasm-bindgen
┌─────────────────────────────────────────────────────────────────────┐
│                    WASM Backend (wasm_api.rs)                       │
│  - run_simulation_wasm(players, timeline, iterations)               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Phase 2: Processing

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Orchestration Layer                                │
│  run_simulation_with_three_tier()                                   │
│    ├── Phase 1: run_lightweight_survey()                           │
│    │         - 10,100 iterations, no events                         │
│    │         - Returns: Vec<SurveyResult>                           │
│    │                                                                │
│    ├── Phase 2: select_interesting_seeds_with_tiers()              │
│    │         - 1% bucket analysis                                   │
│    │         - Returns: ~170 seeds (3 tiers)                        │
│    │                                                                │
│    └── Phase 3: run_deep_dive_simulation()                         │
│              - Re-run selected seeds with events                    │
│              - Returns: HashMap<seed, SimulationResult>             │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                  Analysis Module                                    │
│  build_gui_output()                                                 │
│    ├── compute_decile_stats()                                       │
│    ├── compute_skyline_analysis()                                  │
│    ├── compute_vitals()                                            │
│    └── generate_full_analysis_output()                             │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Phase 3: Progress Updates

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Worker Response Streaming                        │
│  Worker posts incremental updates:                                  │
│                                                                     │
│  { type: 'SIMULATION_UPDATE',                                       │
│    genId, kFactor, results?, analysis? }                            │
│                                                                     │
│  Frontend state updates:                                            │
│  - progress: kFactor / maxK                                         │
│  - results: accumulated results                                    │
│  - analysis: accumulated analysis                                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ useSimulationWorker state update
┌─────────────────────────────────────────────────────────────────────┐
│                     UI Re-render                                    │
│  - BackendStatusPanel shows progress                               │
│  - OverallSummary updates with new results                         │
│  - Skyline visualizations render incremental data                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Phase 4: Completion

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Worker Final Response                              │
│  { type: 'SIMULATION_COMPLETE',                                     │
│    genId, results, analysis, events }                               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ useSimulationWorker state update
┌─────────────────────────────────────────────────────────────────────┐
│                     UI Final Render                                 │
│  - BackendStatusPanel shows "Complete"                             │
│  - OverallSummary shows final statistics                           │
│  - EventLog renders first run events                               │
│  - Skyline visualizations render full data                         │
│  - CombatReplayModal can be opened                                 │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Auto-Balance Request Flow

### Overview Diagram

```
User Click → useSimulationWorker.autoAdjustEncounter() → Worker → WASM → Binary Search → Results
```

### Phase 1: Initiation

```
┌─────────────────────────────────────────────────────────────────────┐
│                     USER ACTION                                     │
│  Click "Auto-Balance" button on encounter                           │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│          useSimulationWorker.autoAdjustEncounter()                  │
│  - Generates unique genId                                           │
│  - Creates WorkerMessage:                                           │
│    { type: 'AUTO_ADJUST_ENCOUNTER',                                │
│      players, monsters, timeline, encounterIndex, genId }           │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ postMessage
┌─────────────────────────────────────────────────────────────────────┐
│              WebWorker (simulation.worker.controller.ts)            │
│  - Receives AUTO_ADJUST_ENCOUNTER message                           │
│  - Calls WASM auto_adjust_encounter_wasm()                          │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ wasm-bindgen
┌─────────────────────────────────────────────────────────────────────┐
│                    WASM Backend (auto_balancer.rs)                  │
│  auto_adjust_encounter()                                            │
│    ├── Calculate current encounter tier                             │
│    ├── Calculate target tier based on resources                     │
│    └── Adjust monsters to match target tier                         │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Phase 2: Optimization Loop

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Binary Search Algorithm                            │
│  1. Simulate current encounter 10,100 times                        │
│  2. Calculate win rate and encounter tier                          │
│  3. If tier == target tier: done                                   │
│  4. If tier < target: increase monster count/CR                    │
│  5. If tier > target: decrease monster count/CR                    │
│  6. Repeat until convergence or max iterations                     │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Phase 3: Results

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Worker Final Response                              │
│  { type: 'AUTO_ADJUST_COMPLETE',                                   │
│    genId, result: { monsters, analysis } }                         │
│                                                                     │
│  Frontend:                                                          │
│  - Updates timeline item with new monsters                         │
│  - Triggers re-simulation with adjusted encounter                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## State Management Architecture

### Session State (`useSimulationSession`)

**Storage**: localStorage (persistent)

**State**:
```typescript
{
    players: Creature[]
    timeline: TimelineEvent[]
    hasChanges: boolean
}
```

**Transitions**:
```
Initial → Load from localStorage → Ready
    ↓
User edits → Update state → Save to localStorage → hasChanges = true
    ↓
Save manually → Persist to localStorage → hasChanges = false
```

**Key Methods**:
- `loadState()`: Load from localStorage
- `saveState()`: Save to localStorage
- `setPlayers()`: Update players
- `setTimeline()`: Update timeline
- `createCombat()`: Add encounter to timeline
- `createShortRest()`: Add rest to timeline
- `updateTimelineItem()`: Update timeline item
- `deleteTimelineItem()`: Delete timeline item
- `swapTimelineItems()`: Reorder timeline

---

### Worker State (`useSimulationWorker`)

**Storage**: Component state (ephemeral)

**State**:
```typescript
{
    isRunning: boolean
    progress: number
    kFactor: number
    maxK: number
    results: SimulationResult[] | null
    analysis: FullAnalysisOutput | null
    events: SimulationEvent[] | null
    error: string | null
    optimizedResult: AutoAdjustmentResult | null
    genId: number
    isCancelled: boolean
}
```

**Transitions**:
```
Idle → runSimulation() → Running
    ↓                           ↓
    ← Worker updates ←←←←←←←←←←←←
    ↓                           ↓
Complete ←←←←←←←←←←←←←←←←←←←←←←←←
    ↓
Idle (ready for next simulation)
```

**Cancellation**:
```
Running → cancel() → isCancelled = true → Worker aborts → Idle
```

**Error**:
```
Running → Error from worker → error = message → isRunning = false → Idle
```

---

## Event Flow

### Event Generation

```
┌─────────────────────────────────────────────────────────────────────┐
│              TurnContext (Backend)                                  │
│  - All state changes emit events                                   │
│  - Events queued in EventBus.pending_events                        │
│                                                                     │
│  Event emission examples:                                          │
│  - apply_damage() → DamageTaken event                              │
│  - apply_healing() → HealingApplied event                          │
│  - pay_costs() → ResourceConsumed event                            │
│  - add_buff() → BuffApplied event                                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              EventBus (Backend)                                    │
│  - Collects events in history (max 1000 when logging)              │
│  - Triggers reactions via check_reactions()                        │
│  - Returns events in SimulationResult                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ wasm-bindgen
┌─────────────────────────────────────────────────────────────────────┐
│              Frontend Event Handling                               │
│  - Events received in WorkerResponse                               │
│  - Stored in useSimulationWorker.events                            │
│  - Passed to EventLog component                                   │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              EventLog Component                                    │
│  - Renders events chronologically                                  │
│  - Filters by event type                                           │
│  - Highlights events on hover                                      │
│  - Click to show details                                           │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Event Propagation

```
User Action → useSimulationWorker.runSimulation()
    ↓
WebWorker → WASM Backend
    ↓
ActionExecutionEngine.execute_encounter()
    ↓
execute_action_with_reactions()
    ↓
ActionResolver.resolve_action()
    ↓
TurnContext.apply_damage() / apply_healing() / etc.
    ↓
EventBus.emit(DamageTaken) / emit(HealingApplied) / etc.
    ↓
EventBus.pending_events.push(event)
    ↓
ReactionManager.check_reactions(event)
    ↓
(If reaction triggered) execute_reaction()
    ↓
Continue action execution
    ↓
EventBus.history.push(event)
    ↓
Return SimulationResult { events }
    ↓
Worker posts SIMULATION_COMPLETE { events }
    ↓
Frontend state.events = events
    ↓
EventLog renders events
```

---

## Replay System

### Replay Creation

```
┌─────────────────────────────────────────────────────────────────────┐
│              Simulation Complete                                   │
│  - First run events captured in full detail                        │
│  - Stored in useSimulationWorker.events                            │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              CombatReplayModal Opens                               │
│  - Receives events prop                                            │
│  - Initializes useCombatPlayback hook                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │
```

### Replay Navigation

```
┌─────────────────────────────────────────────────────────────────────┐
│              useCombatPlayback State                               │
│  - isPlaying: boolean                                              │
│  - currentRound: number                                            │
│  - currentTurn: number                                            │
│  - speed: number (1x, 2x, 4x)                                      │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Playback Controls                                     │
│  - Play: Start auto-advance with setInterval                       │
│  - Pause: Clear interval                                          │
│  - Next: Advance to next event                                    │
│  - Prev: Go to previous event                                     │
│  - Seek: Jump to specific round/turn                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Event Filtering                                       │
│  - Filter events by type                                           │
│  - Filter events by combatant                                      │
│  - Highlight specific events                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## State Transition Diagrams

### Simulation State Machine

```
                    ┌─────────────┐
                    │    IDLE     │
                    └──────┬──────┘
                           │ runSimulation()
                           ▼
                    ┌─────────────┐
                    │  RUNNING    │◄─────────────────┐
                    └──────┬──────┘                  │
                           │                         │
                           │ Worker updates          │ cancel()
                           ▼                         │
                    ┌─────────────┐                  │
                    │ UPDATING   │                   │
                    └──────┬──────┘                  │
                           │                         │
                           │ Complete                │
                           ▼                         │
                    ┌─────────────┐                  │
                    │  COMPLETE  │                   │
                    └──────┬──────┘                  │
                           │                         │
                           │                         │
                           └─────────────────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    IDLE     │
                    └─────────────┘

Error Path:
    RUNNING → Error → IDLE (with error state)
```

---

### Auto-Balance State Machine

```
┌─────────────┐     autoAdjustEncounter()     ┌─────────────┐
│    IDLE     │───────────────────────────────▶│ BALANCING   │
└─────────────┘                               └──────┬──────┘
                                                   │
                                                   │ Binary search iterations
                                                   ▼
                                            ┌─────────────┐
                                            │ ADJUSTING   │
                                            └──────┬──────┘
                                                   │
                                                   │ Converged
                                                   ▼
                                            ┌─────────────┐
                                            │  COMPLETE   │
                                            └──────┬──────┘
                                                   │
                                                   │
                                                   ▼
                                            ┌─────────────┐
                                            │    IDLE     │
                                            └─────────────┘
```

---

## Type Coercion & Validation

### Frontend → Backend

```
┌─────────────────────────────────────────────────────────────────────┐
│              Frontend (TypeScript)                                  │
│  Creature: { id: string, name: string, hp: number, ... }           │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ postMessage (structured clone)
┌─────────────────────────────────────────────────────────────────────┐
│              WebWorker                                              │
│  - Receives JavaScript objects                                     │
│  - Passes to WASM via wasm-bindgen                                 │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ wasm-bindgen
┌─────────────────────────────────────────────────────────────────────┐
│              WASM Backend (Rust)                                   │
│  - serde-wasm-bindgen deserializes to Rust structs                 │
│  - Creature { id: String, name: String, hp: u32, ... }             │
└─────────────────────────────────────────────────────────────────────┘
```

### Validation Points

1. **Frontend** (`model/schemas.ts`):
   - Zod schemas validate TypeScript types at runtime
   - Prevents invalid data from being sent to worker

2. **WASM Boundary** (`api/wasm_api.rs`):
   - `serde` deserialization fails on type mismatch
   - Returns error to frontend

3. **Backend** (`validation.rs`):
   - Action requirements validated before execution
   - Resource availability checked

---

### Backend → Frontend

```
┌─────────────────────────────────────────────────────────────────────┐
│              WASM Backend (Rust)                                   │
│  SimulationResult { seed, score, deaths, events }                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ serde-wasm-bindgen
┌─────────────────────────────────────────────────────────────────────┐
│              WebWorker                                              │
│  - Receives WASM objects                                          │
│  - Converts to JavaScript objects                                  │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼ postMessage
┌─────────────────────────────────────────────────────────────────────┐
│              Frontend (TypeScript)                                 │
│  SimulationResult: { seed: number, score: number, ... }            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Error Handling Flow

### Simulation Errors

```
┌─────────────────────────────────────────────────────────────────────┐
│              Error Detection                                       │
│  - Backend panic (caught by WASM)                                  │
│  - Worker crash (caught by try/catch)                              │
│  - Invalid input (caught by validation)                            │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Worker Error Response                                 │
│  { type: 'SIMULATION_ERROR', genId, error: string }               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Frontend Error Handling                               │
│  - useSimulationWorker.error = error message                       │
│  - isRunning = false                                               │
│  - BackendStatusPanel shows error state                            │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Auto-Balance Errors

```
┌─────────────────────────────────────────────────────────────────────┐
│              Error Detection                                       │
│  - Failed to converge after max iterations                        │
│  - Invalid monster configuration                                   │
│  - Simulation error during balance                                │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Worker Error Response                                 │
│  { type: 'SIMULATION_ERROR', genId, error: string }               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│              Frontend Error Handling                               │
│  - Show error to user                                              │
│  - Keep original monsters (no changes applied)                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Performance Optimization Points

### Backend Optimizations

1. **Two-Pass Simulation**:
   - Phase 1: No event collection (~323 KB for 10,100 runs)
   - Phase 2: Seed selection (1% buckets)
   - Phase 3: Re-simulation only selected seeds (~4.2 MB)
   - **Savings**: ~10-15 MB memory

2. **Action Template Cache** (`action_cache.rs`):
   - Caches resolved template spells
   - Max 1000 entries LRU
   - **Savings**: Avoids repeated resolution

3. **Combat Stats Cache** (`context.rs`):
   - Caches combatant statistics
   - **Savings**: Faster lookups

4. **Memory Guardrails** (`memory_guardrails.rs`):
   - Auto-switches to lightweight mode at 10,000 iterations
   - **Savings**: Prevents OOM errors

---

### Frontend Optimizations

1. **WebWorker Isolation**:
   - Heavy computations off-main-thread
   - **Benefit**: UI remains responsive

2. **Debouncing** (`useAutoSimulation`):
   - 500ms delay before triggering simulation
   - **Benefit**: Prevents excessive simulations

3. **Canvas Rendering** (`SkylineCanvas`):
   - Single canvas for all skyline visualizations
   - `requestAnimationFrame` for smooth rendering
   - **Benefit**: Better performance than DOM nodes

4. **Incremental Updates**:
   - Worker streams results as they arrive
   - UI updates incrementally
   - **Benefit**: User sees progress immediately

---

### Optimization Flow

```
User edits → 500ms debounce → useSimulationWorker.runSimulation()
    ↓
WebWorker → WASM Backend → Two-Pass Simulation
    ↓
Phase 1: Lightweight (no events) → Fast completion
    ↓
Phase 2: Seed selection → Minimal memory
    ↓
Phase 3: Re-simulation → Only 170 seeds
    ↓
Results streamed incrementally → UI updates progressively
    ↓
User sees results faster
```

---

## References

- **ARCHITECTURE.md**: Comprehensive system architecture
- **BACKEND_API.md**: Rust/WASM function catalog
- **FRONTEND_API.md**: React/TypeScript component catalog
- **AGENTS.md**: Protocols and guidelines for LLM agents
