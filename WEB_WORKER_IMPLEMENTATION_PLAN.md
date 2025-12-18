# BattleSim Web Worker Architecture Implementation Plan

## Current State Analysis

**✅ What we already have:**
- Web Worker implementation (`useSimulationWorker.ts` and `simulation.worker.ts`)
- Basic simulation state management with `needsResimulation` flag
- Worker runs simulation when not editing and needs resimulation

**❌ What's missing from the plan:**
- Debounce logic for rapid user input
- Worker termination on new input
- "Stale" UI state indicators
- Auto-simulate toggle
- Proper dirty state handling

## Step-by-Step Implementation Plan

### Step 1: Add Debounce Logic to State Management (✅ Completed)
- Create a debounce timer for user input changes
- Reset timer on each change, only trigger simulation after delay
- Prevent multiple rapid simulations from queuing up

### Step 2: Implement Worker Termination (✅ Completed)
- Add logic to terminate existing worker when new debounced input comes in
- Ensure clean worker cleanup to prevent memory leaks

### Step 3: Add "Stale" UI State Indicators (✅ Completed)
- Create state for tracking "stale" vs "fresh" simulation results
- Add visual indicators (opacity, badges) when results are out of date
- Show loading spinner during simulation

### Step 4: Create Auto-Simulate Toggle (✅ Completed)
- Add toggle switch for "Auto-Simulate" functionality
- Default to ON for automatic updates
- Allow users to disable auto-simulation for complex setups

### Step 5: Refactor Simulation Trigger Logic (✅ Completed)
- Replace `needsResimulation` with more sophisticated state management
- Integrate debounce and worker termination logic
- Ensure proper state updates and UI feedback

### Step 6: Update UI Components (✅ Completed)
- Add "Stale" indicators to EncounterResult components
- Update button states based on simulation status
- Ensure proper visual feedback during simulations

## Detailed Implementation Steps

### Step 1: Add Debounce Logic
```typescript
// In simulation.tsx
const [debounceTimer, setDebounceTimer] = useState<NodeJS.Timeout | null>(null);
const [isStale, setIsStale] = useState(false);

useEffect(() => {
    // Clear previous timer
    if (debounceTimer) clearTimeout(debounceTimer);
    
    // Set new timer (500ms delay)
    const timer = setTimeout(() => {
        setNeedsResimulation(true);
        setIsStale(true);
    }, 500);
    
    setDebounceTimer(timer);
    
    return () => {
        if (debounceTimer) clearTimeout(debounceTimer);
    };
}, [players, encounters]);
```

### Step 2: Implement Worker Termination
```typescript
// In useSimulationWorker.ts
const terminateAndRestart = useCallback(() => {
    if (workerRef.current) {
        workerRef.current.terminate();
        workerRef.current = null;
    }
    // Re-initialize worker
    const worker = new Worker(new URL('../worker/simulation.worker.ts', import.meta.url));
    workerRef.current = worker;
}, []);

const runSimulation = useCallback((players: Creature[], encounters: Encounter[], iterations: number = 1005) => {
    terminateAndRestart();
    // ... rest of runSimulation logic
}, [terminateAndRestart]);
```

### Step 3: Add "Stale" UI State
```typescript
// In simulation.tsx
const EncounterResult: FC<{ value: SimulationResult; analysis: AggregateOutput | null }> = ({ value, analysis }) => {
    const [isStale, setIsStale] = useState(false);
    
    return (
        <div className={`${styles.encounterResult} ${isStale ? styles.stale : ''}`}>
            {isStale && <div className={styles.staleBadge}>Out of Date</div>}
            {/* ... existing result display */}
        </div>
    );
};
```

### Step 4: Create Auto-Simulate Toggle
```typescript
// In simulation.tsx
const [autoSimulate, setAutoSimulate] = useState(true);

// In useEffect for triggering simulation
useEffect(() => {
    if (!autoSimulate) return; // Don't auto-trigger if disabled
    
    if (!isEditing && !saving && !loading && needsResimulation && !worker.isRunning) {
        worker.runSimulation(players, encounters, 1005);
        setNeedsResimulation(false);
        setIsStale(false);
    }
}, [autoSimulate, isEditing, saving, loading, needsResimulation, worker.isRunning, players, encounters, worker]);
```

### Step 5: Refactor Simulation Trigger Logic
```typescript
// Replace current useEffect with:
useEffect(() => {
    if (!autoSimulate) return;
    
    if (!isEditing && !saving && !loading && needsResimulation && !worker.isRunning) {
        console.log('Triggering background simulation...');
        worker.runSimulation(players, encounters, 1005);
        setNeedsResimulation(false);
        setIsStale(false);
    }
}, [autoSimulate, isEditing, saving, loading, needsResimulation, worker.isRunning, players, encounters, worker]);
```

### Step 6: Update UI Components
- Add "Stale" CSS classes to styles
- Update button states to reflect simulation status
- Ensure proper visual feedback during simulations

## File Changes Required

1. `src/components/simulation/simulation.tsx` - Add debounce, stale state, auto-simulate toggle
2. `src/model/useSimulationWorker.ts` - Add worker termination logic
3. `src/components/simulation/simulation.module.scss` - Add stale state styles
4. `src/components/simulation/encounterResult.tsx` - Add stale state indicators

## Testing Plan

1. Test rapid input changes to ensure only latest simulation runs
2. Verify worker termination works correctly
3. Check "Stale" UI indicators appear and disappear
4. Test auto-simulate toggle functionality
5. Ensure UI remains responsive during long simulations

This plan will transform the current implementation into a non-blocking, auto-updating architecture as outlined in the enhancement plan.