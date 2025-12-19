import { useState, useCallback, useRef, useEffect } from 'react';
import { SimulationResult, FullAnalysisOutput, Creature, Encounter } from '@/model/model';
import { getFinalAction } from "@/data/actions";

export interface SimulationWorkerState {
    isRunning: boolean;
    progress: number;
    completed: number;
    total: number;
    results: SimulationResult[] | null;
    analysis: FullAnalysisOutput | null;
    events: any[] | null;
    error: string | null;
}

export function useSimulationWorker() {
    const [state, setState] = useState<SimulationWorkerState>({
        isRunning: false,
        progress: 0,
        completed: 0,
        total: 0,
        results: null,
        analysis: null,
        events: null,
        error: null,
    });

    const workerRef = useRef<Worker | null>(null);

    const setupWorkerListener = useCallback((worker: Worker) => {
        worker.onmessage = (e) => {
            const { type, progress, completed, total, results, analysis, events, error } = e.data;

            switch (type) {
                case 'SIMULATION_PROGRESS':
                    setState(prev => ({
                        ...prev,
                        progress: progress * 100,
                        completed,
                        total
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

    const runSimulation = useCallback((players: Creature[], encounters: Encounter[], iterations: number = 2510) => {
        // Terminate existing worker if running
        terminateAndRestart();
        
        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            completed: 0,
            total: iterations,
            error: null
        }));

        // Clean data before sending to worker
        const cleanPlayers = players.map(p => ({
            ...p,
            actions: p.actions.map(getFinalAction)
        }));

        const cleanEncounters = encounters.map(e => ({
            ...e,
            monsters: e.monsters.map(m => ({
                ...m,
                actions: m.actions.map(getFinalAction)
            }))
        }));

        workerRef.current?.postMessage({
            type: 'START_SIMULATION',
            players: cleanPlayers,
            encounters: cleanEncounters,
            iterations
        });
    }, [terminateAndRestart]);

    return {
        ...state,
        runSimulation,
        terminateAndRestart
    };
}