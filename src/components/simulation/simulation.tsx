import React, { FC, useEffect, useState, useRef, memo, useMemo } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, SimulationResult, AggregateOutput } from "@/model/model"
import { parseEventString, SimulationEvent } from "@/model/events"
import { clone, useStoredState } from "@/model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import EventLog from "../combat/EventLog"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash, faEye, faTimes, faChartLine, faRedo } from "@fortawesome/free-solid-svg-icons"
import { semiPersistentContext } from "@/model/semiPersistentContext"
import AdventuringDayForm from "./adventuringDayForm"
import { getFinalAction } from "@/data/actions"
import QuintileAnalysis from "./quintileAnalysis"
import { UIToggleProvider } from "@/model/uiToggleState"
import { useSimulationWorker } from "@/model/useSimulationWorker"



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
    const [simulationResults, setSimulationResults] = useState<SimulationResult>([])
    const [state, setState] = useState(new Map<string, any>())
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])

    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [isEditing, setIsEditing] = useState(false)
    const [showLogModal, setShowLogModal] = useState(false)
    const [showQuintileModal, setShowQuintileModal] = useState(false)
    const [selectedEncounterIndex, setSelectedEncounterIndex] = useState<number | null>(null)
    
    // Web Worker Simulation
    const worker = useSimulationWorker();
    const [needsResimulation, setNeedsResimulation] = useState(false);
    const [isStale, setIsStale] = useState(false);
    const [autoSimulate, setAutoSimulate] = useState(true);
    const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
    
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

    const [canSave, setCanSave] = useState(false)
    
    useEffect(() => {
        setCanSave(!isEmptyResult)
    }, [isEmptyResult])

    // Detect changes that need resimulation with debounce
    useEffect(() => {
        if (!autoSimulate) return;

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
    }, [players, encounters, autoSimulate]);

    // Trigger simulation when not editing and needs resimulation
    useEffect(() => {
        if (!autoSimulate) return;
        
        if (!isEditing && !saving && !loading && needsResimulation && !worker.isRunning) {
            console.log('Triggering background simulation...');
            worker.runSimulation(players, encounters, 2510);
            setNeedsResimulation(false);
        }
    }, [isEditing, saving, loading, needsResimulation, worker.isRunning, players, encounters, worker, autoSimulate]);

    // Update display results when worker finishes
    useEffect(() => {
        if (!worker.results || worker.results.length === 0) return;

        const results = worker.results;
        // The backend now only returns 5 representative runs:
        // [Disaster, Struggle, Typical, Heroic, Legend]
        // Index 2 is the 'Typical' (Median) run.
        const index = results.length >= 3 ? 2 : 0;
        const selectedRun = results[index];

        setSimulationResults(selectedRun);

        // Update events from the first run returned by the worker
        if (worker.events) {
            // events in worker are already structured objects, not strings
            // But let's check if they need parsing
            setSimulationEvents(worker.events as SimulationEvent[]);
        }

        // Clear stale state when new results are available
        setIsStale(false);
    }, [worker.results, worker.events]);


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
        <UIToggleProvider>
            <div className={styles.simulation}>
                <semiPersistentContext.Provider value={{ state, setState }}>
                    <h1 className={styles.header}>BattleSim</h1>

                    

                    {/* Backend Features Status Panel */}
                    <div className={styles.backendStatus}>
                        <h4>🔧 Event-Driven Backend {worker.isRunning ? '(Processing...)' : 'Active'}</h4>
                        <div className={styles.statusItems}>
                            <span>✅ ActionResolution Engine</span>
                            <span>✅ Event System</span>
                            <span>✅ Reaction Processing</span>
                            <span>✅ Effect Tracking</span>
                            <span>📊 Events: {simulationEvents.length}</span>
                        </div>
                        {worker.isRunning && (
                            <div className={styles.progressBar}>
                                <div 
                                    className={styles.progressFill} 
                                    style={{ width: `${worker.progress}%` }}
                                />
                                <span className={styles.progressText}>
                                    Simulating {worker.completed} / {worker.total} runs ({Math.round(worker.progress)}%)
                                </span>
                            </div>
                        )}
                        <div className={styles.autoSimulateToggle}>
                            <label className={styles.toggleLabel}>
                                <input
                                    type="checkbox"
                                    checked={autoSimulate}
                                    onChange={(e) => setAutoSimulate(e.target.checked)}
                                    className={styles.toggleInput}
                                />
                                <span className={styles.toggleSwitch}></span>
                                <span className={styles.toggleText}>
                                    Auto-Simulate Changes
                                </span>
                            </label>
                        </div>
                        {isEditing && <div className={styles.editingNotice}>⚠️ Simulation paused while editing</div>}
                    </div>

                    <EncounterForm
                        mode='player'
                        encounter={{ monsters: players }}
                        onUpdate={(newValue) => setPlayers(newValue.monsters)}
                        onEditingChange={setIsEditing}>
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
                                onEditingChange={setIsEditing}
                            />
                            {(worker.analysis?.encounters?.[index] ? (
                                <EncounterResult 
                                    value={worker.analysis.encounters[index].quintiles[2].medianRunData || simulationResults[index]} 
                                    analysis={worker.analysis.encounters[index]} 
                                    isStale={isStale}
                                    isPreliminary={worker.isRunning && worker.progress < 100}
                                />
                            ) : (!simulationResults[index] ? null : (
                                <EncounterResult 
                                    value={simulationResults[index]} 
                                    analysis={null} 
                                    isStale={isStale}
                                    isPreliminary={worker.isRunning && worker.progress < 100}
                                />
                            )))}
                            <div className={styles.buttonGroup}>
                                <button
                                    onClick={() => {
                                        console.log('Manually rerunning simulation...');
                                        worker.runSimulation(players, encounters, 2510);
                                        setIsStale(false);
                                    }}
                                    className={styles.rerunButton}
                                    disabled={worker.isRunning}>
                                    <FontAwesomeIcon icon={faRedo} spin={worker.isRunning} />
                                    Rerun
                                </button>
                                <button
                                    onClick={() => {
                                        setSelectedEncounterIndex(index);
                                        setShowLogModal(true);
                                    }}
                                    className={styles.showLogButton}>
                                    <FontAwesomeIcon icon={faEye} />
                                    Show Log
                                </button>
                                <button
                                    onClick={() => {
                                        setSelectedEncounterIndex(index);
                                        setShowQuintileModal(true);
                                    }}
                                    className={styles.showQuintileButton}>
                                    <FontAwesomeIcon icon={faChartLine} />
                                    Show Quintile Analysis
                                </button>
                            </div>
                        </div>
                    ))}

