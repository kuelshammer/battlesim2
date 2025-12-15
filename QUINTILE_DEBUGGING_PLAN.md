# Quintile Analysis Debugging Plan

## Problem Summary

The quintile analysis is partially working:
- ✅ **CLI works perfectly** - All 5 quintiles generate with proper data
- ✅ **Worst 20% quintile displays correctly** - Shows HP bars and names
- ❌ **GUI only shows first quintile** - Quintiles 2-5 show "Loading battle details..."
- ❌ **WASM crashes** - `RuntimeError: unreachable` when processing quintiles 2-5

## Root Cause Analysis

**The issue is WASM-specific, not in the core quintile analysis logic.** The CLI proves that:
- ✅ Quintile analysis algorithm works perfectly
- ✅ All 5 quintiles generate with proper data  
- ✅ 1005 simulation runs are sufficient
- ✅ Data extraction and processing are functional

**The problem is in the WASM/JavaScript integration layer.** Most likely causes:

### 1. Data Marshaling Problems
- JavaScript object conversion might be corrupting `SimulationResult` data
- TypeScript type mismatches between WASM output and frontend expectations
- Array slicing issues when passing data from JavaScript to Rust

### 2. WASM Memory/Serialization Issues  
- WASM memory constraints might be truncating larger data sets
- Serde serialization might be failing for complex nested data
- JavaScript `from_value()` conversion might be losing data

### 3. JavaScript-Side Data Processing
- Frontend might be modifying data before passing to WASM
- Type coercion might be changing data structure
- Async timing issues between data generation and WASM call

## Diagnostic Plan

### Phase 1: Add WASM Debug Logging (Critical)
1. Add console logging in WASM function to show:
   - Total results received: `results.len()`
   - Results structure validation
   - Each quintile's data extraction success/failure
   - Bounds checking triggers
   - What `extract_combatant_visualization` returns

2. Add Rust debug output using `web_sys::console::log_1()`

3. Log party size calculation to verify it's correct

### Phase 2: Validate Data Flow (High Priority)
1. Check JavaScript data structure before WASM call
2. Verify WASM return value matches expected format
3. Test with minimal data to isolate marshaling issues
4. Compare CLI vs GUI data for same scenario

### Phase 3: Fix Integration Issues (Medium Priority)
1. Fix data serialization if corruption is detected
2. Improve error handling in WASM binding
3. Add data validation on both sides of the boundary
4. Ensure consistent data structure between CLI and GUI

### Phase 4: Test with Known Good Data (Medium Priority)
1. Test with simple 1v1 scenario first to isolate complexity issues
2. Ensure 1005 simulation runs are generated
3. Verify GUI shows correct number of simulation runs
4. Test with scenario that works in CLI

## Questions for User

1. Can you test with a simple 1v1 scenario in the GUI? This would help isolate whether the issue is related to multi-combat complexity.

2. Are there any browser console errors when quintile analysis runs in GUI? There might be JavaScript errors that aren't crashing the app but corrupting data.

3. Does the GUI show the correct number of simulation runs somewhere in the interface? It should indicate 1005 runs for quintile analysis.

## Expected Outcomes

After implementing this plan:
- All 5 quintiles should display with HP bars and real names
- No more `RuntimeError: unreachable` crashes
- Consistent behavior between CLI and GUI
- Proper error handling and debugging information

## Next Steps

1. Implement comprehensive debug logging in WASM quintile analysis function
2. Add data validation and error reporting
3. Test with simple scenarios to isolate issues
4. Fix any data marshaling or serialization problems discovered
5. Ensure consistent behavior between CLI and GUI implementations

The CLI analysis proves the core logic is perfect, so we need to focus on the WASM/JavaScript integration layer to fix why only the first quintile is working in the GUI.