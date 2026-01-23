import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faQuestionCircle, faTachometerAlt } from "@fortawesome/free-solid-svg-icons"
import styles from '../simulation.module.scss'

interface SimulationHeaderProps {
    runTour: boolean
    setRunTour: (run: boolean) => void
    showPerformanceDashboard: boolean
    setShowPerformanceDashboard: (show: boolean) => void
}

export const SimulationHeader = memo<SimulationHeaderProps>(({
    runTour,
    setRunTour,
    showPerformanceDashboard,
    setShowPerformanceDashboard
}) => {
    return (
        <div className={styles.header}>
            <h1>BattleSim</h1>
            <div className={styles.headerButtons}>
                <button
                    className={styles.helpButton}
                    onClick={() => setRunTour(true)}
                    title="Start guided tour"
                    aria-label="Start guided tour"
                    data-testid="help-btn"
                >
                    <FontAwesomeIcon icon={faQuestionCircle} />
                    Help
                </button>
                <button
                    className={styles.helpButton}
                    onClick={() => setShowPerformanceDashboard(!showPerformanceDashboard)}
                    title="Toggle performance dashboard"
                    aria-label={`${showPerformanceDashboard ? 'Hide' : 'Show'} performance dashboard`}
                    data-testid="perf-btn"
                >
                    <FontAwesomeIcon icon={faTachometerAlt} />
                    {showPerformanceDashboard ? 'Hide' : 'Perf'}
                </button>
            </div>
        </div>
    )
})

SimulationHeader.displayName = 'SimulationHeader'