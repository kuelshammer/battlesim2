/* tslint:disable */
/* eslint-disable */

export class ChunkedSimulationRunner {
  free(): void;
  [Symbol.dispose](): void;
  constructor(players: any, timeline: any, iterations: number, seed?: bigint | null, k_factor?: number | null);
  finalize(): any;
  run_chunk(chunk_size: number): number;
}

export class ConfigManagerWrapper {
  free(): void;
  [Symbol.dispose](): void;
  markClean(): void;
  exportToJson(): string;
  getPreferences(): any;
  importFromJson(json: string): void;
  resetToDefaults(): void;
  updatePreferences(preferences: any): void;
  validatePreferences(): any;
  constructor();
  isDirty(): boolean;
}

export class DisplayManagerWrapper {
  free(): void;
  [Symbol.dispose](): void;
  getStatusText(): string;
  getDisplayMode(): string;
  setDisplayMode(mode: string): void;
  userSelectedSlot(slot: string): any;
  getDisplayResults(players: any, timeline: any, iterations: number): any;
  handleSimulationFailed(error: string): void;
  handleSimulationCompleted(slot: string): void;
  constructor();
}

export class ProgressUIManagerWrapper {
  free(): void;
  [Symbol.dispose](): void;
  getProgress(simulation_id: string): any;
  stopTracking(simulation_id: string): void;
  startTracking(simulation_id: string): void;
  getAllProgress(): any;
  getProgressSummary(): any;
  createCompactIndicator(simulation_id: string): string;
  createProgressBarHtml(simulation_id: string): string;
  constructor();
  clearAll(): void;
}

/**
 * Answer a confirmation request
 */
export function answer_confirmation(confirmation_id: string, confirmed: boolean): any;

export function auto_adjust_encounter_wasm(players: any, monsters: any, timeline: any, encounter_index: number): any;

/**
 * Cancel a running simulation
 */
export function cancel_simulation(simulation_id: string): any;

export function clear_simulation_cache(): void;

/**
 * Clear simulation cache
 */
export function clear_simulation_cache_gui(): any;

/**
 * Create compact progress indicator for a simulation
 */
export function create_compact_indicator(simulation_id: string): any;

/**
 * Create HTML progress bar for a simulation
 */
export function create_progress_bar(simulation_id: string): any;

/**
 * Get progress information for all active simulations
 */
export function get_all_progress(): any;

/**
 * Get cache statistics for memory monitoring
 * Returns an object with entry_count and estimated_bytes
 */
export function get_cache_stats(): any;

/**
 * Get current display mode
 */
export function get_display_mode(): any;

/**
 * Get display results for current parameters
 */
export function get_display_results(players: any, timeline: any, iterations: number): any;

export function get_last_simulation_events(): any;

/**
 * Get pending user confirmations
 */
export function get_pending_confirmations(): any;

/**
 * Get progress information for a specific simulation
 */
export function get_progress(simulation_id: string): any;

/**
 * Get progress summary for dashboard
 */
export function get_progress_summary(): any;

/**
 * Get current user interaction state
 */
export function get_user_interaction_state(): any;

/**
 * Handle parameter change event
 */
export function handle_parameters_changed(players: any, timeline: any, iterations: number): any;

/**
 * Initialize memory guardrails for safe simulation
 *
 * Call this once at application startup to set up:
 * - Panic hooks for user-friendly OOM error messages
 * - Memory safety checks for large simulations
 */
export function init_memory_guardrails(): void;

/**
 * Initialize the GUI integration system
 */
export function initialize_gui_integration(): any;

/**
 * Run a batch of simulations synchronously
 */
export function run_batch_simulation_wasm(batch_request: any): any;

/**
 * Run a batch of simulations with progress callback
 */
export function run_batch_simulation_with_callback(batch_request: any, callback: Function): any;

export function run_decile_analysis_wasm(results: any, scenario_name: string, _party_size: number): any;

export function run_event_driven_simulation(players: any, timeline: any, iterations: number): any;

export function run_simulation_wasm(players: any, timeline: any, iterations: number): any;

/**
 * O(1) memory WASM simulation using rolling aggregation
 * Returns SimulationSummary with aggregated statistics instead of all individual results
 */
export function run_simulation_wasm_rolling_stats(players: any, timeline: any, iterations: number, seed?: bigint | null): any;

export function run_simulation_with_callback(players: any, timeline: any, iterations: number, callback: Function): any;

/**
 * Run Skyline Spectrogram analysis - 100-bucket percentile aggregation
 *
 * Returns per-character HP/Resources data for visualization in the Skyline UI.
 * Results should be pre-sorted by score (worst to best).
 */
export function run_skyline_analysis_wasm(results: any, party_size: number, encounter_index?: number | null): any;

/**
 * Set display mode
 */
export function set_display_mode(mode_str: string): any;

/**
 * Check if a simulation size requires lightweight mode
 *
 * Returns true if iterations > 1000, which means full event logging
 * should be disabled to prevent out-of-memory errors.
 */
export function should_force_lightweight_mode(iterations: number): boolean;

/**
 * Start a background simulation
 */
export function start_background_simulation(players: any, timeline: any, iterations: number, priority_str: string): any;

/**
 * Update GUI configuration
 */
export function update_gui_configuration(display_config_json?: any | null, progress_config_json?: any | null, interaction_config_json?: any | null): any;

/**
 * User selected a specific slot
 */
