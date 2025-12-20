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

    const renderHpBar = (currentHp: number, startHp: number, maxHp: number) => {
        // Calculate segments (10 total)
        const totalSegments = 10;
        
        // Green: Remaining HP
        const greenCount = Math.floor((currentHp / maxHp) * totalSegments);
        
        // Red: Lost in this battle (Start HP - Current HP)
        const newDamage = Math.max(0, startHp - currentHp);
        const redCount = Math.floor((newDamage / maxHp) * totalSegments);
        
        // Grey: Previously lost HP (Max HP - Start HP)
        const greyCount = totalSegments - greenCount - redCount;
        
        const segments = [];
        // Green segments (Remaining)
        for (let i = 0; i < greenCount; i++) {
            segments.push(<span key={`g-${i}`} className={styles.segmentGreen}>█</span>);
        }
        // Red segments (Newly lost)
        for (let i = 0; i < redCount; i++) {
            segments.push(<span key={`r-${i}`} className={styles.segmentRed}>█</span>);
        }
        // Grey segments (Previously lost)
        for (let i = 0; i < greyCount; i++) {
            segments.push(<span key={`gr-${i}`} className={styles.segmentGrey}>░</span>);
        }
        
        // Ensure we always have exactly 10 segments due to rounding
        while (segments.length < totalSegments) {
            segments.push(<span key={`f-${segments.length}`} className={styles.segmentGrey}>░</span>);
        }
        if (segments.length > totalSegments) {
            segments.splice(totalSegments);
        }

        return segments;
    };

    const getOutcomeIcon = (winRate: number): string => {
        if (winRate < 100) return '💀'
        if (winRate === 100 && quintile.medianSurvivors < quintile.partySize) return '⚠️'
        return '✅'
    }

    const getOutcomeLabel = (winRate: number): string => {
        if (winRate < 100) return 'TPK'
        if (winRate === 100 && quintile.medianSurvivors < quintile.partySize) return 'Pyrrhic Victory'
        return 'Victory'
    }

    const getWinRateBadgeClass = (winRate: number): string => {
        if (winRate < 100) return styles.dangerBadge
        return styles.successBadge
    }

    // Map decile labels to methodology labels
    const getQuintileLabel = (quintileNum: number): string => {
        switch (quintileNum) {
            case 1: return 'Disaster'
            case 3: return 'Struggle'
            case 5: return 'Typical'
            case 8: return 'Heroic'
            case 10: return 'Legend'
            default: return `Decile ${quintileNum}`
        }
    }

    // Map decile number to statistical meaning
    const getStatisticalMeaning = (quintileNum: number): string => {
        switch (quintileNum) {
            case 1: return '5th Percentile (Worst Case)'
            case 3: return '25th Percentile (Bad Luck)'
            case 5: return '50th Percentile (Global Median)'
            case 8: return '75th Percentile (Good Luck)'
            case 10: return '95th Percentile (Best Case)'
            default: {
                const start = (quintileNum - 1) * 10;
                const end = quintileNum * 10;
                return `${start}-${end}% Performance Slice`;
            }
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
                    <span className={`${styles.outcomeBadge} ${getWinRateBadgeClass(quintile.winRate)}`}>
                        {getOutcomeIcon(quintile.winRate)} {getOutcomeLabel(quintile.winRate)}
                    </span>
                </div>
                <div className={styles.duration}>
                    Duration: {quintile.battleDurationRounds} Rounds
                </div>
            </div>

            <div className={styles.combatants}>
                {quintile.medianRunVisualization?.map((combatant: CombatantVisualization, index: number) => (
                    <div key={index} className={styles.combatant}>
                        <div className={styles.combatantName}>
                            {combatant.name}
                            {combatant.isDead && <span className={styles.deathIndicator}> 💀 Dead</span>}
                        </div>
                        <div className={styles.hpBar}>
                            <span className={getHpBarColor(combatant.hpPercentage, combatant.isDead)}>
                                [{renderHpBar(combatant.currentHp, combatant.startHp, combatant.maxHp)}] 
                                <span className={styles.hpText}>
                                    {combatant.currentHp}/{combatant.maxHp}
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
                    Win Rate: {quintile.winRate.toFixed(1)}%
                </div>
                <div className={styles.survivors}>
                    Survivors: {quintile.medianSurvivors}/{quintile.partySize}
                </div>
            </div>
        </div>
    )
})

export default BattleCard