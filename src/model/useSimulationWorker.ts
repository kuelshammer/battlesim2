import { useState, useCallback, useRef, useEffect } from 'react';
import { SimulationResult, FullAnalysisOutput, Creature, TimelineEvent, AutoAdjustmentResult } from '@/model/model';
import { getFinalAction } from "@/data/actions";
import { SimulationEvent } from '@/model/events';

export interface SimulationWorkerState {
    isRunning: boolean;
    progress: number;
    kFactor: number;
    maxK: number;
    results: SimulationResult[] | null;
    analysis: FullAnalysisOutput | null;
    events: SimulationEvent[] | null;
    error: string | null;
    optimizedResult: AutoAdjustmentResult | null;
    genId: number;
}

export function useSimulationWorker() {
    const [state, setState] = useState<SimulationWorkerState>({
        isRunning: false,
        progress: 0,
        kFactor: 0,
        maxK: 51,
        results: null,
        analysis: null,
        events: null,
        error: null,
        optimizedResult: null,
        genId: 0
    });

    const workerRef = useRef<Worker | null>(null);
    const currentGenIdRef = useRef(0);

    const setupWorkerListener = useCallback((worker: Worker) => {
        worker.onmessage = (e) => {
            const { type, genId, results, analysis, events, error, result, kFactor, isFinal } = e.data;

            // Discard messages from old generations
            if (genId !== undefined && genId < currentGenIdRef.current) return;

            switch (type) {
                case 'SIMULATION_UPDATE':
                    setState(prev => ({
                        ...prev,
                        isRunning: !isFinal,
                        progress: (kFactor / prev.maxK) * 100,
                        kFactor,
                        results,
                        analysis,
                        events: events || prev.events,
                        error: null
                    }));
                    break;
                case 'AUTO_ADJUST_COMPLETE':
                    setState(prev => ({
                        ...prev,
                        isRunning: false,
                        progress: 100,
                        optimizedResult: result,
                        error: null
                    }));
                    break;
                case 'SIMULATION_ERROR':
                    setState(prev => ({
                        ...prev,
                        isRunning: false,
                        error
                    }));
                    break;
            }
        };
    }, []);

    useEffect(() => {
        // Initialize worker
        const worker = new Worker(new URL('../worker/simulation.worker.ts', import.meta.url));
        setupWorkerListener(worker);
        workerRef.current = worker;

        return () => {
            worker.terminate();
        };
    }, [setupWorkerListener]);

    const terminateAndRestart = useCallback(() => {
        if (workerRef.current) {
            workerRef.current.terminate();
            workerRef.current = null;
        }
        
        // Re-initialize worker
        const worker = new Worker(new URL('../worker/simulation.worker.ts', import.meta.url));
        setupWorkerListener(worker);
        workerRef.current = worker;
        
        return worker;
    }, [setupWorkerListener]);

    const runSimulation = useCallback((players: Creature[], timeline: TimelineEvent[], maxK: number = 51, seed?: number) => {
        // Increment Generation ID
        currentGenIdRef.current += 1;
        const genId = currentGenIdRef.current;

        // Note: We DON'T necessarily need to terminate and restart if the worker is responsive.
        // But for safety against overlapping loops from the same worker (if any survived), 
        // we can still restart or just rely on the genId check in the worker refinement loop.
        // Given the instructions, we'll keep the worker alive but the refinement loop will stop itself.
        if (!workerRef.current) {
            terminateAndRestart();
        }
        const worker = workerRef.current!;

        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            kFactor: 0,
            maxK,
            error: null,
            optimizedResult: null,
            genId
        }));

        const cleanPlayers = players.map(p => ({
            ...p,
            actions: p.actions.map(getFinalAction)
        }));

        const cleanTimeline = timeline.map(event => {
            if (event.type === 'combat') {
                return {
                    ...event,
                    monsters: event.monsters.map(m => ({
                        ...m,
                        actions: m.actions.map(getFinalAction)
                    }))
                };
            }
            return event;
        });

        worker.postMessage({
            type: 'START_SIMULATION',
            players: cleanPlayers,
            timeline: cleanTimeline,
            genId,
            seed,
            maxK
        });
    }, [terminateAndRestart]);

    const autoAdjustEncounter = useCallback((players: Creature[], monsters: Creature[], timeline: TimelineEvent[], encounterIndex: number) => {
        // Increment Generation ID to stop any pending simulation updates
        currentGenIdRef.current += 1;
        const genId = currentGenIdRef.current;

        // Always terminate and restart to clear worker state
        const worker = terminateAndRestart();

        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            kFactor: 0,
            maxK: 1, // Auto-adjust is a single step
            error: null,
            optimizedResult: null,
            genId
        }));

        // Clean data
        const cleanPlayers = players.map(p => ({
            ...p,
            actions: p.actions.map(getFinalAction)
        }));

        const cleanMonsters = monsters.map(m => ({
            ...m,
            actions: m.actions.map(getFinalAction)
        }));

        if (!Array.isArray(timeline)) {
            console.error("autoAdjustEncounter called with non-array timeline:", timeline);
            return;
        }

        const cleanTimeline = timeline.map(event => {
            if (event.type === 'combat') {
                return {
                    ...event,
                    monsters: event.monsters.map(m => ({
                        ...m,
                        actions: m.actions.map(getFinalAction)
                    }))
                };
            }
            return event;
        });

        worker.postMessage({
            type: 'AUTO_ADJUST_ENCOUNTER',
            players: cleanPlayers,
            monsters: cleanMonsters,
            timeline: cleanTimeline,
            encounterIndex,
            genId
        });
    }, [terminateAndRestart]);

    const clearOptimizedResult = useCallback(() => {
        setState(prev => ({ ...prev, optimizedResult: null }));
    }, []);

    return {
        ...state,
        runSimulation,
        autoAdjustEncounter,
        clearOptimizedResult,
        terminateAndRestart
    };
}