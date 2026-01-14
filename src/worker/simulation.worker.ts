import init, { ChunkedSimulationRunner, auto_adjust_encounter_wasm } from 'simulation-wasm';
import wasmUrl from 'simulation-wasm/simulation_wasm_bg.wasm';

let wasmInitialized = false;
let currentGenId = 0;
let activeRunner: ChunkedSimulationRunner | null = null;

async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init({ module_or_path: wasmUrl });
        wasmInitialized = true;
    }
}

function refineSimulation(genId: number, targetK: number, maxK: number) {

    // STOP if generation changed

    if (genId !== currentGenId || !activeRunner) return;



    // STOP if reached requested max precision

    if (targetK > maxK) return;



    // Calculate runs needed to reach next K

    // N = (2K-1) * 100

    // K=1: 100

    // K=2: 300 (+200)

    // K=3: 500 (+200)

    const incrementalRuns = 200;



    try {

        activeRunner.run_chunk(incrementalRuns);



        // Check GenID again after potentially slow WASM work

        if (genId !== currentGenId) return;



        const output = activeRunner.get_analysis(targetK);

        const { results, analysis, firstRunEvents } = output;



        self.postMessage({

            type: 'SIMULATION_UPDATE',

            genId,

            results,

            analysis,

            events: firstRunEvents,

            kFactor: targetK,

            isFinal: targetK === maxK

        });



        if (targetK < maxK) {

            setTimeout(() => refineSimulation(genId, targetK + 1, maxK), 0);

        }

    } catch (error) {

        console.error('Refinement error:', error);

    }

}



export const handleMessage = async (e: MessageEvent) => {

    const { type: messageType, players, timeline, genId, seed, maxK = 51 } = e.data;



    if (messageType === 'START_SIMULATION') {

        try {

            await ensureWasmInitialized();



            // 1. Update Generation ID

            currentGenId = genId;



            // 2. Initialize Runner

            activeRunner = new ChunkedSimulationRunner(players, timeline, seed);



            // 3. Initial Pass (K=1, 100 runs)

            activeRunner.run_chunk(100);



            const output = activeRunner.get_analysis(1);

            const { results, analysis, firstRunEvents } = output;



            // 4. Send Instant Result

            self.postMessage({

                type: 'SIMULATION_UPDATE',

                genId,

                results,

                analysis,

                events: firstRunEvents,

                kFactor: 1,

                isFinal: maxK <= 1

            });



            // 5. Start Background Refinement

            if (maxK > 1) {

                setTimeout(() => refineSimulation(genId, 2, maxK), 0);

            }



        } catch (error) {

            console.error('Worker simulation error:', error);

            self.postMessage({

                type: 'SIMULATION_ERROR',

                genId,

                error: error instanceof Error ? error.message : String(error)

            });

        }

    }

    else if (messageType === 'AUTO_ADJUST_ENCOUNTER') {
        const { players, monsters, timeline, encounterIndex, genId } = e.data;
        try {
            await ensureWasmInitialized();
            const adjustmentResult = auto_adjust_encounter_wasm(players, monsters, timeline, encounterIndex);
            self.postMessage({
                type: 'AUTO_ADJUST_COMPLETE',
                genId,
                result: adjustmentResult
            });
        } catch (error) {
            console.error('Worker auto-adjust error:', error);
            self.postMessage({
                type: 'SIMULATION_ERROR',
                genId,
                error: error instanceof Error ? error.message : String(error)
            });
        }
    }
};

self.onmessage = handleMessage;