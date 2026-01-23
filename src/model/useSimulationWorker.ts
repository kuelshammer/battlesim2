import { useState, useCallback, useRef, useEffect } from 'react';
import { SimulationResult, FullAnalysisOutput, Creature, TimelineEvent, AutoAdjustmentResult } from '@/model/model';
import { getFinalAction } from "@/data/actions";
import { SimulationEvent } from '@/model/events';
import { SimulationWorkerController } from '@/worker/simulation.worker.controller';

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
    isCancelled: boolean;
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
        genId: 0,
        isCancelled: false
    });

    const controllerRef = useRef<SimulationWorkerController | null>(null);
    const currentGenIdRef = useRef(0);

    const handleStructuredResult = useCallback((result: any) => {
        // Discard messages from old generations
        if (result.genId !== undefined && result.genId < currentGenIdRef.current) return;

        switch (result.type) {
            case 'completed':
                if (result.result !== undefined) {
                    // Auto-adjust result
                    setState(prev => ({
                        ...prev,
                        isRunning: false,
                        progress: 100,
                        optimizedResult: result.result,
                        error: null,
                        isCancelled: false
                    }));
                } else {
                    // Simulation update
                    setState(prev => ({
                        ...prev,
                        isRunning: !result.isFinal,
                        progress: (result.kFactor / prev.maxK) * 100,
                        kFactor: result.kFactor,
                        results: result.results,
                        analysis: result.analysis,
                        events: result.events || prev.events,
                        error: null,
                        isCancelled: false
                    }));
                }
                break;
            case 'cancelled':
                setState(prev => ({
                    ...prev,
                    isRunning: false,
                    isCancelled: true
                }));
                break;
            case 'errored':
                setState(prev => ({
                    ...prev,
                    isRunning: false,
                    error: result.error,
                    isCancelled: false
                }));
                break;
        }
    }, []);

    useEffect(() => {
        // Initialize controller with centralized worker creation
        controllerRef.current = new SimulationWorkerController();

        return () => {
            controllerRef.current?.terminate();
        };
    }, []);

    const terminateAndRestart = useCallback(() => {
        controllerRef.current?.terminate();
        controllerRef.current = new SimulationWorkerController();
        return controllerRef.current;
    }, []);

    const runSimulation = useCallback((players: Creature[], timeline: TimelineEvent[], maxK: number = 51, seed?: number) => {
        // Cancel any existing simulation first
        controllerRef.current?.cancel();

        // Increment Generation ID
        currentGenIdRef.current += 1;
        const genId = currentGenIdRef.current;

        // Ensure controller exists
        if (!controllerRef.current) {
            terminateAndRestart();
        }

        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            kFactor: 0,
            maxK,
            error: null,
            optimizedResult: null,
            isCancelled: false,
            genId
        }));

        const cleanPlayers = players.map(p => {
            // Flatten magicItems[].buffs into initialBuffs
            const magicItemBuffs = p.magicItems?.flatMap(item => item.buffs) ?? [];
            const mergedInitialBuffs = [...(p.initialBuffs ?? []), ...magicItemBuffs];

            return {
                ...p,
                actions: p.actions.map(getFinalAction),
                initialBuffs: mergedInitialBuffs
            };
        });

        const cleanTimeline = timeline.map(event => {
            if (event.type === 'combat') {
                return {
                    ...event,
                    monsters: event.monsters.map(m => {
                        // Flatten magicItems[].buffs into initialBuffs for monsters too
                        const magicItemBuffs = m.magicItems?.flatMap(item => item.buffs) ?? [];
                        const mergedInitialBuffs = [...(m.initialBuffs ?? []), ...magicItemBuffs];

                        return {
                            ...m,
                            actions: m.actions.map(getFinalAction),
                            initialBuffs: mergedInitialBuffs
                        };
                    })
                };
            }
            return event;
        });

        controllerRef.current?.startSimulation(cleanPlayers, cleanTimeline, genId, maxK, seed, handleStructuredResult);
    }, [terminateAndRestart, handleStructuredResult]);

    const autoAdjustEncounter = useCallback((players: Creature[], monsters: Creature[], timeline: TimelineEvent[], encounterIndex: number) => {
        // Cancel any existing simulation first
        controllerRef.current?.cancel();

        // Increment Generation ID to stop any pending simulation updates
        currentGenIdRef.current += 1;
        const genId = currentGenIdRef.current;

        // Ensure controller exists
        if (!controllerRef.current) {
            terminateAndRestart();
        }

        setState(prev => ({
            ...prev,
            isRunning: true,
            progress: 0,
            kFactor: 0,
            maxK: 1, // Auto-adjust is a single step
            error: null,
            optimizedResult: null,
            isCancelled: false,
            genId
        }));

        // Clean data
        const cleanPlayers = players.map(p => {
            // Flatten magicItems[].buffs into initialBuffs
            const magicItemBuffs = p.magicItems?.flatMap(item => item.buffs) ?? [];
            const mergedInitialBuffs = [...(p.initialBuffs ?? []), ...magicItemBuffs];

            return {
                ...p,
                actions: p.actions.map(getFinalAction),
                initialBuffs: mergedInitialBuffs
            };
        });

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
                    monsters: event.monsters.map(m => {
                        // Flatten magicItems[].buffs into initialBuffs for monsters too
                        const magicItemBuffs = m.magicItems?.flatMap(item => item.buffs) ?? [];
                        const mergedInitialBuffs = [...(m.initialBuffs ?? []), ...magicItemBuffs];

                        return {
                            ...m,
                            actions: m.actions.map(getFinalAction),
                            initialBuffs: mergedInitialBuffs
                        };
                    })
                };
            }
            return event;
        });

        controllerRef.current?.autoAdjustEncounter(cleanPlayers, cleanMonsters, cleanTimeline, encounterIndex, genId, handleStructuredResult);
    }, [terminateAndRestart, handleStructuredResult]);

    const clearOptimizedResult = useCallback(() => {
        setState(prev => ({ ...prev, optimizedResult: null }));
    }, []);

    const cancel = useCallback(() => {
        controllerRef.current?.cancel();
        setState(prev => ({
            ...prev,
            isRunning: false,
            isCancelled: true
        }));
    }, []);

    return {
        ...state,
        runSimulation,
        autoAdjustEncounter,
        clearOptimizedResult,
        terminateAndRestart,
        cancel
    };
}