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

    useEffect(() => {
        const loadWasmModule = async () => {
            if (typeof window !== 'undefined' && window.electronAPI) {
                try {
                    const wasmBytes = await window.electronAPI.loadWasm('simulation_wasm_bg.wasm');
                    const wasmModule = await import('simulation-wasm');
                    await wasmModule.default(await WebAssembly.compile(wasmBytes));
                    setWasm(wasmModule);
                } catch (error) {
                    console.error('Failed to load WASM in Electron:', error);
                }
            } else {
                // Fallback for web environment or when electronAPI is not available
                import('simulation-wasm').then(async (module) => {
                    await module.default('./simulation_wasm_bg.wasm');
                    setWasm(module);
                }).catch(error => {
                    console.error('Failed to load WASM in web environment:', error);
                });
            }
        };
        loadWasmModule();
    }, []);


    useEffect(() => {
        if (!wasm) return

        // Map luck (0-1) to index (0-4)
        const index = Math.min(4, Math.floor(luck * 5))

        if (allResults.length === 0) {
            // Run simulation if not cached
            // We need to resolve templates before sending to WASM?
            // The WASM expects Creature structs.
            // Assuming the frontend objects match the WASM structs closely enough (thanks to serde).
            // But we need to make sure we don't send extra fields that might break serde if strict?
            // Serde is usually forgiving with extra fields if not configured otherwise.

            // However, we need to handle the fact that `run_simulation_wasm` is synchronous in Rust but might take time.
            // Ideally we should use a worker, but for now let's try direct call.

            try {
                // We need to pass clean objects.
                // The `players` and `encounters` might contain Zod parsed objects which are fine.
                // But we need to ensure `getFinalAction` is applied?
                // The Rust code expects `Action` enum which matches `ActionSchema`.
                // If `ActionSchema` includes templates, Rust needs to handle them.
                // I removed Template support in Rust for now.
                // So I should resolve templates in JS before sending to Rust.

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

                const results = wasm.run_simulation_wasm(cleanPlayers, cleanEncounters, 1005) as SimulationResult[]
                setAllResults(results)

                // Aggregate results based on luck slice
                // Luck 0 -> 0-20% slice
                // Luck 0.5 -> 40-60% slice
                // Luck 1 -> 80-100% slice

                // We need to pass the slice to aggregate_simulation_results
                // But run_simulation_wasm returns sorted results? 
                // Yes, run_monte_carlo sorts by score.

                // Calculate slice indices
                const total = results.length
                const sliceSize = Math.floor(total / 5) // 20% slice
                const start = Math.min(total - sliceSize, Math.floor(luck * (total - sliceSize))) // Ensure start index is valid
                const end = start + sliceSize

                const slice = results.slice(start, end)

                // Aggregate
                const aggregated = wasm.aggregate_simulation_results(slice) as any // Returns Vec<Round> which is SimulationResult[0] (EncounterResult) rounds?
                // Wait, aggregate_results returns Vec<Round>.
                // SimulationResult is Vec<EncounterResult>.
                // EncounterResult has rounds: Vec<Round>.
                // So we need to wrap it in EncounterResult structure.

                // Construct synthetic EncounterResult
                const syntheticResult = [{
                    rounds: aggregated,
                    stats: new Map(),
                    team1: [], // Not used by EncounterResult component? It uses rounds.
                    team2: []
                }]

                setSimulationResults(syntheticResult)
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
                    team1: [],
                    team2: []
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