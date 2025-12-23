
import init, { run_simulation_with_callback, auto_adjust_encounter_wasm } from 'simulation-wasm';
import wasmUrl from 'simulation-wasm/simulation_wasm_bg.wasm';

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init({ module_or_path: wasmUrl });
        wasmInitialized = true;
    }
}

export const handleMessage = async (e: MessageEvent) => {
    const { type: messageType, players, timeline, monsters, iterations, encounterIndex } = e.data;

    if (messageType === 'START_SIMULATION') {
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
    } else if (messageType === 'AUTO_ADJUST_ENCOUNTER') {
        const { players, monsters, timeline, encounterIndex } = e.data;
        try {
            await ensureWasmInitialized();
            const adjustmentResult = auto_adjust_encounter_wasm(players, monsters, timeline, encounterIndex);
            self.postMessage({
                type: 'AUTO_ADJUST_COMPLETE',
                result: adjustmentResult
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

self.onmessage = handleMessage;
