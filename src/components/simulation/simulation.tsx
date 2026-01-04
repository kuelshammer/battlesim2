import React, { FC, useEffect, useState, useRef, memo, useMemo } from "react"
import { z } from "zod"
import { Creature, CreatureSchema, Encounter, EncounterSchema, TimelineEvent, TimelineEventSchema, SimulationResult, AggregateOutput, EncounterResult as EncounterResultType } from "@/model/model"
import { parseEventString, SimulationEvent } from "@/model/events"
import { clone, useStoredState } from "@/model/utils"
import styles from './simulation.module.scss'
import EncounterForm from "./encounterForm"
import EncounterResult from "./encounterResult"
import EventLog from "../combat/EventLog"
import OnboardingTour from "./OnboardingTour"
import PerformanceDashboard from "../debug/PerformanceDashboard"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faFolder, faPlus, faSave, faTrash, faEye, faTimes, faChartLine, faRedo, faBed, faMagicWandSparkles, faBolt, faBullseye, faQuestionCircle, faTachometerAlt } from "@fortawesome/free-solid-svg-icons"
import { v4 as uuid } from 'uuid'
import { semiPersistentContext } from "@/model/semiPersistentContext"
import AdventuringDayForm from "./adventuringDayForm"
import { getFinalAction } from "@/data/actions"
import { UIToggleProvider } from "@/model/uiToggleState"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import AdjustmentPreview from "./AdjustmentPreview"
import AssistantSummary from "./AssistantSummary"
import { calculatePacingData } from "./pacingUtils"
import PartyOverview from "./PartyOverview"
import PlayerGraphs from "./PlayerGraphs"
import { SkylineAnalysis, PlayerSlot } from "@/model/model"



type PropType = {
    // TODO
}

const emptyCombat: TimelineEvent = {
    type: 'combat',
    id: uuid(),
    monsters: [],
    monstersSurprised: false,
    playersSurprised: false,
    targetRole: 'Standard',
}

const FAST_ITERATIONS = 100;
const PRECISE_ITERATIONS = 2511;

// Sanitization helper: Fix duplicate IDs in players array
const sanitizePlayersParser = (parser: (data: unknown) => Creature[]) => (data: unknown) => {
    const parsed = parser(data);
    if (!parsed) return null;

    const playerIds = new Set<string>();
    const sanitized = parsed.map(p => {
        if (playerIds.has(p.id)) {
            return { ...p, id: uuid() }; // Generate new ID for duplicate
        }
        playerIds.add(p.id);
        return p;
    });

    return sanitized;
};

// Sanitization helper: Fix duplicate IDs in timeline monsters
const sanitizeTimelineParser = (parser: (data: unknown) => TimelineEvent[]) => (data: unknown) => {
    const parsed = parser(data);
    if (!parsed) return null;

    return parsed.map(item => {
        if (item.type !== 'combat') return item;

        const monsterIds = new Set<string>();
        const sanitizedMonsters = item.monsters.map(m => {
            if (monsterIds.has(m.id)) {
                return { ...m, id: uuid() }; // Generate new ID for duplicate
            }
            monsterIds.add(m.id);
            return m;
        });

        return { ...item, monsters: sanitizedMonsters };
    });
};

