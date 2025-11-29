import { FC, useEffect, useState } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, SimulationResult } from "../../model/model"
import { clone, useStoredState } from "../../model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash } from "@fortawesome/free-solid-svg-icons"
import { semiPersistentContext } from "../../model/simulationContext"
import AdventuringDayForm from "./adventuringDayForm"

// Ensure crypto is available globally for wasm-bindgen BEFORE any WASM imports
if (typeof window !== 'undefined' && typeof window.crypto !== 'undefined') {
    if (!globalThis.crypto) {
        globalThis.crypto = window.crypto;
    }
    // Also set on self for compatibility with different bundlers
    if (typeof self !== 'undefined' && !self.crypto) {
        (self as any).crypto = window.crypto;
    }
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

    function isEmpty() {
        const hasPlayers = !!players.length
        const hasMonsters = !!encounters.find(encounter => !!encounter.monsters.length)
        return !hasPlayers && !hasMonsters
    }

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
    const [medianLog, setMedianLog] = useState<string | null>(null)
    const [showLog, setShowLog] = useState(false)

    useEffect(() => {
        const loadWasmModule = async () => {
            // Skip Electron-specific path due to crypto import issues
            // Always use web fallback which properly handles wasm-bindgen imports
            // if (typeof window !== 'undefined' && window.electronAPI) {
            //     try {
            //         const wasmBytes = await window.electronAPI.loadWasm('simulation_wasm_bg.wasm');
            //         const wasmModule = await import('simulation-wasm');
            //         await wasmModule.default(wasmBytes); 
            //         setWasm(wasmModule);
            //     } catch (error) {
            //         console.error('Failed to load WASM in Electron:', error);
            //     }
            // } else {
            // Web environment - this works in both browser and Electron
            import('simulation-wasm').then(async (module) => {
                await module.default();
                setWasm(module);
            }).catch(error => {
                console.error('Failed to load WASM:', error);
            });
            // }
        };
        loadWasmModule();
    }, []);


    useEffect(() => {
        if (!wasm) return

        // Map luck (0-1) to index (0-4)
        const index = Math.min(4, Math.floor(luck * 5))

        if (allResults.length === 0) {
            // Run simulation if not cached
            try {
                // Import getFinalAction
                const { getFinalAction } = require('../../data/actions')

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

                // The return type is now just SimulationResult[]
                const results = wasm.run_simulation_wasm(cleanPlayers, cleanEncounters, 1005) as SimulationResult[]
                console.log('Simulation complete. Results:', results.length, 'runs')
                console.log('First result:', results[0])

                setAllResults(results)
                setMedianLog(null) // Median log is now written to file instead

                // Aggregate results based on luck slice
                const total = results.length
                const sliceSize = Math.floor(total / 5) // 20% slice
                const start = Math.min(total - sliceSize, Math.floor(luck * (total - sliceSize))) // Ensure start index is valid
                const end = start + sliceSize

                const slice = results.slice(start, end)
                console.log('Aggregating slice:', start, 'to', end, '(', slice.length, 'results)')

                // Aggregate
                const aggregated = wasm.aggregate_simulation_results(slice) as any
                console.log('Aggregated result:', aggregated)

                // Construct synthetic EncounterResult with proper structure
                const syntheticResult = [{
                    rounds: aggregated,
                    stats: new Map(),
                }]

                setSimulationResults(syntheticResult)
                console.log('Simulation results set!')
            } catch (e) {
                console.error("Simulation failed", e)
            }
        } else {
            // Update aggregation based on new luck
            try {
                const total = allResults.length
                const sliceSize = Math.floor(total / 5)
                const start = Math.min(total - sliceSize, Math.floor(luck * (total - sliceSize)))
                const end = start + sliceSize

                const slice = allResults.slice(start, end)
                const aggregated = wasm.aggregate_simulation_results(slice) as any

                const syntheticResult = [{
                    rounds: aggregated,
                    stats: new Map(),
                }]

                setSimulationResults(syntheticResult)
            } catch (e) {
                console.error("Aggregation failed", e)
            }
        }
    }, [players, encounters, luck, wasm, allResults])

    // Reset results when inputs change
    useEffect(() => {
        setAllResults([])
        setMedianLog(null)
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
                        {medianLog && (
                            <button onClick={() => setShowLog(true)}>
                                <FontAwesomeIcon icon={faFolder} />
                                View Median Run Log
                            </button>
                        )}
                        {!saving ? null : (
                            <AdventuringDayForm
                                players={players}
                                encounters={encounters}
                                onCancel={() => setSaving(false)} />
                        )}
                        {!loading ? null : (
                            <AdventuringDayForm
                                players={players}
                                encounters={encounters}
                                onCancel={() => setLoading(false)}
                                onLoad={(p, e) => {
                                    setPlayers(p)
                                    setEncounters(e)
                                    setLoading(false)
                                }} />
                        )}
                        {showLog && medianLog && (
                            <div style={{
                                position: 'fixed',
                                top: 0,
                                left: 0,
                                width: '100%',
                                height: '100%',
                                backgroundColor: 'rgba(0,0,0,0.8)',
                                zIndex: 1000,
                                display: 'flex',
                                justifyContent: 'center',
                                alignItems: 'center',
                            }}>
                                <div style={{
                                    backgroundColor: '#222',
                                    color: '#eee',
                                    padding: '20px',
                                    borderRadius: '8px',
                                    width: '80%',
                                    height: '80%',
                                    display: 'flex',
                                    flexDirection: 'column',
                                }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '10px' }}>
                                        <h2>Median Run Log</h2>
                                        <button onClick={() => setShowLog(false)}>Close</button>
                                    </div>
                                    <textarea
                                        readOnly
                                        value={medianLog}
                                        style={{
                                            flex: 1,
                                            backgroundColor: '#111',
                                            color: '#ddd',
                                            fontFamily: 'monospace',
                                            padding: '10px',
                                            border: '1px solid #444',
                                            resize: 'none'
                                        }}
                                    />
                                </div>
                            </div>
                        )}
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
            </semiPersistentContext.Provider>
        </div>
    )
}

export default Simulation