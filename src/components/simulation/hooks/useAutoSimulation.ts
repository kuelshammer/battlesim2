import { useState, useEffect, useRef } from "react"
import { EncounterResult as EncounterResultType } from "@/model/model"
import { SimulationEvent } from "@/model/events"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import { useStoredState } from "@/model/utils"
import { Creature, TimelineEvent } from "@/model/model"

export interface AutoSimulationState {
    simulationResults: EncounterResultType[]
    simulationEvents: SimulationEvent[]
    needsResimulation: boolean
    isStale: boolean
    highPrecision: boolean
    isHighPrecisionLoaded: boolean
    isEditing: boolean
    canSave: boolean
}

export interface AutoSimulationActions {
    setHighPrecision: (highPrecision: boolean) => void
    setIsEditing: (isEditing: boolean) => void
    setSaving: (saving: boolean) => void
    setLoading: (loading: boolean) => void
    triggerResimulation: () => void
}

export interface AutoSimulationSelectors {
    worker: ReturnType<typeof useSimulationWorker>
}

export const useAutoSimulation = (
    players: Creature[],
    timeline: TimelineEvent[],
    isPlayersLoaded: boolean,
    isTimelineLoaded: boolean
): AutoSimulationState & AutoSimulationActions & AutoSimulationSelectors => {
    const [simulationResults, setSimulationResults] = useState<EncounterResultType[]>([])
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])
    const [needsResimulation, setNeedsResimulation] = useState(false)
    const [isStale, setIsStale] = useState(false)
    const [highPrecision, setHighPrecision, isHighPrecisionLoaded] = useStoredState<boolean>('highPrecision', false, Boolean)
    const [isEditing, setIsEditing] = useState(false)
    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [canSave, setCanSave] = useState(false)

    // Web Worker Simulation
    const worker = useSimulationWorker()
    const debounceTimerRef = useRef<NodeJS.Timeout | null>(null)

    // Expose for E2E tests
    useEffect(() => {
        if (isPlayersLoaded && isTimelineLoaded && isHighPrecisionLoaded) {
            (window as Window & { simulationWasm?: boolean; storageLoaded?: boolean }).simulationWasm = true;
            (window as Window & { simulationWasm?: boolean; storageLoaded?: boolean }).storageLoaded = true;
        }
    }, [isPlayersLoaded, isTimelineLoaded, isHighPrecisionLoaded])

    useEffect(() => {
        const hasPlayers = !!players.length
        const hasMonsters = !!timeline.find(item => item.type === 'combat' && !!item.monsters.length)
        setCanSave(!hasPlayers && !hasMonsters ? false : true)
    }, [players.length, timeline])

    // Detect changes that need resimulation with debounce
    useEffect(() => {
        // Clear previous timer
        if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);

        // Set new timer (500ms delay)
        debounceTimerRef.current = setTimeout(() => {
            setNeedsResimulation(true);
            setIsStale(true);
        }, 500);

        return () => {
            if (debounceTimerRef.current) clearTimeout(debounceTimerRef.current);
        };
    }, [players, timeline, highPrecision]);

    // Trigger simulation when not editing and needs resimulation
    useEffect(() => {
        if (!isEditing && !saving && !loading && needsResimulation) {
            if (Array.isArray(timeline) && timeline.length > 0) {
                worker.runSimulation(players, timeline, highPrecision ? 51 : 3);
                setNeedsResimulation(false);
            }
        }
    }, [isEditing, saving, loading, needsResimulation, players, timeline, worker, highPrecision]);

    // Update display results when worker finishes
    useEffect(() => {
        if (!worker.results || worker.results.length === 0) return;

        const results = worker.results;
        // [Disaster, Struggle, Typical, Heroic, Legend]
        const index = results.length >= 3 ? 2 : 0;
        const selectedRun = results[index];

        setSimulationResults(selectedRun.encounters);

        // Update events from the first run returned by the worker
        if (worker.events) {
            // events in worker are already structured objects, not strings
            // But let's check if they need parsing
            setSimulationEvents(worker.events as SimulationEvent[]);
        }

        // Clear stale state when new results are available
        setIsStale(false);
    }, [worker.results, worker.events]);

    const triggerResimulation = () => {
        setNeedsResimulation(true)
        setIsStale(true)
    }

    return {
        // State
        simulationResults,
        simulationEvents,
        needsResimulation,
        isStale,
        highPrecision,
        isHighPrecisionLoaded,
        isEditing,
        canSave,

        // Actions
        setHighPrecision,
        setIsEditing,
        setSaving,
        setLoading,
        triggerResimulation,

        // Selectors
        worker,
    }
}