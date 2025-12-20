
import init, { run_simulation_with_callback, run_decile_analysis_wasm } from 'simulation-wasm';
const wasmUrl = require('simulation-wasm/simulation_wasm_bg.wasm');

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init({ module_or_path: wasmUrl });
        wasmInitialized = true;
    }
}

self.onmessage = async (e: MessageEvent) => {
    const { type, players, encounters, iterations } = e.data;

    if (type === 'START_SIMULATION') {
        try {
            await ensureWasmInitialized();

            const progressCallback = (progress: number, completed: number, total: number, partialData?: any) => {
                self.postMessage({
                    type: 'SIMULATION_PROGRESS',
                    progress,
                    completed,
                    total,
                    results: partialData?.results,
                    analysis: partialData?.analysis
                });
            };

            const output = run_simulation_with_callback(
                players,
                encounters,
                iterations,
                progressCallback
            );

            const { results, analysis, first_run_events } = output;

            self.postMessage({
                type: 'SIMULATION_COMPLETE',
                results,
                analysis,
                events: first_run_events
            });
        } catch (error) {
            console.error('Worker simulation error:', error);
            self.postMessage({
                type: 'SIMULATION_ERROR',
                error: error instanceof Error ? error.message : String(error)
            });
        }
    }
};
