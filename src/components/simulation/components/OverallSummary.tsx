import React, { memo } from "react"
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome"
import { faChartLine } from "@fortawesome/free-solid-svg-icons"
import { VitalsDashboard, ValidationNotice } from "../AnalysisComponents"
import AssistantSummary from "../AssistantSummary"
import { calculatePacingData } from "../pacingUtils"
import PartyOverview from "../PartyOverview"
import PlayerGraphs from "../PlayerGraphs"
import HeartbeatGraph from "../HeartbeatGraph"
import { SkylineAnalysis, PlayerSlot } from "@/model/model"
import { useSimulationWorker } from "@/model/useSimulationWorker"
import styles from '../simulation.module.scss'

interface OverallSummaryProps {
    worker: ReturnType<typeof useSimulationWorker>
    timeline: any[]
    encounterWeights: number[]
    combatantNames: Map<string, string>
}

export const OverallSummary = memo<OverallSummaryProps>(({
    worker,
    timeline,
    encounterWeights,
    combatantNames
}) => {
    if (!worker.analysis?.overall?.skyline || !worker.analysis?.partySlots) return null

    const pacingData = calculatePacingData(timeline, worker.analysis, encounterWeights)

    return (
        <div className={styles.overallSummary} data-testid="overall-summary">
            <div className={styles.summaryDivider} data-testid="summary-divider">
                <div className={styles.dividerLine} />
                <h3 className={styles.summaryTitle} data-testid="summary-title">
                    <FontAwesomeIcon icon={faChartLine} /> Projected Day Outcome Summary
                </h3>
                <div className={styles.dividerLine} />
            </div>

            <VitalsDashboard analysis={worker.analysis.overall} isPreliminary={worker.isRunning} />

            <ValidationNotice analysis={worker.analysis.overall} isDaySummary={true} />

            {pacingData && <AssistantSummary
                pacingData={pacingData}
            />}

            {worker.analysis.overall.pacing && (
                <div className={styles.pacingHeader} data-testid="pacing-header">
                    <div className={styles.archetypeBadge} data-testid="pacing-archetype">
                        {worker.analysis.overall.pacing.archetype}
                    </div>
                    <div className={styles.directorScore} data-testid="director-score">
                        DIRECTOR'S SCORE: {Math.round(worker.analysis.overall.pacing.directorScore)}
                    </div>
                </div>
            )}

            <div className={styles.summaryGrid} data-testid="summary-grid">
                <HeartbeatGraph
                    encounters={worker.analysis.encounters}
                    className={styles.tensionArc}
                />

                <PartyOverview
                    skyline={worker.analysis.overall.skyline as SkylineAnalysis}
                    partySlots={worker.analysis.partySlots as PlayerSlot[]}
                    playerNames={combatantNames}
                    className="overall-party-overview"
                />
            </div>

            <PlayerGraphs
                skyline={worker.analysis.overall.skyline as SkylineAnalysis}
                partySlots={worker.analysis.partySlots as PlayerSlot[]}
                playerNames={combatantNames}
            />
        </div>
    )
})

OverallSummary.displayName = 'OverallSummary'