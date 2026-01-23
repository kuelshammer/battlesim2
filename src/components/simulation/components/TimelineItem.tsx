import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBed, faTrash, faEye } from "@fortawesome/free-solid-svg-icons"
import EncounterForm from "../encounterForm"
import EncounterResult from "../encounterResult"
import { TimelineEvent } from "@/model/model"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import { calculatePacingData } from "../pacingUtils"
import styles from '../simulation.module.scss'

interface TimelineItemProps {
    item: TimelineEvent
    index: number
    timeline: TimelineEvent[]
    players: any[]
    combatantNames: Map<string, string>
    isStale: boolean
    encounterWeights: number[]
    worker: ReturnType<typeof useSimulationWorker>
    simulationResults: any[]
    setIsEditing: (isEditing: boolean) => void
    updateTimelineItem: (index: number, newValue: TimelineEvent) => void
    deleteTimelineItem: (index: number) => void
    swapTimelineItems: (index1: number, index2: number) => void
    setSelectedEncounterIndex: (index: number | null) => void
    setSelectedDecileIndex: (index: number) => void
    setShowLogModal: (show: boolean) => void
}

export const TimelineItem = memo<TimelineItemProps>(({
    item,
    index,
    timeline,
    players,
    combatantNames,
    isStale,
    encounterWeights,
    worker,
    simulationResults,
    setIsEditing,
    updateTimelineItem,
    deleteTimelineItem,
    swapTimelineItems,
    setSelectedEncounterIndex,
    setSelectedDecileIndex,
    setShowLogModal
}) => {
    // Find index within combat-only array for pacingData
    const combatIndex = timeline.slice(0, index).filter(i => i.type === 'combat').length;
    const totalWeight = encounterWeights.reduce((a, b) => a + b, 0);
    const targetPercent = (encounterWeights[combatIndex] / totalWeight) * 100;
    const pacingData = calculatePacingData(timeline, worker.analysis, encounterWeights);
    const actualPercent = pacingData?.actualCosts[combatIndex];
    const cumulativeDrift = pacingData?.cumulativeDrifts[combatIndex];

    return (
        <div className={item.type === 'combat' ? styles.encounter : styles.rest} key={index} data-testid={item.type === 'combat' ? `encounter-${index}` : `short-rest-${index}`}>
            {item.type === 'combat' ? (
                <div className="monster-form-section" data-testid="monster-section">
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
                <div className={styles.restCard} data-testid="short-rest-card">
                    <div className={styles.restHeader}>
                        <h3><FontAwesomeIcon icon={faBed} /> Short Rest</h3>
                        <div className={styles.restControls} data-testid="rest-controls">
                            <button onClick={() => swapTimelineItems(index, index - 1)} disabled={index === 0} data-testid="move-rest-up-btn">↑</button>
                            <button onClick={() => swapTimelineItems(index, index + 1)} disabled={index === timeline.length - 1} data-testid="move-rest-down-btn">↓</button>
                            <button onClick={() => deleteTimelineItem(index)} className={styles.deleteBtn} data-testid="delete-rest-btn"><FontAwesomeIcon icon={faTrash} /></button>
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
                    targetRole={item.type === 'combat' ? item.targetRole : undefined}
                />                                                    ) : (item.type === 'combat' && simulationResults[index] ? (
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
                    targetRole={item.targetRole}
                />                                                    ) : null))}

            {item.type === 'combat' && (
                <div className={styles.buttonGroup}>
                    <button
                        onClick={() => {
                            setSelectedEncounterIndex(index);
                            setSelectedDecileIndex(5); // Reset to Median
                            setShowLogModal(true);
                        }}
                        className={styles.showLogButton}
                        data-testid="show-log-btn"
                    >
                        <FontAwesomeIcon icon={faEye} />
                        Show Log
                    </button>
                </div>
            )}
        </div>
    )
})

TimelineItem.displayName = 'TimelineItem'