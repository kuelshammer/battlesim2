import init, { ChunkedSimulationRunner, auto_adjust_encounter_wasm } from 'simulation-wasm';
import wasmUrl from 'simulation-wasm/simulation_wasm_bg.wasm';

let wasmInitialized = false;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init({ module_or_path: wasmUrl });
        wasmInitialized = true;
    }
}

export const handleMessage = async (e: MessageEvent) => {
    const { type: messageType, players, timeline, monsters, iterations, encounterIndex, seed, preciseMode } = e.data;

    if (messageType === 'START_SIMULATION') {
        try {
            await ensureWasmInitialized();

            const runner = new ChunkedSimulationRunner(players, timeline, iterations, seed, preciseMode);
            const CHUNK_SIZE = 500;
            
            const runChunk = () => {
                const progress = runner.run_chunk(CHUNK_SIZE);
                const completed = Math.min(iterations, Math.floor(progress * iterations / 0.8));
                
                self.postMessage({
                    type: 'SIMULATION_PROGRESS',
                    progress,
                    completed,
                    total: iterations
                });

                if (completed < iterations) {
                    // Use setTimeout to yield to the event loop and allow termination/responsiveness
                    setTimeout(runChunk, 0);
                } else {
                    const output = runner.finalize();
                    const { results, analysis, firstRunEvents } = output;

                    self.postMessage({
                        type: 'SIMULATION_COMPLETE',
                        results,
                        analysis,
                        events: firstRunEvents
                    });
                }
            };

            runChunk();
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