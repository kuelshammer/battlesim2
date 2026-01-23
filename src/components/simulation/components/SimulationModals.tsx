import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faTimes } from "@fortawesome/free-solid-svg-icons"
import EventLog from "../../combat/EventLog"
import AdjustmentPreview from "../AdjustmentPreview"
import { Encounter } from "@/model/model"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import styles from '../simulation.module.scss'

interface SimulationModalsProps {
    showLogModal: boolean
    selectedEncounterIndex: number | null
    selectedDecileIndex: number
    setSelectedDecileIndex: (index: number) => void
    setShowLogModal: (show: boolean) => void
    setSelectedEncounterIndex: (index: number | null) => void
    worker: ReturnType<typeof useSimulationWorker>
    combatantNames: Map<string, string>
    actionNames: Map<string, string>
    timeline: any[]
    applyOptimizedResult: () => void
}

export const SimulationModals = memo<SimulationModalsProps>(({
    showLogModal,
    selectedEncounterIndex,
    selectedDecileIndex,
    setSelectedDecileIndex,
    setShowLogModal,
    setSelectedEncounterIndex,
    worker,
    combatantNames,
    actionNames,
    timeline,
    applyOptimizedResult
}) => {
    return (
        <>
            {/* Event Log Modal */}
            {showLogModal && selectedEncounterIndex !== null && (
                <div className={`${styles.logModalOverlay} event-log-section`} data-testid="log-modal">
                    <div className={styles.logModal}>
                        <div className={styles.modalHeader}>
                            <div className={styles.modalHeaderTitle}>
                                <h3>Combat Log - Encounter {selectedEncounterIndex + 1}</h3>
                                <div className={styles.decileNav} data-testid="decile-nav">
                                    <button
                                        disabled={selectedDecileIndex === 0}
                                        onClick={() => setSelectedDecileIndex(selectedDecileIndex - 1)}
                                        className={styles.navBtn}
                                        data-testid="worse-run-btn"
                                    >
                                        &larr; Worse Run
                                    </button>
                                    <span className={styles.percentileLabel} data-testid="percentile-label">
                                        {selectedDecileIndex === 5 ? "50% (Median)" : `${selectedDecileIndex * 10 + (selectedDecileIndex < 5 ? 5 : -5)}% Run`}
                                    </span>
                                    <button
                                        disabled={selectedDecileIndex === 10}
                                        onClick={() => setSelectedDecileIndex(selectedDecileIndex + 1)}
                                        className={styles.navBtn}
                                        data-testid="better-run-btn"
                                    >
                                        Better Run &rarr;
                                    </button>
                                </div>
                            </div>
                            <button
                                onClick={() => setShowLogModal(false)}
                                className={styles.closeButton}
                                data-testid="close-log-btn"
                            >
                                <FontAwesomeIcon icon={faTimes} />
                            </button>
                        </div>
                        <div className={styles.logBody} data-testid="log-body">
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
        </>
    )
})

SimulationModals.displayName = 'SimulationModals'