export function user_selected_slot(slot_str: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_chunkedsimulationrunner_free: (a: number, b: number) => void;
  readonly __wbg_configmanagerwrapper_free: (a: number, b: number) => void;
  readonly __wbg_displaymanagerwrapper_free: (a: number, b: number) => void;
  readonly __wbg_progressuimanagerwrapper_free: (a: number, b: number) => void;
  readonly answer_confirmation: (a: number, b: number, c: number) => [number, number, number];
  readonly auto_adjust_encounter_wasm: (a: any, b: any, c: any, d: number) => [number, number, number];
  readonly cancel_simulation: (a: number, b: number) => [number, number, number];
  readonly chunkedsimulationrunner_finalize: (a: number) => [number, number, number];
  readonly chunkedsimulationrunner_new: (a: any, b: any, c: number, d: number, e: bigint, f: number) => [number, number, number];
  readonly chunkedsimulationrunner_run_chunk: (a: number, b: number) => number;
  readonly clear_simulation_cache: () => void;
  readonly clear_simulation_cache_gui: () => [number, number, number];
  readonly configmanagerwrapper_exportToJson: (a: number) => [number, number, number, number];
  readonly configmanagerwrapper_getPreferences: (a: number) => [number, number, number];
  readonly configmanagerwrapper_importFromJson: (a: number, b: number, c: number) => [number, number];
  readonly configmanagerwrapper_isDirty: (a: number) => number;
  readonly configmanagerwrapper_markClean: (a: number) => void;
  readonly configmanagerwrapper_new: () => number;
  readonly configmanagerwrapper_resetToDefaults: (a: number) => void;
  readonly configmanagerwrapper_updatePreferences: (a: number, b: any) => [number, number];
  readonly configmanagerwrapper_validatePreferences: (a: number) => [number, number, number];
  readonly create_compact_indicator: (a: number, b: number) => [number, number, number];
  readonly create_progress_bar: (a: number, b: number) => [number, number, number];
  readonly displaymanagerwrapper_getDisplayMode: (a: number) => [number, number];
  readonly displaymanagerwrapper_getDisplayResults: (a: number, b: any, c: any, d: number) => [number, number, number];
  readonly displaymanagerwrapper_getStatusText: (a: number) => [number, number];
  readonly displaymanagerwrapper_handleSimulationCompleted: (a: number, b: number, c: number) => [number, number];
  readonly displaymanagerwrapper_handleSimulationFailed: (a: number, b: number, c: number) => [number, number];
  readonly displaymanagerwrapper_new: () => [number, number, number];
  readonly displaymanagerwrapper_setDisplayMode: (a: number, b: number, c: number) => [number, number];
  readonly displaymanagerwrapper_userSelectedSlot: (a: number, b: number, c: number) => [number, number, number];
  readonly get_all_progress: () => [number, number, number];
  readonly get_cache_stats: () => any;
  readonly get_display_mode: () => [number, number, number];
  readonly get_display_results: (a: any, b: any, c: number) => [number, number, number];
  readonly get_last_simulation_events: () => [number, number, number];
  readonly get_pending_confirmations: () => [number, number, number];
  readonly get_progress: (a: number, b: number) => [number, number, number];
  readonly get_progress_summary: () => [number, number, number];
  readonly get_user_interaction_state: () => [number, number, number];
  readonly handle_parameters_changed: (a: any, b: any, c: number) => [number, number, number];
  readonly init_memory_guardrails: () => void;
  readonly initialize_gui_integration: () => [number, number, number];
  readonly progressuimanagerwrapper_clearAll: (a: number) => [number, number];
  readonly progressuimanagerwrapper_createCompactIndicator: (a: number, b: number, c: number) => [number, number, number, number];
  readonly progressuimanagerwrapper_createProgressBarHtml: (a: number, b: number, c: number) => [number, number, number, number];
  readonly progressuimanagerwrapper_getAllProgress: (a: number) => [number, number, number];
  readonly progressuimanagerwrapper_getProgress: (a: number, b: number, c: number) => [number, number, number];
  readonly progressuimanagerwrapper_getProgressSummary: (a: number) => [number, number, number];
  readonly progressuimanagerwrapper_new: () => [number, number, number];
  readonly progressuimanagerwrapper_startTracking: (a: number, b: number, c: number) => [number, number];
  readonly progressuimanagerwrapper_stopTracking: (a: number, b: number, c: number) => [number, number];
  readonly run_batch_simulation_wasm: (a: any) => [number, number, number];
  readonly run_batch_simulation_with_callback: (a: any, b: any) => [number, number, number];
  readonly run_decile_analysis_wasm: (a: any, b: number, c: number, d: number) => [number, number, number];
  readonly run_event_driven_simulation: (a: any, b: any, c: number) => [number, number, number];
  readonly run_simulation_wasm: (a: any, b: any, c: number) => [number, number, number];
  readonly run_simulation_wasm_rolling_stats: (a: any, b: any, c: number, d: number, e: bigint) => [number, number, number];
  readonly run_simulation_with_callback: (a: any, b: any, c: number, d: any) => [number, number, number];
  readonly run_skyline_analysis_wasm: (a: any, b: number, c: number) => [number, number, number];
  readonly set_display_mode: (a: number, b: number) => [number, number, number];
  readonly should_force_lightweight_mode: (a: number) => number;
  readonly start_background_simulation: (a: any, b: any, c: number, d: number, e: number) => [number, number, number];
  readonly update_gui_configuration: (a: number, b: number, c: number) => [number, number, number];
  readonly user_selected_slot: (a: number, b: number) => [number, number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
