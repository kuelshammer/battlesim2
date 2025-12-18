# BattleSim UI Enhancement Plan: Non-Blocking Auto-Updating Architecture

## Problem Statement
The current BattleSim UI freezes during simulations because the Rust/WASM simulation runs on the main thread. This creates a poor user experience, especially when users make rapid changes to their encounters.

## Proposed Solution: Web Worker + Debounce Architecture

### 1. The "Golden Rule": Move Simulation to a Web Worker
- **Current Issue:** WASM runs on Main Thread, causing UI freezes
- **Solution:** Move entire Rust simulation engine to a Web Worker
- **Benefit:** UI remains responsive while simulation runs in background

### 2. Handling "Dirty State" (Rapid Fire Problem)
- **Current Issue:** Multiple rapid changes queue up simulations
- **Solution:** Debounce + Worker Termination
  - Wait 500ms-1000ms after user input
  - If new input comes in, terminate existing worker and restart
- **Benefit:** Only processes latest state, prevents unnecessary calculations

### 3. UX Solution: "Stale" Simulation Results
- **Step 1:** User edits a monster's AC
- **Step 2:** Mark current results as "Stale" (lower opacity, greyscale, or badge)
- **Step 3:** Show non-intrusive loading spinner
- **Step 4:** Update results when worker finishes

### 4. Implementation Sketch (Frontend)
```javascript
let simulationWorker = null;
let debounceTimer = null;

function handleUserChange(newState) {
    // 1. Mark UI as "stale" / dirty immediately
    setUiState('stale');

    // 2. Debounce: Clear previous timer
    clearTimeout(debounceTimer);

    // 3. Set new timer (e.g., 1 second)
    debounceTimer = setTimeout(() => {
        runSimulation(newState);
    }, 1000);
}

function runSimulation(state) {
    // 4. Kill existing worker if it's still crunching old data
    if (simulationWorker) {
        simulationWorker.terminate();
    }

    // 5. Start fresh
    simulationWorker = new Worker('simulation.js');
    
    simulationWorker.postMessage(state);

    simulationWorker.onmessage = (e) => {
        updateResults(e.data);
        setUiState('fresh');
        simulationWorker = null; // Cleanup
    };
}
```

### 5. Advanced Feature: "Live Update" Toggle
- **Auto-Simulate (ON - Default):** Uses Worker + Debounce for real-time updates
- **Auto-Simulate (OFF):** Shows "Stale" badge but doesn't run until manual "Run Simulation" click

## Summary Recommendation
1. **Don't** use manual button as primary method - feels clunky
2. **Do** use **Web Worker** to unblock UI immediately  
3. **Do** use **`worker.terminate()`** to handle dirty state cleanly
4. **Do** implement "Stale" UI feedback for better user experience

## Next Steps
- Implement Web Worker architecture
- Add debounce logic for user input
- Create "Stale" UI state indicators
- Add "Auto-Simulate" toggle for power users
- Test performance and responsiveness improvements