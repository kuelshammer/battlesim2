import React, { FC, memo } from "react"
import { AggregateOutput } from "@/model/model"
import BattleCard from "./battleCard"
import styles from './quintileAnalysis.module.scss'

type PropType = {
    analysis: AggregateOutput | null
}

const QuintileAnalysis: FC<PropType> = memo(({ analysis }) => {
    if (!analysis) {
        return (
            <div className={styles.quintileAnalysis}>
                <h3>Quintile Analysis</h3>
                <p>Run simulations to see quintile analysis...</p>
            </div>
        )
    }

    return (
        <div className={styles.quintileAnalysis}>
            <h3>5-Timeline Dashboard: {analysis.scenario_name}</h3>
            <div className={styles.battleCards}>
                {analysis.quintiles.map((quintile) => (
                    <BattleCard key={quintile.quintile} quintile={quintile} />
                ))}
            </div>
            <div className={styles.analysisSummary}>
                <p>Based on {analysis.total_runs} simulation runs</p>
            </div>
        </div>
    )
})

export default QuintileAnalysis