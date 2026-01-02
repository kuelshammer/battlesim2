import { FC } from 'react'
import { SkylineAnalysis, PlayerSlot } from '@/model/model'
import styles from './PlayerGraphs.module.scss'

interface PlayerGraphsProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
}

/**
 * PlayerGraphs displays detailed individual statistics for each player.
 *
 * Layout:
 * - Responsive grid: Auto-fit columns based on party size
 * - Each player card shows:
 *   - Name and role icon
 *   - Survivability score
 *   - HP distribution across 100 runs
 *   - Death rate and statistics
 * - Cards are ordered by survivability (Tank → Glass Cannon)
 */
const PlayerGraphs: FC<PlayerGraphsProps> = ({ skyline, partySlots }) => {
    // Calculate grid columns based on party size
    const gridColumns = partySlots.length <= 2 ? 1
        : partySlots.length <= 4 ? 2
        : partySlots.length <= 6 ? 3
        : 4

    return (
        <div className={styles.playerGraphs}>
            <h4 className={styles.title}>Individual Player Statistics</h4>

            <div
                className={styles.grid}
                style={{
                    gridTemplateColumns: `repeat(${gridColumns}, 1fr)`,
                }}
            >
                {partySlots.map((slot) => {
                    // Find this player's data across all buckets
                    const playerBuckets = skyline.buckets.map((bucket) => {
                        const character = bucket.characters.find(
                            (c) => c.id === slot.playerId || c.name === slot.playerId
                        )
                        return {
                            percentile: bucket.percentile,
                            character: character || null,
                        }
                    })

                    // Calculate statistics
                    const deathCount = playerBuckets.filter((b) => b.character?.isDead).length
                    const deathRate = (deathCount / skyline.buckets.length) * 100

                    const hpValues = playerBuckets
                        .map((b) => b.character?.hpPercent ?? 0)
                        .filter((hp) => hp > 0)

                    const avgHp = hpValues.length > 0
                        ? hpValues.reduce((sum, hp) => sum + hp, 0) / hpValues.length
                        : 0

                    const worstHp = Math.min(...hpValues, 100)
                    const bestHp = Math.max(...hpValues, 0)

                    return (
                        <div key={slot.playerId} className={styles.playerCard}>
                            {/* Card Header */}
                            <div className={styles.cardHeader}>
                                <div className={styles.playerInfo}>
                                    <h5 className={styles.playerName}>{slot.playerId}</h5>
                                    <div className={styles.survivabilityBadge}>
                                        EHP: {Math.round(slot.survivabilityScore)}
                                    </div>
                                </div>
                                <div className={styles.roleIcon}>
                                    {slot.position === 0 && (
                                        <span title="Tank - Highest Survivability">🛡️</span>
                                    )}
                                    {slot.position === partySlots.length - 1 && partySlots.length > 1 && (
                                        <span title="Glass Cannon - Lowest Survivability">🎯</span>
                                    )}
                                </div>
                            </div>

                            {/* Statistics Summary */}
                            <div className={styles.statsSummary}>
                                <div className={styles.statItem}>
                                    <span className={styles.statLabel}>Death Rate:</span>
                                    <span className={`${styles.statValue} ${deathRate > 50 ? styles.highDeath : styles.lowDeath}`}>
                                        {deathRate.toFixed(1)}%
                                    </span>
                                </div>
                                <div className={styles.statItem}>
                                    <span className={styles.statLabel}>Avg HP:</span>
                                    <span className={styles.statValue}>{avgHp.toFixed(0)}%</span>
                                </div>
                                <div className={styles.statItem}>
                                    <span className={styles.statLabel}>Range:</span>
                                    <span className={styles.statValue}>
                                        {worstHp.toFixed(0)}% - {bestHp.toFixed(0)}%
                                    </span>
                                </div>
                            </div>

                            {/* HP Visualization */}
                            <div className={styles.hpVisualization}>
                                {playerBuckets.map(({ percentile, character }, idx) => {
                                    const hpPercent = character?.hpPercent ?? 0
                                    const isDead = character?.isDead ?? false

                                    return (
                                        <div
                                            key={idx}
                                            className={styles.hpBar}
                                            style={{
                                                height: `${hpPercent}%`,
                                                background: isDead ? '#0f172a' : hpPercent > 50 ? '#22c55e' : hpPercent > 25 ? '#ffaa44' : '#ef4444',
                                            }}
                                            title={`
                                                Run: P${percentile}
                                                HP: ${character ? `${character.hpPercent}%` : 'N/A'}
                                                ${isDead ? '💀 DECEASED' : 'Alive'}
                                            `}
                                        />
                                    )
                                })}
                            </div>

                            {/* Mini-legend */}
                            <div className={styles.miniLegend}>
                                <span className={styles.legendLabel}>P0 (Worst)</span>
                                <span className={styles.legendLabel}>P100 (Best)</span>
                            </div>
                        </div>
                    )
                })}
            </div>
        </div>
    )
}

export default PlayerGraphs
