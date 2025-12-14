import React, { FC, useEffect, useState, useRef, memo, useMemo } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, SimulationResult } from "@/model/model"
import { parseEventString, SimulationEvent } from "@/model/events"
import { clone, useStoredState } from "@/model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import EventLog from "../combat/EventLog"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash } from "@fortawesome/free-solid-svg-icons"
import { semiPersistentContext } from "@/model/simulationContext"
import AdventuringDayForm from "./adventuringDayForm"
import { getFinalAction } from "@/data/actions"



type PropType = {
    // TODO
}

const emptyEncounter: Encounter = {
    monsters: [],
    monstersSurprised: false,
    playersSurprised: false,
}

const Simulation: FC<PropType> = memo(({ }) => {
    const [players, setPlayers] = useStoredState<Creature[]>('players', [], z.array(CreatureSchema).parse)
    const [encounters, setEncounters] = useStoredState<Encounter[]>('encounters', [emptyEncounter], z.array(EncounterSchema).parse)
    const [luck, setLuck] = useStoredState<number>('luck', 0.5, z.number().min(0).max(1).parse)
    const [simulationResults, setSimulationResults] = useState<SimulationResult>([])
    const [state, setState] = useState(new Map<string, any>())
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])

    // Memoize expensive computations
    const isEmptyResult = useMemo(() => {
        const hasPlayers = !!players.length
        const hasMonsters = !!encounters.find(encounter => !!encounter.monsters.length)
        return !hasPlayers && !hasMonsters
    }, [players.length, encounters])

    // Memoize combatant names map
    const combatantNames = useMemo(() => {
        const names = new Map<string, string>()
        players.forEach(p => names.set(p.id, p.name))
        encounters.forEach(e => e.monsters.forEach(m => names.set(m.id, m.name)))
        return names
    }, [players, encounters])

    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    
    // Memoize canSave computation
    const canSave = useMemo(() => (
        !isEmptyResult
        && (typeof window !== "undefined")
        && !!localStorage
        && !!localStorage.getItem('useLocalStorage')
    ), [isEmptyResult])

    const [wasm, setWasm] = useState<typeof import('simulation-wasm') | null>(null)
    const [allResults, setAllResults] = useState<SimulationResult[]>([])
    const wasmLoading = useRef(false)

    useEffect(() => {
        // Load WASM module using the original working approach
        const loadWasm = async () => {
            if (wasmLoading.current) return
            wasmLoading.current = true

            try {
            import('simulation-wasm').then(async (module) => {
                    // Pass object to avoid deprecation warning
                    await module.default({ module_or_path: '/simulation_wasm_bg.wasm' })
                    setWasm(module)
                }).catch(error => {
                    wasmLoading.current = false
                })

            } catch (error) {
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

                console.log('Running event-driven simulation...')
                results = wasm.run_event_driven_simulation(cleanPlayers, cleanEncounters, 1005) as SimulationResult[]

                // Get events from the simulation
                try {
                    const rawEvents = wasm.get_last_simulation_events() as string[]
                    const structuredEvents = rawEvents.map(parseEventString).filter((e): e is SimulationEvent => e !== null);
                    setSimulationEvents(structuredEvents)
                    console.log('Events collected and parsed:', structuredEvents.length)
                } catch (eventError) {
                    setSimulationEvents([])
                }

                setAllResults(results)

                // Select single run based on luck
                const total = results.length
                const index = Math.min(total - 1, Math.floor(luck * total))
                const selectedRun = results[index]

                setSimulationResults(selectedRun)
            } catch (e) {
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
    }, [players, encounters, luck, wasm]) // Removed allResults and useEventDriven to prevent loop

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

    // Memoize action names map
    const actionNames = useMemo(() => {
        const names = new Map<string, string>()
        players.forEach(p => p.actions.forEach(a => {
            const name = a.type === 'template' ? a.templateOptions.templateName : a.name
            names.set(a.id, name)
        }))
        encounters.forEach(e => e.monsters.forEach(m => m.actions.forEach(a => {
            const name = a.type === 'template' ? a.templateOptions.templateName : a.name
            names.set(a.id, name)
        })))
        return names
    }, [players, encounters])

    return (
        <div className={styles.simulation}>
            <semiPersistentContext.Provider value={{ state, setState }}>
                <h1 className={styles.header}>BattleSim</h1>

                {/* Backend Features Status Panel */}
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

                <EncounterForm
                    mode='player'
                    encounter={{ monsters: players }}
                    onUpdate={(newValue) => setPlayers(newValue.monsters)}
                    luck={luck}
                    setLuck={setLuck}>
                    <>
                        {!isEmptyResult ? (
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
                <EventLog
                    events={simulationEvents}
                    combatantNames={Object.fromEntries(combatantNames)}
                    actionNames={Object.fromEntries(actionNames)}
                />

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
})

export default Simulation