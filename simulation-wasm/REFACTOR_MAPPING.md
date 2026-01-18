# WASM API Refactor Mapping

This document serves as the blueprint for refactoring `simulation-wasm/src/wasm_api.rs` into focused modules. The goal is to reduce `wasm_api.rs` to ≤ 200 LOC by extracting business logic to the orchestration layer and moving repetitive wrappers to `api/wasm.rs`.

## Module Summary
- **orchestration/simulation.rs**: New module for complex simulation and analysis orchestration.
- **orchestration/gui.rs**: Existing module for GUI orchestration (Rust side).
- **api/wasm.rs**: Existing module for WASM-specific DTOs and common bindings/helpers.
- **wasm_api.rs**: Minimal entry point (≤ 200 LOC) containing only the top-level `#[wasm_bindgen]` exports.

## Function Inventory & Mapping

| Function / Item | Lines | LOC | Category | Current Responsibility | Destination Module | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Imports & Types** | | | | | | |
| `AutoAdjustmentResult` | 17-22 | 6 | type-definition | DTO for auto-adjustment results | `api/wasm.rs` | Standard WASM DTO. |
| **Simulation API** | | | | | | |
| `auto_adjust_encounter_wasm` | 28-43 | 16 | thin-wrapper | WASM binding for auto-balancing encounters | `api/wasm.rs` | Pure delegation to `AutoBalancer`. |
| `run_simulation_wasm` | 55-74 | 20 | thin-wrapper | Basic simulation WASM binding | `api/wasm.rs` | Simple delegation to `runners`. |
| `run_simulation_with_callback` | 76-143 | 68 | complex-logic | Three-phase simulation orchestration | `orchestration/simulation.rs` | Extract complex multi-phase logic. |
| `run_event_driven_simulation` | 145-172 | 28 | complex-logic | Legacy simulation with state storage | `delete` | Redundant with `run_simulation_with_callback`. |
| `get_last_simulation_events` | 174-180 | 7 | thin-wrapper | Retrieve stored events from state | `api/wasm.rs` | Simple state retrieval. |
| `run_simulation_wasm_rolling_stats` | 182-195 | 14 | thin-wrapper | Run simulation with three-tier stats | `api/wasm.rs` | Delegates to `two_pass`. |
| `run_batch_simulation_wasm` | 197-209 | 13 | thin-wrapper | Run a batch of simulations | `api/wasm.rs` | Batch processing orchestration. |
| `run_batch_simulation_with_callback` | 211-229 | 19 | thin-wrapper | Batch simulation with progress | `api/wasm.rs` | Batch processing with JS callback. |
| **Analysis API** | | | | | | |
| `run_skyline_analysis_wasm` | 245-270 | 26 | complex-logic | Skyline analysis orchestration | `orchestration/simulation.rs` | Extract sorting and mapping logic. |
| `run_decile_analysis_wasm` | 272-312 | 41 | complex-logic | Decile analysis orchestration | `orchestration/simulation.rs` | Extract complex analysis steps. |
| **GUI API** | | | | | | |
| `initialize_gui_integration` | 318-322 | 5 | thin-wrapper | Initialize GUI state | `api/wasm.rs` | Pure delegation. |
| `get_display_results` | 324-330 | 7 | thin-wrapper | Get results formatted for GUI | `api/wasm.rs` | Pure delegation. |
| `set_display_mode` | 332-345 | 14 | thin-wrapper | Set GUI display mode | `api/wasm.rs` | Mapping and delegation. |
| `get_display_mode` | 347-359 | 13 | thin-wrapper | Get current GUI display mode | `api/wasm.rs` | Delegation and mapping. |
| `user_selected_slot` | 361-369 | 9 | thin-wrapper | Handle user slot selection | `api/wasm.rs` | Mapping and delegation. |
| `start_background_simulation` | 371-392 | 22 | thin-wrapper | Trigger background simulation | `api/wasm.rs` | Delegation. |
| `get_all_progress` | 394-397 | 4 | thin-wrapper | Get progress for all simulations | `api/wasm.rs` | Pure delegation. |
| `get_progress` | 399-406 | 8 | thin-wrapper | Get progress for specific sim | `api/wasm.rs` | Pure delegation. |
| `create_progress_bar` | 408-412 | 5 | thin-wrapper | Generate progress bar HTML | `api/wasm.rs` | Pure delegation. |
| `create_compact_indicator` | 414-418 | 5 | thin-wrapper | Generate compact indicator HTML | `api/wasm.rs` | Pure delegation. |
| `cancel_simulation` | 420-424 | 5 | thin-wrapper | Cancel an active simulation | `api/wasm.rs` | Pure delegation. |
| `clear_simulation_cache_gui` | 426-429 | 4 | thin-wrapper | Clear cache via GUI layer | `api/wasm.rs` | Pure delegation. |
| `get_pending_confirmations` | 431-434 | 4 | thin-wrapper | Check for pending user actions | `api/wasm.rs` | Pure delegation. |
| `answer_confirmation` | 436-439 | 4 | thin-wrapper | Submit user confirmation | `api/wasm.rs` | Pure delegation. |
| `get_user_interaction_state` | 441-444 | 4 | thin-wrapper | Get current interaction state | `api/wasm.rs` | Pure delegation. |
| `update_gui_configuration` | 446-458 | 13 | thin-wrapper | Update GUI settings | `api/wasm.rs` | Mapping and delegation. |
| `get_progress_summary` | 460-463 | 4 | thin-wrapper | Get overall progress summary | `api/wasm.rs` | Pure delegation. |
| `handle_parameters_changed` | 465-474 | 10 | thin-wrapper | Notify parameter changes | `api/wasm.rs` | Pure delegation. |
| **System & Helpers** | | | | | | |
| `init_memory_guardrails` | 45-48 | 4 | thin-wrapper | Initialize memory limits | `stay-in-wasm_api` | Essential system hook. |
| `should_force_lightweight_mode` | 50-53 | 4 | thin-wrapper | Check memory limits | `stay-in-wasm_api` | Essential system hook. |
| `clear_simulation_cache` | 231-234 | 4 | thin-wrapper | Clear global cache | `stay-in-wasm_api` | Essential system hook. |
| `get_cache_stats` | 236-243 | 8 | thin-wrapper | Get cache statistics | `stay-in-wasm_api` | Essential system hook. |
| `parse_js_value` | 486-489 | 4 | helper | Deserialize JsValue | `api/wasm.rs` | Common WASM helper. |
| `serialize_result` | 491-495 | 5 | helper | Serialize to JsValue | `api/wasm.rs` | Common WASM helper. |
| `report_progress` | 497-507 | 11 | helper | JS progress callback | `api/wasm.rs` | Common WASM helper. |

## Extraction Plan

### 1. Complex Logic Extraction
Move the following to `orchestration/simulation.rs`:
- `run_simulation_with_callback` logic (Three-phase orchestration).
- `run_skyline_analysis_wasm` logic.
- `run_decile_analysis_wasm` logic.
- Related imports from `sorting` and `aggregation`.

### 2. Wrapper Relocation
Move all GUI-related and non-essential simulation wrappers to `api/wasm.rs`. 
This includes:
- All 17 GUI functions (lines 318-474).
- Batch and stats simulation wrappers.

### 3. Cleaning `wasm_api.rs`
The final `wasm_api.rs` should contain:
- Essential imports and `wasm_bindgen` setup.
- Core simulation entry points (as thin re-exports or very thin wrappers).
- System-level hooks (memory guardrails, cache management).
- Total estimated size: ~120-150 LOC.
