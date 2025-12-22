import React, { FC, useEffect, useState, useRef, memo, useMemo } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, TimelineEvent, TimelineEventSchema, SimulationResult, AggregateOutput, EncounterResult as EncounterResultType } from "@/model/model"
import { parseEventString, SimulationEvent } from "@/model/events"
import { clone, useStoredState } from "@/model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import EventLog from "../combat/EventLog"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash, faEye, faTimes, faChartLine, faRedo, faBed, faMagicWandSparkles } from "@fortawesome/free-solid-svg-icons"
import { v4 as uuid } from 'uuid'
import { semiPersistentContext } from "@/model/semiPersistentContext"
import AdventuringDayForm from "./adventuringDayForm"
import { getFinalAction } from "@/data/actions"
import DecileAnalysis from "./decileAnalysis"
import { UIToggleProvider } from "@/model/uiToggleState"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import AdjustmentPreview from "./AdjustmentPreview"



type PropType = {
    // TODO
}

const emptyCombat: TimelineEvent = {
    type: 'combat',
    id: uuid(),
    monsters: [],
    monstersSurprised: false,
    playersSurprised: false,
}

const Simulation: FC<PropType> = memo(({ }) => {
    const [players, setPlayers] = useStoredState<Creature[]>('players', [], z.array(CreatureSchema).parse)
    const [timeline, setTimeline] = useStoredState<TimelineEvent[]>('timeline', [emptyCombat], z.array(TimelineEventSchema).parse)
    const [simulationResults, setSimulationResults] = useState<EncounterResultType[]>([])
    const [state, setState] = useState(new Map<string, any>())
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])

    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [isEditing, setIsEditing] = useState(false)
    const [showLogModal, setShowLogModal] = useState(false)
    const [showDecileModal, setShowDecileModal] = useState(false)
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
        const hasMonsters = !!timeline.find(item => item.type === 'combat' && !!item.monsters.length)
        return !hasPlayers && !hasMonsters
    }, [players.length, timeline])

    // Memoize combatant names map
    const combatantNames = useMemo(() => {
        const names = new Map<string, string>()
        players.forEach(p => names.set(p.id, p.name))
        timeline.forEach(item => {
            if (item.type === 'combat') {
                item.monsters.forEach(m => names.set(m.id, m.name))
            }
        })
        return names
    }, [players, timeline])

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
    }, [players, timeline, autoSimulate]);

    // Trigger simulation when not editing and needs resimulation
    useEffect(() => {
        if (!autoSimulate) return;
        
        if (!isEditing && !saving && !loading && needsResimulation && !worker.isRunning) {
            worker.runSimulation(players, timeline, 2511);
            setNeedsResimulation(false);
        }
    }, [isEditing, saving, loading, needsResimulation, worker.isRunning, players, timeline, worker, autoSimulate]);

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


    function createCombat() {
        setTimeline([...timeline, {
            type: 'combat',
            id: uuid(), // Ensure new items have a unique ID
            monsters: [],
            monstersSurprised: false,
            playersSurprised: false,
        }])
    }

    function createShortRest() {
        setTimeline([...timeline, {
            type: 'shortRest',
            id: uuid(),
        }])
    }

    function updateTimelineItem(index: number, newValue: TimelineEvent) {
        const timelineClone = clone(timeline)
        timelineClone[index] = newValue
        setTimeline(timelineClone)
    }

    function deleteTimelineItem(index: number) {
        if (timeline.length <= 1) return // Must have at least one item
        const timelineClone = clone(timeline)
        timelineClone.splice(index, 1)
        setTimeline(timelineClone)
    }

    function swapTimelineItems(index1: number, index2: number) {
        const timelineClone = clone(timeline)
        const tmp = timelineClone[index1]
        timelineClone[index1] = timelineClone[index2]
        timelineClone[index2] = tmp
        setTimeline(timelineClone)
    }

    function applyOptimizedResult() {
        if (selectedEncounterIndex === null || !worker.optimizedResult) return;
        
        const timelineClone = clone(timeline);
        const item = timelineClone[selectedEncounterIndex];
        
        if (item.type === 'combat') {
            item.monsters = worker.optimizedResult.monsters;
            setTimeline(timelineClone);
        }
        
        worker.clearOptimizedResult();
        setSelectedEncounterIndex(null);
    }

    // Memoize action names map
    const actionNames = useMemo(() => {
        const names = new Map<string, string>()
        players.forEach(p => p.actions.forEach(a => {
            const name = a.type === 'template' ? a.templateOptions.templateName : a.name
            names.set(a.id, name)
        }))
        timeline.forEach(item => {
            if (item.type === 'combat') {
                item.monsters.forEach(m => m.actions.forEach(a => {
                    const name = a.type === 'template' ? a.templateOptions.templateName : a.name
                    names.set(a.id, name)
                }))
            }
        })
        return names
    }, [players, timeline])

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
                        {worker.error && <div className={styles.errorNotice}>❌ Simulation Error: {worker.error}</div>}
                    </div>

                    <EncounterForm
                        mode='player'
                        encounter={{ id: 'players', monsters: players }}
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

                                        {timeline.map((item, index) => (
                                            <div className={item.type === 'combat' ? styles.encounter : styles.rest} key={index}>
                                                {item.type === 'combat' ? (
                                                    <EncounterForm
                                                        mode='monster'
                                                        encounter={item}
                                                        onUpdate={(newValue) => updateTimelineItem(index, newValue)}
                                                        onDelete={(index > 0) ? () => deleteTimelineItem(index) : undefined}
                                                        onMoveUp={(!!timeline.length && !!index) ? () => swapTimelineItems(index, index - 1) : undefined}
                                                        onMoveDown={(!!timeline.length && (index < timeline.length - 1)) ? () => swapTimelineItems(index, index + 1) : undefined}
                                                        onEditingChange={setIsEditing}
                                                        onAutoAdjust={() => {
                                                            setSelectedEncounterIndex(index);
                                                            worker.autoAdjustEncounter(players, item.monsters);
                                                        }}
                                                        autoAdjustDisabled={worker.isRunning}
                                                    />
                                                ) : (
                                                    <div className={styles.restCard}>
                                                        <div className={styles.restHeader}>
                                                            <h3><FontAwesomeIcon icon={faBed} /> Short Rest</h3>
                                                            <div className={styles.restControls}>
                                                                <button onClick={() => swapTimelineItems(index, index - 1)} disabled={index === 0}>↑</button>
                                                                <button onClick={() => swapTimelineItems(index, index + 1)} disabled={index === timeline.length - 1}>↓</button>
                                                                <button onClick={() => deleteTimelineItem(index)} className={styles.deleteBtn}><FontAwesomeIcon icon={faTrash} /></button>
                                                            </div>
                                                        </div>
                                                        <div className={styles.restBody}>
                                                            Characters spend Hit Dice to recover HP and reset "Short Rest" resources.
                                                        </div>
                                                    </div>
                                                )}
                                                
                                                {item.type === 'combat' && (worker.analysis?.encounters?.[index] ? (
                                                    <EncounterResult 
                                                        value={worker.analysis.encounters[index].globalMedian?.medianRunData || worker.analysis.encounters[index].deciles?.[4]?.medianRunData || simulationResults[index]} 
                                                        analysis={worker.analysis.encounters[index]} 
                                                        isStale={isStale}
                                                        isPreliminary={worker.isRunning && worker.progress < 100}
                                                    />
                                                ) : (simulationResults[index] ? (
                                                    <EncounterResult 
                                                        value={simulationResults[index]} 
                                                        analysis={null} 
                                                        isStale={isStale}
                                                        isPreliminary={worker.isRunning && worker.progress < 100}
                                                    />
                                                ) : null))}
                                                
                                                {item.type === 'combat' && (
                                                    <div className={styles.buttonGroup}>
                                                        <button
                                                            onClick={() => {
                                                                console.log('Manually rerunning simulation...');
                                                                worker.runSimulation(players, timeline, 2511);
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
                                                                setShowDecileModal(true);
                                                            }}
                                                            className={styles.showDecileButton}>
                                                            <FontAwesomeIcon icon={faChartLine} />
                                                            Show Decile Analysis
                                                        </button>
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                    
                                        <div className={styles.addButtons}>
                                            <button
                                                onClick={createCombat}
                                                className={styles.addEncounterBtn}>
                                                <FontAwesomeIcon icon={faPlus} />
                                                Add Combat
                                            </button>
                                            <button
                                                onClick={createShortRest}
                                                className={`${styles.addEncounterBtn} ${styles.restBtn}`}>
                                                <FontAwesomeIcon icon={faBed} />
                                                Add Short Rest
                                            </button>
                                        </div>
{/* Decile Analysis Display */}
                    {worker.analysis && (
                        <DecileAnalysis analysis={worker.analysis.overall} />
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
                            currentTimeline={timeline}
                            onCancel={() => { setSaving(false); setLoading(false); }}
                            onApplyChanges={(newPlayers, newTimeline) => {
                                setPlayers(newPlayers);
                                setTimeline(newTimeline);
                                setSaving(false);
                                setLoading(false);
                            }}
                            onEditingChange={setIsEditing}
                        />
                    )}

                    {/* Decile Analysis Modal */}
                    {showDecileModal && selectedEncounterIndex !== null && (
                        <div className={styles.logModalOverlay}>
                            <div className={styles.logModal}>
                                <div className={styles.modalHeader}>
                                    <h3>Decile Analysis - Encounter {selectedEncounterIndex + 1}</h3>
                                    <button 
                                        onClick={() => setShowDecileModal(false)}
                                        className={styles.closeButton}>
                                        <FontAwesomeIcon icon={faTimes} />
                                    </button>
                                </div>
                                <div className={styles.logBody}>
                                    <DecileAnalysis analysis={worker.analysis?.encounters?.[selectedEncounterIndex] ?? null} />
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Adjustment Preview Modal */}
                    {worker.optimizedResult && selectedEncounterIndex !== null && timeline[selectedEncounterIndex].type === 'combat' && (
                        <AdjustmentPreview
                            originalMonsters={(timeline[selectedEncounterIndex] as Encounter).monsters}
                            adjustmentResult={worker.optimizedResult}
                            onApply={applyOptimizedResult}
                            onCancel={() => {
                                worker.clearOptimizedResult();
                                setSelectedEncounterIndex(null);
                            }}
                        />
                    )}
                </semiPersistentContext.Provider>
            </div>
        </UIToggleProvider>
    )
})

export default Simulation