import { useState, useCallback, useRef, useEffect } from 'react';
import { SimulationResult, FullAnalysisOutput, Creature, TimelineEvent, AutoAdjustmentResult } from '@/model/model';
import { getFinalAction } from "@/data/actions";

export interface SimulationWorkerState {
    isRunning: boolean;
    progress: number;
    completed: number;
    total: number;
    currentIterations: number;
    results: SimulationResult[] | null;
    analysis: FullAnalysisOutput | null;
    events: any[] | null;
    error: string | null;
    optimizedResult: AutoAdjustmentResult | null;
}

export function useSimulationWorker() {
    const [state, setState] = useState<SimulationWorkerState>({
        isRunning: false,
        progress: 0,
        completed: 0,
        total: 0,
        currentIterations: 0,
        results: null,
        analysis: null,
        events: null,
        error: null,
        optimizedResult: null,
    });

    const workerRef = useRef<Worker | null>(null);

    const setupWorkerListener = useCallback((worker: Worker) => {
        worker.onmessage = (e) => {
            const { type, progress, completed, total, results, analysis, events, error, result } = e.data;

            switch (type) {
                case 'SIMULATION_PROGRESS':
                    setState(prev => ({
                        ...prev,
                        progress: progress * 100,
                        completed,
                        total,
                        results: results || prev.results,
                        analysis: analysis || prev.analysis
                    }));
                    break;
                case 'SIMULATION_COMPLETE':
                    setState(prev => ({
                        ...prev,
                        isRunning: false,
                        progress: 100,
                        results,
                        analysis,
                        events,
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

    const runSimulation = useCallback((players: Creature[], timeline: TimelineEvent[], iterations: number = 2511) => {
        // Terminate existing worker if running
        terminateAndRestart();
        
        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            completed: 0,
            total: iterations,
            currentIterations: iterations,
            error: null,
            optimizedResult: null
        }));

        // Clean data before sending to worker
        const cleanPlayers = players.map(p => ({
            ...p,
            actions: p.actions.map(getFinalAction)
        }));

        if (!Array.isArray(timeline)) {
            console.error("runSimulation called with non-array timeline:", timeline);
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
            return event; // shortRest needs no specific cleaning for now
        });

        workerRef.current?.postMessage({
            type: 'START_SIMULATION',
            players: cleanPlayers,
            timeline: cleanTimeline,
            iterations
        });
    }, [terminateAndRestart]);

    const autoAdjustEncounter = useCallback((players: Creature[], monsters: Creature[], timeline: TimelineEvent[], encounterIndex: number) => {
        // Terminate existing worker if running
        terminateAndRestart();
        
        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            completed: 0,
            total: 1, // Only one step conceptually
            error: null,
            optimizedResult: null
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

        workerRef.current?.postMessage({
            type: 'AUTO_ADJUST_ENCOUNTER',
            players: cleanPlayers,
            monsters: cleanMonsters,
            timeline: cleanTimeline,
            encounterIndex
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