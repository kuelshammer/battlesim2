# GUI Refactoring Guide: Removing Legacy Simulation

## Objective
Update the React frontend to exclusively use the event-driven simulation engine, removing all conditional logic and UI toggles related to the legacy simulation. This aligns the frontend with the backend refactoring.

## Guidelines for the AI Agent
1.  **Safety First:** Ensure the application always compiles (`npm run build` or similar checking).
2.  **User Experience:** The user should no longer see an option to switch simulation modes; it should "just work" using the new engine.
3.  **Code Quality:** proper error handling and cleanups.

---

## Phase 1: Refactor Simulation Logic

The main logical change happens in the simulation component.

### Step 1.1: Modify `src/components/simulation/simulation.tsx`
*   **State Removal:**
    *   Locate `const [useEventDriven, setUseEventDriven] = useState(true)`.
    *   Delete this line.
*   **Logic Simplification:**
    *   Inside the `useEffect` hook (around line 123), verify the `if (useEventDriven)` block.
    *   **Action:** Convert the conditional logic to be unconditional.
        *   Keep the code that calls `wasm.run_event_driven_simulation(...)`.
        *   Keep the code that fetches and parses events (`wasm.get_last_simulation_events()`).
        *   **Delete** the entire `else` block (which called `wasm.run_simulation_wasm` and `setSimulationEvents([])`).
    *   **Result:** The simulation should ALWAYS run the event-driven path and ALWAYs capture events.

### Step 1.2: Remove UI Toggle
*   **Locate:** Find the JSX element (button or checkbox) that triggers `setUseEventDriven(!useEventDriven)` (likely around line 255).
*   **Action:** Remove this element entirely.
*   **Cleanup:** Remove any labels or wrapper divs associated solely with this toggle.

## Phase 2: Verification

### Step 2.1: check build
*   Ensure the changes didn't break TS compilation.

### Step 2.2: Verify references
*   Search the entire `src` directory for `run_simulation_wasm`.
*   If `src/components/simulation/simulation.tsx` was the only place, you are done.
*   If found elsewhere, assess if it should also be switched to `run_event_driven_simulation`.

## Final Output
A `simulation.tsx` file that cleanly executes the new event-driven simulation without legacy code paths or unused state variables.