<button
                        onClick={createEncounter}
                        className={styles.addEncounterBtn}>
                        <FontAwesomeIcon icon={faPlus} />
                        Add Encounter
                    </button>

{/* Quintile Analysis Display */}
                    {worker.analysis && (
                        <QuintileAnalysis analysis={worker.analysis.overall} />
                    )}

                    {/* Event Log Modal */}
                    {showLogModal && selectedEncounterIndex !== null && (
                        <div className={styles.logModalOverlay}>
                            <div className={styles.logModal}>
                                <div className={styles.modalHeader}>
                                    <h3>Combat Log - Encounter {selectedEncounterIndex + 1}</h3>
                                    <button 
                                        onClick={() => setShowLogModal(false)}
                                        className={styles.closeButton}>
                                        <FontAwesomeIcon icon={faTimes} />
                                    </button>
                                </div>
                                <div className={styles.logBody}>
                                    <EventLog
                                        events={simulationEvents}
                                        combatantNames={Object.fromEntries(combatantNames)}
                                        actionNames={Object.fromEntries(actionNames)}
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Adventuring Day Editor (Save/Load) */}
                    {(saving || loading) && (
                        <AdventuringDayForm
                            currentPlayers={players}
                            currentEncounters={encounters}
                            onCancel={() => { setSaving(false); setLoading(false); }}
                            onApplyChanges={(newPlayers, newEncounters) => {
                                setPlayers(newPlayers);
                                setEncounters(newEncounters);
                                setSaving(false);
                                setLoading(false);
                            }}
                            onEditingChange={setIsEditing}
                        />
                    )}

                    {/* Quintile Analysis Modal */}
                    {showQuintileModal && selectedEncounterIndex !== null && (
                        <div className={styles.logModalOverlay}>
                            <div className={styles.logModal}>
                                <div className={styles.modalHeader}>
                                    <h3>Quintile Analysis - Encounter {selectedEncounterIndex + 1}</h3>
                                    <button 
                                        onClick={() => setShowQuintileModal(false)}
                                        className={styles.closeButton}>
                                        <FontAwesomeIcon icon={faTimes} />
                                    </button>
                                </div>
                                <div className={styles.logBody}>
                                    <QuintileAnalysis analysis={worker.analysis?.encounters?.[selectedEncounterIndex] ?? null} />
                                </div>
                            </div>
                        </div>
                    )}
                </semiPersistentContext.Provider>
            </div>
        </UIToggleProvider>
    )
})

export default Simulation