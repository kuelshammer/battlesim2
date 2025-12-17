
import init, { run_simulation_with_callback, run_quintile_analysis_wasm } from '../../public/simulation_wasm.js';

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

self.onmessage = async (e: MessageEvent) => {
    const { type, players, encounters, iterations } = e.data;

    if (type === 'START_SIMULATION') {
        try {
            await ensureWasmInitialized();

            const progressCallback = (progress: number, completed: number, total: number) => {
                self.postMessage({
                    type: 'SIMULATION_PROGRESS',
                    progress,
                    completed,
                    total
                });
            };

            const output = run_simulation_with_callback(
                players,
                encounters,
                iterations,
                progressCallback
            );

            const { results, first_run_events } = output;

            // Run quintile analysis
            const partySize = players.length;
            const scenarioName = "Current Scenario";
            const analysis = run_quintile_analysis_wasm(results, scenarioName, partySize);

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
