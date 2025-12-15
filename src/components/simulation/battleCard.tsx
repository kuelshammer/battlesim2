import React, { FC, memo } from "react"
import { QuintileStats, CombatantVisualization } from "../../model/model"
import styles from './battleCard.module.scss'

type PropType = {
    quintile: QuintileStats
}

const BattleCard: FC<PropType> = memo(({ quintile }) => {
    const getHpBarColor = (hpPercentage: number, isDead: boolean): string => {
        if (isDead) return styles.dead
        if (hpPercentage <= 20) return styles.danger
        if (hpPercentage <= 50) return styles.bloodied
        return styles.healthy
    }

    const getHpBarFill = (hpPercentage: number): string => {
        if (hpPercentage <= 0) return '░░░░░░░░░░'
        if (hpPercentage <= 10) return '█░░░░░░░░░'
        if (hpPercentage <= 20) return '██░░░░░░░░'
        if (hpPercentage <= 30) return '███░░░░░░░'
        if (hpPercentage <= 40) return '████░░░░░░'
        if (hpPercentage <= 50) return '█████░░░░░'
        if (hpPercentage <= 60) return '██████░░░░'
        if (hpPercentage <= 70) return '███████░░░'
        if (hpPercentage <= 80) return '████████░░'
        if (hpPercentage <= 90) return '█████████░'
        return '██████████'
    }

    const getOutcomeIcon = (winRate: number): string => {
        if (winRate < 100) return '💀'
        if (winRate === 100 && quintile.median_survivors < quintile.party_size) return '⚠️'
        return '✅'
    }

    const getOutcomeLabel = (winRate: number): string => {
        if (winRate < 100) return 'TPK'
        if (winRate === 100 && quintile.median_survivors < quintile.party_size) return 'Pyrrhic Victory'
        return 'Victory'
    }

    const getWinRateBadgeClass = (winRate: number): string => {
        if (winRate < 100) return styles.dangerBadge
        return styles.successBadge
    }

    return (
        <div className={styles.battleCard}>
            <div className={styles.header}>
                <div className={styles.quintileInfo}>
                    <span className={styles.quintileLabel}>{quintile.label}</span>
                    <span className={`${styles.outcomeBadge} ${getWinRateBadgeClass(quintile.win_rate)}`}>
                        {getOutcomeIcon(quintile.win_rate)} {getOutcomeLabel(quintile.win_rate)}
                    </span>
                </div>
                <div className={styles.duration}>
                    Duration: {quintile.battle_duration_rounds} Rounds
                </div>
            </div>

            <div className={styles.combatants}>
                {quintile.median_run_visualization?.map((combatant: CombatantVisualization, index: number) => (
                    <div key={index} className={styles.combatant}>
                        <div className={styles.combatantName}>
                            {combatant.name}
                            {combatant.is_dead && <span className={styles.deathIndicator}> 💀 Dead</span>}
                        </div>
                        <div className={styles.hpBar}>
                            <span className={getHpBarColor(combatant.hp_percentage, combatant.is_dead)}>
                                [{getHpBarFill(combatant.hp_percentage)}] 
                                <span className={styles.hpText}>
                                    {combatant.current_hp.toFixed(0)}/{combatant.max_hp.toFixed(0)}
                                </span>
                            </span>
                        </div>
                    </div>
                )) || (
                    <div className={styles.combatant}>
                        <div className={styles.combatantName}>
                            Loading battle details...
                        </div>
                    </div>
                )}
            </div>

            <div className={styles.footer}>
                <div className={styles.winRate}>
                    Win Rate: {quintile.win_rate.toFixed(1)}%
                </div>
                <div className={styles.survivors}>
                    Survivors: {quintile.median_survivors}/{quintile.party_size}
                </div>
            </div>
        </div>
    )
})

export default BattleCard