const Simulation: FC<PropType> = memo(({ }) => {
    const [players, setPlayers] = useStoredState<Creature[]>('players', [], sanitizePlayersParser(z.array(CreatureSchema).parse))
    const [timeline, setTimeline] = useStoredState<TimelineEvent[]>('timeline', [emptyCombat], sanitizeTimelineParser(z.array(TimelineEventSchema).parse))
    const [simulationResults, setSimulationResults] = useState<EncounterResultType[]>([])
    const [state, setState] = useState(new Map<string, any>())
    const [simulationEvents, setSimulationEvents] = useState<SimulationEvent[]>([])

    const [saving, setSaving] = useState(false)
    const [loading, setLoading] = useState(false)
    const [isEditing, setIsEditing] = useState(false)
    const [showLogModal, setShowLogModal] = useState(false)
    const [selectedEncounterIndex, setSelectedEncounterIndex] = useState<number | null>(null)
    const [selectedDecileIndex, setSelectedDecileIndex] = useState<number>(5) // Default to 50% Median
    const [runTour, setRunTour] = useState(false)
    const [showPerformanceDashboard, setShowPerformanceDashboard] = useState(false)

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
        
        // Add players - IDs are prefixed with 'p-' and numbered if count > 1
        players.forEach((p, group_idx) => {
            for (let i = 0; i < (p.count || 1); i++) {
                const id = `p-${group_idx}-${i}-${p.id}`
                const name = (p.count || 1) > 1 ? `${p.name} ${i + 1}` : p.name
                names.set(id, name)
            }
            // Fallback for base ID - VERY IMPORTANT for resolving WASM partySlots
            names.set(p.id, p.name)
        })

        // Add monsters - IDs include encounter index and are numbered if count > 1
        timeline.forEach((item, step_idx) => {
            if (item.type === 'combat') {
                item.monsters.forEach((m, group_idx) => {
                    for (let i = 0; i < (m.count || 1); i++) {
                        const id = `step${step_idx}-m-${group_idx}-${i}-${m.id}`
                        const name = (m.count || 1) > 1 ? `${m.name} ${i + 1}` : m.name
                        names.set(id, name)
                    }
                    // Fallback for base ID
                    names.set(m.id, m.name)
                })
            }
        })
        
        return names
    }, [players, timeline])

    const [canSave, setCanSave] = useState(false)
    
    const encounterWeights = useMemo(() => {
        const weights: number[] = [];
        timeline.forEach(item => {
            if (item.type === 'combat') {
                const role = item.targetRole || 'Standard';
                const weight = role === 'Skirmish' ? 1 : role === 'Standard' ? 2 : role === 'Elite' ? 3 : 4;
                weights.push(weight);
            }
        });
        return weights;
    }, [timeline]);

    const pacingData = useMemo(() => {
        return calculatePacingData(timeline, worker.analysis, encounterWeights);
    }, [worker.analysis, timeline, encounterWeights]);

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
            if (Array.isArray(timeline) && timeline.length > 0) {
                // Default to FAST mode for auto-simulation
                worker.runSimulation(players, timeline, FAST_ITERATIONS);
                setNeedsResimulation(false);
            }
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
            targetRole: 'Standard',
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
                    <div className={styles.header}>
                        <h1>BattleSim</h1>
                        <div className={styles.headerButtons}>
                            <button
                                className={styles.helpButton}
                                onClick={() => setRunTour(true)}
                                title="Start guided tour"
                                aria-label="Start guided tour"
                            >
                                <FontAwesomeIcon icon={faQuestionCircle} />
                                Help
                            </button>
                            <button
                                className={styles.helpButton}
                                onClick={() => setShowPerformanceDashboard(!showPerformanceDashboard)}
                                title="Toggle performance dashboard"
                                aria-label={`${showPerformanceDashboard ? 'Hide' : 'Show'} performance dashboard`}
                            >
                                <FontAwesomeIcon icon={faTachometerAlt} />
                                {showPerformanceDashboard ? 'Hide' : 'Perf'}
                            </button>
                        </div>
                    </div>

                    
                    {/* Backend Features Status Panel */}
                    <div className={`${styles.backendStatus} simulation-controls`} role="region" aria-label="Simulation Status">
                        <h4>🔧 Event-Driven Backend {worker.isRunning ? '(Processing...)' : 'Active'}</h4>
                        <div className={styles.statusItems} aria-live="polite" role="status">
                            <span>✅ ActionResolution Engine</span>
                            <span>✅ Event System</span>
                            <span>✅ Reaction Processing</span>
                            <span>✅ Effect Tracking</span>
                            <span>📊 Events: {simulationEvents.length}</span>
                        </div>
                        {worker.isRunning && (
                            <div 
                                className={styles.progressBar}
                                role="progressbar"
                                aria-valuenow={Math.round(worker.progress)}
                                aria-valuemin={0}
                                aria-valuemax={100}
                                aria-label="Simulation progress"
                            >
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

                        {worker.analysis && !worker.isRunning && (
                            <div className={styles.simulationMode}>
                                <div className={`${styles.modeIndicator} ${worker.currentIterations <= FAST_ITERATIONS ? styles.fast : styles.precise}`}>
                                    {worker.currentIterations <= FAST_ITERATIONS ? (
                                        <><FontAwesomeIcon icon={faBolt} /> Fast Sim ({worker.currentIterations} Runs)</>
                                    ) : (
                                        <><FontAwesomeIcon icon={faBullseye} /> Precise Sim ({worker.currentIterations} Runs)</>
                                    )}
                                </div>
                                {worker.currentIterations <= FAST_ITERATIONS && (
                                    <button
                                        className={styles.preciseButton}
                                        onClick={() => worker.runSimulation(players, timeline, FAST_ITERATIONS, undefined, true)}
                                        disabled={worker.isRunning}>
                                        Run Precise Sim
                                    </button>
                                )}
                            </div>
                        )}

                        {isEditing && <div className={styles.editingNotice}>⚠️ Simulation paused while editing</div>}
                        {worker.error && <div className={styles.errorNotice}>❌ Simulation Error: {worker.error}</div>}
                    </div>

                    <div className="encounter-builder-section player-form-section">
                        <EncounterForm
                            mode='player'
                            encounter={{ id: 'players', monsters: players, type: 'combat', targetRole: 'Standard' }}
                            onUpdate={(newValue) => setPlayers(newValue.monsters)}
                            onEditingChange={setIsEditing}>
                            <>
                                {!isEmptyResult ? (
                                    <button onClick={() => { setPlayers([]); setTimeline([emptyCombat]) }}>
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
                    </div>

                    {worker.analysis && pacingData && (
                        <>
                            <AssistantSummary 
                                pacingData={pacingData} 
                            />
                        </>
                    )}

                                        {timeline.map((item, index) => {
                                            // Find index within combat-only array for pacingData
                                            const combatIndex = timeline.slice(0, index).filter(i => i.type === 'combat').length;
                                            const totalWeight = encounterWeights.reduce((a, b) => a + b, 0);
                                            const targetPercent = (encounterWeights[combatIndex] / totalWeight) * 100;
                                            const actualPercent = pacingData?.actualCosts[combatIndex];
                                            const cumulativeDrift = pacingData?.cumulativeDrifts[combatIndex];

                                            return (
                                                <div className={item.type === 'combat' ? styles.encounter : styles.rest} key={index}>
                                                    {item.type === 'combat' ? (
                                                        <div className="monster-form-section">
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
                                                                worker.autoAdjustEncounter(players, item.monsters, timeline, index);
                                                            }}
                                                            autoAdjustDisabled={worker.isRunning}
                                                        />
                                                        </div>
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
                                                
                                                {(worker.analysis?.encounters?.[index] ? (
                                                    <EncounterResult
                                                        value={worker.analysis.encounters[index].globalMedian?.medianRunData || worker.analysis.encounters[index].deciles?.[4]?.medianRunData || simulationResults[index]}
                                                        analysis={worker.analysis.encounters[index]}
                                                        fullAnalysis={worker.analysis} 
                                                        playerNames={combatantNames}
                                                        isStale={isStale}
                                                        isPreliminary={worker.isRunning && worker.progress < 100}
                                                        targetPercent={item.type === 'combat' ? targetPercent : undefined}
                                                        actualPercent={item.type === 'combat' ? actualPercent : undefined}
                                                        cumulativeDrift={item.type === 'combat' ? cumulativeDrift : undefined}
                                                        isShortRest={item.type === 'shortRest'}
                                                    />
                                                ) : (item.type === 'combat' && simulationResults[index] ? (
                                                    <EncounterResult
                                                        value={simulationResults[index]}
                                                        analysis={null}
                                                        fullAnalysis={worker.analysis} 
                                                        playerNames={combatantNames}
                                                        isStale={isStale}
                                                        isPreliminary={worker.isRunning && worker.progress < 100}
                                                        targetPercent={targetPercent}
                                                        actualPercent={actualPercent}
                                                        cumulativeDrift={cumulativeDrift}
                                                    />
                                                ) : null))}
                                                
                                                {item.type === 'combat' && (
                                                    <div className={styles.buttonGroup}>
                                                        <button
                                                            onClick={() => {
                                                                console.log('Manually rerunning simulation...');
                                                                worker.runSimulation(players, timeline, PRECISE_ITERATIONS);
                                                                setIsStale(false);
                                                            }}
                                                            className={styles.rerunButton}
                                                            disabled={worker.isRunning}>
                                                            <FontAwesomeIcon icon={faRedo} spin={worker.isRunning} />
                                                            Rerun (Precise)
                                                        </button>
                                                        <button
                                                            onClick={() => {
                                                                setSelectedEncounterIndex(index);
                                                                setSelectedDecileIndex(5); // Reset to Median
                                                                setShowLogModal(true);
                                                            }}
                                                            className={styles.showLogButton}>
                                                            <FontAwesomeIcon icon={faEye} />
                                                            Show Log
                                                        </button>
                                                    </div>
                                                )}
                                            </div>
                                        )
                                    })}
                    
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

                    {/* Party Overview - Compact spectrogram */}
                    {worker.analysis?.overall?.skyline && worker.analysis?.partySlots && (
                        <PartyOverview
                            skyline={worker.analysis.overall.skyline as SkylineAnalysis}
                            partySlots={worker.analysis.partySlots as PlayerSlot[]}
                            playerNames={combatantNames}
                        />
                    )}

                    {/* Individual Player Statistics */}
                    {worker.analysis?.overall?.skyline && worker.analysis?.partySlots && (
                        <PlayerGraphs
                            skyline={worker.analysis.overall.skyline as SkylineAnalysis}
                            partySlots={worker.analysis.partySlots as PlayerSlot[]}
                            playerNames={combatantNames}
                        />
                    )}

                    {/* Event Log Modal */}
                    {showLogModal && selectedEncounterIndex !== null && (
                        <div className={`${styles.logModalOverlay} event-log-section`}>
                            <div className={styles.logModal}>
                                <div className={styles.modalHeader}>
                                    <div className={styles.modalHeaderTitle}>
                                        <h3>Combat Log - Encounter {selectedEncounterIndex + 1}</h3>
                                        <div className={styles.decileNav}>
                                            <button 
                                                disabled={selectedDecileIndex === 0}
                                                onClick={() => setSelectedDecileIndex(selectedDecileIndex - 1)}
                                                className={styles.navBtn}
                                            >
                                                &larr; Worse Run
                                            </button>
                                            <span className={styles.percentileLabel}>
                                                {selectedDecileIndex === 5 ? "50% (Median)" : `${selectedDecileIndex * 10 + (selectedDecileIndex < 5 ? 5 : -5)}% Run`}
                                            </span>
                                            <button 
                                                disabled={selectedDecileIndex === 10}
                                                onClick={() => setSelectedDecileIndex(selectedDecileIndex + 1)}
                                                className={styles.navBtn}
                                            >
                                                Better Run &rarr;
                                            </button>
                                        </div>
                                    </div>
                                    <button 
                                        onClick={() => setShowLogModal(false)}
                                        className={styles.closeButton}>
                                        <FontAwesomeIcon icon={faTimes} />
                                    </button>
                                </div>
                                <div className={styles.logBody}>
                                    <EventLog
                                        events={worker.analysis?.encounters[selectedEncounterIndex]?.decileLogs?.[selectedDecileIndex] || []}
                                        combatantNames={Object.fromEntries(combatantNames)}
                                        actionNames={Object.fromEntries(actionNames)}
                                        isModal={true}
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

                    {/* Adjustment Preview Modal */}
                    {worker.optimizedResult && selectedEncounterIndex !== null && timeline[selectedEncounterIndex].type === 'combat' && (
                        <div className="auto-balancer-section">
                            <AdjustmentPreview
                                originalMonsters={(timeline[selectedEncounterIndex] as Encounter).monsters}
                                adjustmentResult={worker.optimizedResult}
                                onApply={applyOptimizedResult}
                                onCancel={() => {
                                    worker.clearOptimizedResult();
                                    setSelectedEncounterIndex(null);
                                }}
                            />
                        </div>
                    )}
                </semiPersistentContext.Provider>

                {/* Onboarding Tour */}
                <OnboardingTour
                    forceRun={runTour}
                    onTourEnd={() => setRunTour(false)}
                />

                {/* Performance Dashboard */}
                <PerformanceDashboard
                    isVisible={showPerformanceDashboard}
                    onClose={() => setShowPerformanceDashboard(false)}
                />
            </div>
        </UIToggleProvider>
    )
})

export default Simulation