
import init, { run_simulation_with_callback, auto_adjust_encounter_wasm } from 'simulation-wasm';
const wasmUrl = require('simulation-wasm/simulation_wasm_bg.wasm');

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init({ module_or_path: wasmUrl });
        wasmInitialized = true;
    }
}

self.onmessage = async (e: MessageEvent) => {
    const { type, players, timeline, monsters, iterations } = e.data;

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
                timeline,
                iterations,
                progressCallback
            );

            // output is a JS object with camelCase properties from WASM
            const { results, analysis, firstRunEvents } = output;

            self.postMessage({
                type: 'SIMULATION_COMPLETE',
                results,
                analysis,
                events: firstRunEvents
            });
        } catch (error) {
            console.error('Worker simulation error:', error);
            self.postMessage({
                type: 'SIMULATION_ERROR',
                error: error instanceof Error ? error.message : String(error)
            });
        }
    } else if (type === 'AUTO_ADJUST_ENCOUNTER') {
        try {
            await ensureWasmInitialized();
            const result = auto_adjust_encounter_wasm(players, monsters);
            self.postMessage({
                type: 'AUTO_ADJUST_COMPLETE',
                result
            });
        } catch (error) {
            console.error('Worker auto-adjust error:', error);
            self.postMessage({
                type: 'SIMULATION_ERROR',
                error: error instanceof Error ? error.message : String(error)
            });
        }
    }
};
