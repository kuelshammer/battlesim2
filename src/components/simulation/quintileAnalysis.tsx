import React, { FC, memo } from "react"
import { AggregateOutput } from "@/model/model"
import BattleCard from "./battleCard"
import styles from './quintileAnalysis.module.scss'
import { useUIToggles, UIToggleType } from "@/model/uiToggleState"

type PropType = {
    analysis: AggregateOutput | null
}

const QuintileAnalysis: FC<PropType> = memo(({ analysis }) => {
    const { getToggleState } = useUIToggles()

    if (!analysis) {
        return (
            <div className={styles.quintileAnalysis}>
                <h3>Quintile Analysis</h3>
                <p>Run simulations to see quintile analysis...</p>
            </div>
        )
    }

    // Filter quintiles based on toggle states
    const visibleQuintiles = analysis.quintiles.filter(quintile => {
        const toggleId = `quintile-${quintile.quintile}` as UIToggleType
        return getToggleState(toggleId)
    })

    return (
        <div className={styles.quintileAnalysis}>
            <h3>5-Timeline Dashboard: {analysis.scenario_name}</h3>
            {visibleQuintiles.length === 0 ? (
                <div className={styles.emptyState}>
                    <p>All quintiles are hidden</p>
                    <p className={styles.emptyHint}>Use the UI controls to show specific quintiles</p>
                </div>
            ) : (
                <div className={styles.battleCards}>
                    {visibleQuintiles.map((quintile) => (
                        <BattleCard key={quintile.quintile} quintile={quintile} />
                    ))}
                </div>
            )}
            <div className={styles.analysisSummary}>
                <p>Based on {analysis.total_runs} simulation runs</p>
                {visibleQuintiles.length !== analysis.quintiles.length && (
                    <p className={styles.visibilityNote}>
                        Showing {visibleQuintiles.length} of {analysis.quintiles.length} quintiles
                    </p>
                )}
            </div>
        </div>
    )
})

export default QuintileAnalysis