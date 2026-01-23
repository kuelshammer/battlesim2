import { SimulationWorkerController } from './simulation.worker.controller';

// Create a single controller instance for the worker
// Allow test injection of worker
const controller = new SimulationWorkerController(
  typeof globalThis !== 'undefined' && (globalThis as any).testWorker
    ? (globalThis as any).testWorker
    : undefined
);



export const handleMessage = async (e: MessageEvent) => {
    const { type: messageType, players, timeline, genId, seed, maxK = 51, monsters, encounterIndex } = e.data;

    const onResult = (result: any) => {
        // Map structured results to the expected message format
        switch (result.type) {
            case 'completed':
                if (result.result !== undefined) {
                    // Auto-adjust result
                    self.postMessage({
                        type: 'AUTO_ADJUST_COMPLETE',
                        genId: result.genId,
                        result: result.result
                    });
                } else {
                    // Simulation update
                    self.postMessage({
                        type: 'SIMULATION_UPDATE',
                        genId: result.genId,
                        results: result.results,
                        analysis: result.analysis,
                        events: result.events,
                        kFactor: result.kFactor,
                        isFinal: result.isFinal
                    });
                }
                break;
            case 'cancelled':
                // Send a cancellation message
                self.postMessage({
                    type: 'SIMULATION_CANCELLED',
                    genId: result.genId
                });
                break;
            case 'errored':
                self.postMessage({
                    type: 'SIMULATION_ERROR',
                    genId: result.genId,
                    error: result.error
                });
                break;
        }
    };

    if (messageType === 'START_SIMULATION') {
        await controller.startSimulation(players, timeline, genId, maxK, seed, onResult);
    }
    else if (messageType === 'AUTO_ADJUST_ENCOUNTER') {
        await controller.autoAdjustEncounter(players, monsters || [], timeline, encounterIndex, genId, onResult);
    }
    else if (messageType === 'CANCEL_SIMULATION') {
        controller.cancel();
    }
};

self.onmessage = handleMessage;