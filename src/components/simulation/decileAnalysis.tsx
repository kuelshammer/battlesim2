import React, { FC, memo, useState } from "react"
import { AggregateOutput } from "@/model/model"
import BattleCard from "./battleCard"
import styles from './decileAnalysis.module.scss'
import { useUIToggles, UIToggleType } from "@/model/uiToggleState"
import { EncounterRating } from "./AnalysisComponents"

type PropType = {
    analysis: AggregateOutput | null,
    isPreliminary?: boolean
}

const DecileAnalysis: FC<PropType> = memo(({ analysis, isPreliminary }) => {
    const { getToggleState } = useUIToggles()
    const [isExpanded, setIsExpanded] = useState(false)

    if (!analysis) {
        return (
            <div className={styles.decileAnalysis}>
                <h3>Decile Analysis</h3>
                <p>Run simulations to see decile analysis...</p>
            </div>
        )
    }

    const visibleDeciles = (analysis.deciles || []).filter(decile => {
        const toggleId = `quintile-${decile.decile}` as UIToggleType
        return getToggleState(toggleId)
    })

    return (
        <div className={styles.decileAnalysis}>
            <div className={styles.adventuringDayHeader}>
                <h2>📊 Adventuring Day Summary</h2>
            </div>
            
            <EncounterRating analysis={analysis} isPreliminary={isPreliminary} label="Day Rating" />
            {/* MedianPerformanceDisplay hidden per user preference */}

            <div className={styles.analysisHeader}>
                <button
                    className={styles.expandToggle}
                    onClick={() => setIsExpanded(!isExpanded)}
                >
                    {isExpanded ? '🔽' : '▶️'} {isExpanded ? 'Hide' : 'Show'} Full 10-Timeline Dashboard
                </button>
                <div className={styles.analysisSummary}>
                    <span>Based on {analysis.totalRuns} simulation runs</span>
                </div>
            </div>

            {isExpanded && (
                <div className={styles.analysisContent}>
                    <h3>10-Timeline Dashboard: {analysis.scenarioName}</h3>
                    {visibleDeciles.length === 0 ? (
                        <div className={styles.emptyState}>
                            <p>All deciles are hidden</p>
                            <p className={styles.emptyHint}>Use the UI controls to show specific deciles</p>
                        </div>
                    ) : (
                        <div className={styles.battleCards}>
                            {visibleDeciles.map((decile) => (
                                <BattleCard key={decile.decile} decile={decile} />
                            ))}
                        </div>
                    )}
                    {visibleDeciles.length !== (analysis.deciles || []).length && (
                        <div className={styles.analysisSummary}>
                            <p className={styles.visibilityNote}>
                                Showing {visibleDeciles.length} of {(analysis.deciles || []).length} deciles
                            </p>
                        </div>
                    )}
                </div>
            )}
        </div>
    )
})

export default DecileAnalysis