import React, { FC, useEffect, useState, useRef } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, SimulationResult } from "../../model/model"
import { parseEventString, SimulationEvent } from "../../model/events"
import { clone, useStoredState } from "../../model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import EventLog from "../combat/EventLog"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash } from "@fortawesome/free-solid-svg-icons"
import { semiPersistentContext } from "../../model/simulationContext"
import AdventuringDayForm from "./adventuringDayForm"
import { getFinalAction } from "../../data/actions"

console.log("Simulation module evaluating...");

try {
    console.log("Imports check:", { getFinalAction: !!getFinalAction });
} catch (e) {
    console.error("Error checking imports:", e);
}

type PropType = {
    // TODO
}

const emptyEncounter: Encounter = {
    monsters: [],
    monstersSurprised: false,
    playersSurprised: false,
}

const Simulation: FC<PropType> = ({ }) => {
    const [players, setPlayers] = useStoredState<Creature[]>('players', [], z.array(CreatureSchema).parse)
    const [encounters, setEncounters] = useStoredState<Encounter[]>('encounters', [emptyEncounter], z.array(EncounterSchema).parse)
    const [luck, setLuck] = useStoredState<number>('luck', 0.5, z.number().min(0).max(1).parse)
    const [simulationResults, setSimulationResults] = useState<SimulationResult>([])
    const [state, setState] = useState(new Map<string, any>())
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])
    const [useEventDriven, setUseEventDriven] = useState(true) // Toggle for testing

    function isEmpty() {
        const hasPlayers = !!players.length
        const hasMonsters = !!encounters.find(encounter => !!encounter.monsters.length)
        return !hasPlayers && !hasMonsters
    }

    // Helper to get combatant names for the log
    const combatantNames = new Map<string, string>();
    players.forEach(p => combatantNames.set(p.id, p.name));
    encounters.forEach(e => e.monsters.forEach(m => combatantNames.set(m.id, m.name)));
    // Also add numbered versions if IDs differ (simplified)

    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [canSave, setCanSave] = useState(false)
    useEffect(() => {
        setCanSave(
            !isEmpty()
            && (typeof window !== "undefined")
            && !!localStorage
            && !!localStorage.getItem('useLocalStorage')
        )
    }, [players, encounters])

    const [wasm, setWasm] = useState<typeof import('simulation-wasm') | null>(null)
    const [allResults, setAllResults] = useState<SimulationResult[]>([])
    const wasmLoading = useRef(false)

    useEffect(() => {
        // Load WASM module using the original working approach
        const loadWasm = async () => {
            if (wasmLoading.current) return
            wasmLoading.current = true
            
            try {
                console.log('🔄 Loading WASM module...')
                import('simulation-wasm').then(async (module) => {
                    console.log('✅ WASM package imported')
                    // Pass object to avoid deprecation warning
                    await module.default({ module_or_path: '/simulation_wasm_bg.wasm' })
                    console.log('✅ WASM initialized successfully!')
                    console.log('Available functions:', Object.keys(module).filter(k => typeof (module as Record<string, unknown>)[k] === 'function'))
                    setWasm(module)
                }).catch(error => {
                    console.error('❌ WASM import failed:', error)
                    wasmLoading.current = false
                })
            } catch (error) {
                console.error('❌ WASM loading error:', error)
                wasmLoading.current = false
            }
        }

        if (!wasm) {
            loadWasm()
        }
    }, [wasm])


    useEffect(() => {
        if (!wasm) return

        if (allResults.length === 0) {
            // Run simulation if not cached
            try {
                const cleanPlayers = players.map(p => ({
                    ...p,
                    actions: p.actions.map(getFinalAction)
                }))

                const cleanEncounters = encounters.map(e => ({
                    ...e,
                    monsters: e.monsters.map(m => ({
                        ...m,
                        actions: m.actions.map(getFinalAction)
                    }))
                }))

                let results: SimulationResult[]

                // Use event-driven simulation if enabled
                if (useEventDriven) {
                    console.log('Running event-driven simulation...')
                    results = wasm.run_event_driven_simulation(cleanPlayers, cleanEncounters, 1005) as SimulationResult[]

                    // Get events from the simulation
                    try {
                        const rawEvents = wasm.get_last_simulation_events() as string[]
                        const structuredEvents = rawEvents.map(parseEventString).filter((e): e is SimulationEvent => e !== null);
                        setSimulationEvents(structuredEvents)
                        console.log('Events collected and parsed:', structuredEvents.length)
                    } catch (eventError) {
                        console.error("Failed to get events:", eventError)
                        setSimulationEvents([])
                    }
                } else {
                    console.log('Running legacy simulation...')
                    results = wasm.run_simulation_wasm(cleanPlayers, cleanEncounters, 1005) as SimulationResult[]
                    setSimulationEvents([])
                }

                console.log('Simulation complete. Results:', results.length, 'runs')
                // console.log('First result:', results[0])

                setAllResults(results)

                // Select single run based on luck
                const total = results.length
                const index = Math.min(total - 1, Math.floor(luck * total))
                const selectedRun = results[index]

                setSimulationResults(selectedRun)
                console.log('Simulation results set!')
            } catch (e) {
                console.error("Simulation failed", e)
                setSimulationEvents([])
            }
        } else {
            // Update selection based on new luck
            try {
                const total = allResults.length
                const index = Math.min(total - 1, Math.floor(luck * total))
                const selectedRun = allResults[index]

                setSimulationResults(selectedRun)
            } catch (e) {
                console.error("Selection failed", e)
            }
        }
    }, [players, encounters, luck, wasm, useEventDriven]) // Removed allResults to prevent loop

    // Reset results when inputs change
    useEffect(() => {
        setAllResults([])
        setSimulationEvents([])
    }, [players, encounters])


    function createEncounter() {
        setEncounters([...encounters, {
            monsters: [],
            monstersSurprised: false,
            playersSurprised: false,
        }])
    }

    function updateEncounter(index: number, newValue: Encounter) {
        const encountersClone = clone(encounters)
        encountersClone[index] = newValue
        setEncounters(encountersClone)
    }

    function deleteEncounter(index: number) {
        if (encounters.length <= 1) return // Must have at least one encounter
        const encountersClone = clone(encounters)
        encountersClone.splice(index, 1)
        setEncounters(encountersClone)
    }

    function swapEncounters(index1: number, index2: number) {
        const encountersClone = clone(encounters)
        const tmp = encountersClone[index1]
        encountersClone[index1] = encountersClone[index2]
        encountersClone[index2] = tmp
        setEncounters(encountersClone)
    }

    return (
        <div className={styles.simulation}>
            <semiPersistentContext.Provider value={{ state, setState }}>
                <h1 className={styles.header}>BattleSim</h1>

                {/* Backend Features Status Panel */}
                {useEventDriven && (
                    <div className={styles.backendStatus}>
                        <h4>🔧 Event-Driven Backend Active</h4>
                        <div className={styles.statusItems}>
                            <span>✅ ActionResolution Engine</span>
                            <span>✅ Event System</span>
                            <span>✅ Reaction Processing</span>
                            <span>✅ Effect Tracking</span>
                            <span>📊 Events: {simulationEvents.length}</span>
                        </div>
                    </div>
                )}

                <EncounterForm
                    mode='player'
                    encounter={{ monsters: players }}
                    onUpdate={(newValue) => setPlayers(newValue.monsters)}
                    luck={luck}
                    setLuck={setLuck}>
                    <>
                        {!isEmpty() ? (
                            <button onClick={() => { setPlayers([]); setEncounters([emptyEncounter]) }}>
                                <FontAwesomeIcon icon={faTrash} />
                                Clear Adventuring Day
                            </button>
                        ) : null}
                        {canSave ? (
                            <button onClick={() => setSaving(true)}>
                                <FontAwesomeIcon icon={faSave} />
                                Save Adventuring Day
                            </button>
                        ) : null}
                        <button onClick={() => setLoading(true)}>
                            <FontAwesomeIcon icon={faFolder} />
                            Load Adventuring Day
                        </button>

                        {/* Toggle for testing event-driven vs legacy simulation */}
                        <button
                            onClick={() => setUseEventDriven(!useEventDriven)}
                            style={{
                                backgroundColor: useEventDriven ? '#4CAF50' : '#f44336',
                                color: 'white'
                            }}
                        >
                            {useEventDriven ? 'Event-Driven ON' : 'Legacy Mode'}
                        </button>

                    </>
                </EncounterForm>

                {encounters.map((encounter, index) => (
                    <div className={styles.encounter} key={index}>
                        <EncounterForm
                            mode='monster'
                            encounter={encounter}
                            onUpdate={(newValue) => updateEncounter(index, newValue)}
                            onDelete={(index > 0) ? () => deleteEncounter(index) : undefined}
                            onMoveUp={(!!encounters.length && !!index) ? () => swapEncounters(index, index - 1) : undefined}
                            onMoveDown={(!!encounters.length && (index < encounters.length - 1)) ? () => swapEncounters(index, index + 1) : undefined}
                            luck={luck}
                            setLuck={setLuck}
                        />
                        {(!simulationResults[index] ? null : (
                            <EncounterResult value={simulationResults[index]} />
                        ))}
                    </div>
                ))}

                <button
                    onClick={createEncounter}
                    className={styles.addEncounterBtn}>
                    <FontAwesomeIcon icon={faPlus} />
                    Add Encounter
                </button>

                {/* Event Log Display */}
                {useEventDriven && (
                    <EventLog
                        events={simulationEvents}
                        combatantNames={Object.fromEntries(combatantNames)}
                    />
                )}

                {(saving || loading) ? (
                    <AdventuringDayForm
                        currentPlayers={players} // New prop name
                        currentEncounters={encounters} // New prop name
                        onCancel={() => { setSaving(false); setLoading(false) }}
                        onApplyChanges={(newPlayers, newEncounters) => { // New prop name and logic
                            setPlayers(newPlayers)
                            setEncounters(newEncounters)
                            setSaving(false)
                            setLoading(false)
                        }}
                    />
                ) : null}
            </semiPersistentContext.Provider>
        </div>
    )
}

export default Simulation