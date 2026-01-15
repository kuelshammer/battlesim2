import React, { FC, memo } from "react"
import { DecileStats, CombatantVisualization } from "../../model/model"
import styles from './battleCard.module.scss'

type PropType = {
    decile: DecileStats
}

const BattleCard: FC<PropType> = memo(({ decile }) => {
    const getHpBarColor = (hpPercentage: number, isDead: boolean): string => {
        if (isDead) return styles.dead
        if (hpPercentage <= 20) return styles.danger
        if (hpPercentage <= 50) return styles.bloodied
        return styles.healthy
    }

    const renderHpBar = (currentHp: number, startHp: number, maxHp: number) => {
        if (!maxHp || maxHp <= 0) return [];
        
        // Calculate segments (10 total)
        const totalSegments = 10;
        
        // Green: Remaining HP
        const greenCount = Math.floor((currentHp / maxHp) * totalSegments);
        
        // Red: Lost in this battle (Start HP - Current HP)
        const newDamage = Math.max(0, startHp - currentHp);
        const redCount = Math.floor((newDamage / maxHp) * totalSegments);
        
        // Grey: Previously lost HP (Max HP - Start HP)
        const greyCount = Math.max(0, totalSegments - greenCount - redCount);
        
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
        if (winRate === 100 && decile.medianSurvivors < decile.partySize) return '⚠️'
        return '✅'
    }

    const getOutcomeLabel = (winRate: number): string => {
        if (winRate < 100) return 'TPK'
        if (winRate === 100 && decile.medianSurvivors < decile.partySize) return 'Pyrrhic Victory'
        return 'Victory'
    }

    const getWinRateBadgeClass = (winRate: number): string => {
        if (winRate < 100) return styles.dangerBadge
        return styles.successBadge
    }

    // Map decile labels to methodology labels
    const getDecileLabel = (decileNum: number): string => {
        switch (decileNum) {
            case 1: return 'Disaster'
            case 3: return 'Struggle'
            case 5: return 'Typical'
            case 8: return 'Heroic'
            case 10: return 'Legend'
            default: return `Decile ${decileNum}`
        }
    }

    // Map decile number to statistical meaning
    const getStatisticalMeaning = (decileNum: number): string => {
        switch (decileNum) {
            case 1: return '5th Percentile (Worst Case)'
            case 3: return '25th Percentile (Bad Luck)'
            case 5: return '50th Percentile (Global Median)'
            case 8: return '75th Percentile (Good Luck)'
            case 10: return '95th Percentile (Best Case)'
            default: {
                const start = (decileNum - 1) * 10;
                const end = decileNum * 10;
                return `${start}-${end}% Performance Slice`;
            }
        }
    }

    return (
        <div className={styles.battleCard} data-testid="timeline-item">
            <div className={styles.header} data-testid="decile-header">
                <div className={styles.decileInfo}>
                    <div className={styles.decileLabel} data-testid="decile-label">
                        {getDecileLabel(decile.decile)}
                        <span className={styles.statisticalMeaning}>
                            {getStatisticalMeaning(decile.decile)}
                        </span>
                    </div>
                    <span className={`${styles.outcomeBadge} ${getWinRateBadgeClass(decile.winRate)}`} data-testid="outcome-badge">
                        {getOutcomeIcon(decile.winRate)} {getOutcomeLabel(decile.winRate)}
                    </span>
                </div>
                <div className={styles.duration} data-testid="battle-duration">
                    Duration: {decile.battleDurationRounds} Rounds
                </div>
            </div>

            <div className={styles.combatants} data-testid="decile-combatants">
                {decile.medianRunVisualization?.map((combatant: CombatantVisualization, index: number) => (
                    <div key={index} className={styles.combatant} data-testid={`combatant-${index}`}>
                        <div className={styles.combatantName} data-testid="creature-name">
                            {combatant.name}
                            {combatant.isDead && <span className={styles.deathIndicator} data-testid="death-indicator"> 💀 Dead</span>}
                        </div>
                        <div className={styles.hpBar} data-testid="hp-bar">
                            <span className={getHpBarColor(combatant.hpPercentage, combatant.isDead)}>
                                [{renderHpBar(combatant.currentHp, combatant.startHp, combatant.maxHp)}] 
                                <span className={styles.hpText} data-testid="hp-text">
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
                <div className={decile.winRate < 100 ? styles.winRateDanger : styles.winRate} data-testid="win-rate">
                    Win Rate: {decile.winRate.toFixed(1)}%
                </div>
                <div className={styles.survivors} data-testid="survivors-count">
                    Survivors: {decile.medianSurvivors}/{decile.partySize}
                </div>
            </div>
        </div>
    )
})

export default BattleCard