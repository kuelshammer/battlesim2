import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faBolt } from "@fortawesome/free-solid-svg-icons"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import styles from '../simulation.module.scss'

interface BackendStatusPanelProps {
    worker: ReturnType<typeof useSimulationWorker>
    highPrecision: boolean
    setHighPrecision: (highPrecision: boolean) => void
    isEditing: boolean
    simulationEvents: any[]
    players: any[]
    timeline: any[]
}

export const BackendStatusPanel = memo<BackendStatusPanelProps>(({
    worker,
    highPrecision,
    setHighPrecision,
    isEditing,
    simulationEvents,
    players,
    timeline
}) => {
    return (
        <div className={`${styles.backendStatus} simulation-controls`} role="region" aria-label="Simulation Status" data-testid="simulation-status">
            <h4>🔧 Event-Driven Backend {worker.isRunning ? '(Processing...)' : 'Active'}</h4>
            <div className={styles.statusItems} aria-live="polite" role="status" data-testid="backend-status-items">
                <span data-testid="status-action-engine">✅ ActionResolution Engine</span>
                <span data-testid="status-event-system">✅ Event System</span>
                <span data-testid="status-reaction-processing">✅ Reaction Processing</span>
                <span data-testid="status-effect-tracking">✅ Effect Tracking</span>
                <span data-testid="event-count">📊 Events: {simulationEvents.length}</span>
            </div>
            {worker.isRunning && (
                <div
                    className={styles.progressBar}
                    role="progressbar"
                    aria-valuenow={Math.round(worker.progress)}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-label="Simulation progress"
                    data-testid="simulation-loading"
                >
                    <div
                        className={styles.progressFill}
                        style={{ width: `${worker.progress}%` }}
                    />
                    <span className={styles.progressText}>
                        Refining Accuracy (K={worker.kFactor}/{highPrecision ? 51 : 3})
                    </span>
                </div>
            )}
            <div className={styles.autoSimulateToggle}>
                <label className={styles.toggleLabel} data-testid="high-precision-toggle">
                    <input
                        type="checkbox"
                        checked={highPrecision}
                        onChange={(e) => setHighPrecision(e.target.checked)}
                        className={styles.toggleInput}
                    />
                    <span className={styles.toggleSwitch}></span>
                    <span className={styles.toggleText}>
                        High Precision Mode
                    </span>
                </label>
            </div>

            {worker.analysis && !worker.isRunning && worker.kFactor < (highPrecision ? 51 : 3) && (
                <div className={styles.simulationMode}>
                    <div className={styles.pausedIndicator}>
                        <FontAwesomeIcon icon={faBolt} /> Refinement Paused
                    </div>
                                <button
                                    className={styles.preciseButton}
                                    onClick={() => worker.runSimulation(players, timeline, highPrecision ? 51 : 3)}
                                    disabled={!worker.analysis}
                                >
                                    Resume Refinement
                                </button>
                </div>
            )}

            {isEditing && <div className={styles.editingNotice}>⚠️ Simulation paused while editing</div>}
            {worker.error && <div className={styles.errorNotice}>❌ Simulation Error: {worker.error}</div>}
        </div>
    )
})

BackendStatusPanel.displayName = 'BackendStatusPanel'