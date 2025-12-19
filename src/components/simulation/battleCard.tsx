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

    // Map quintile labels to methodology labels
    const getQuintileLabel = (quintileNum: number): string => {
        switch (quintileNum) {
            case 1: return 'Disaster'
            case 2: return 'Struggle'
            case 3: return 'Typical'
            case 4: return 'Heroic'
            case 5: return 'Legend'
            default: return `Quintile ${quintileNum}`
        }
    }

    // Map quintile number to statistical meaning
    const getStatisticalMeaning = (quintileNum: number): string => {
        switch (quintileNum) {
            case 1: return '5th Percentile (Worst 10% Median)'
            case 2: return '25th Percentile (Bad Luck Median)'
            case 3: return '50th Percentile (Global Median)'
            case 4: return '75th Percentile (Good Luck Median)'
            case 5: return '95th Percentile (Best 10% Median)'
            default: return 'Unknown'
        }
    }

    return (
        <div className={styles.battleCard}>
            <div className={styles.header}>
                <div className={styles.quintileInfo}>
                    <div className={styles.quintileLabel}>
                        {getQuintileLabel(quintile.quintile)}
                        <span className={styles.statisticalMeaning}>
                            {getStatisticalMeaning(quintile.quintile)}
                        </span>
                    </div>
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