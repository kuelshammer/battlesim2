import { FC, useMemo } from 'react'
import { SkylineAnalysis, PlayerSlot, PercentileBucket } from '@/model/model'
import styles from './PartyOverview.module.scss'

interface PartyOverviewProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
}

/**
 * PartyOverview displays HP status across 100 simulation runs.
 *
 * Layout:
 * - Y-axis: Players sorted by survivability (Tank on top → Glass Cannon on bottom)
 * - X-axis: 100 runs (P0-P100 buckets)
 * - Each row: Horizontal bar showing HP % (green) vs damage (red)
 */
const PartyOverview: FC<PartyOverviewProps> = ({ skyline, partySlots }) => {
    // Map party slots to skyline data
    const playerRows = useMemo(() => {
        return partySlots.map((slot) => {
            // Find this player's data in each bucket
            const bucketData = skyline.buckets.map((bucket) => {
                const character = bucket.characters.find(
                    (c) => c.id === slot.playerId || c.name === slot.playerId
                )
                return {
                    percentile: bucket.percentile,
                    character: character || null,
                }
            })

            return {
                slot,
                bucketData,
            }
        })
    }, [skyline.buckets, partySlots])

    if (playerRows.length === 0) {
        return (
            <div className={styles.partyOverview}>
                <p className={styles.emptyMessage}>No player data available</p>
            </div>
        )
    }

    return (
        <div className={styles.partyOverview}>
            <h4 className={styles.title}>Party Overview - HP Status Across 100 Runs</h4>

            <div className={styles.gridContainer} style={{ gridTemplateRows: `repeat(${playerRows.length}, auto)` }}>
                {playerRows.map(({ slot, bucketData }) => (
                    <div key={slot.playerId} className={styles.playerRow}>
                        {/* Player label column */}
                        <div className={styles.playerLabel}>
                            <div className={styles.playerName}>{slot.playerId}</div>
                            <div className={styles.survivabilityBadge}>
                                EHP: {Math.round(slot.survivabilityScore)}
                            </div>
                            {slot.position === 0 && (
                                <div className={styles.roleBadge} title="Tank - Highest Survivability">
                                    🛡️ Tank
                                </div>
                            )}
                            {slot.position === partySlots.length - 1 && partySlots.length > 1 && (
                                <div className={styles.roleBadge} title="Glass Cannon - Lowest Survivability">
                                    🎯 Glass Cannon
                                </div>
                            )}
                        </div>

                        {/* HP bars row */}
                        <div className={styles.barsRow}>
                            {bucketData.map(({ percentile, character }, idx) => {
                                const hpPercent = character?.hpPercent ?? 0
                                const isDead = character?.isDead ?? false

                                return (
                                    <div
                                        key={idx}
                                        className={styles.barCell}
                                        title={`
                                            Run: P${percentile}
                                            Player: ${slot.playerId}
                                            HP: ${character ? `${character.hpPercent}%` : 'N/A'}
                                            ${isDead ? '💀 DECEASED' : 'Alive'}
                                        `}
                                    >
                                        {isDead ? (
                                            <div className={`${styles.barSegment} ${styles.dead}`}>
                                                💀
                                            </div>
                                        ) : (
                                            <div className={styles.barContainer}>
                                                {/* Green portion: Current HP */}
                                                <div
                                                    className={`${styles.barSegment} ${styles.health}`}
                                                    style={{ width: `${hpPercent}%` }}
                                                />
                                                {/* Red portion: Damage taken */}
                                                <div
                                                    className={`${styles.barSegment} ${styles.damage}`}
                                                    style={{ width: `${100 - hpPercent}%` }}
                                                />
                                            </div>
                                        )}
                                    </div>
                                )
                            })}
                        </div>
                    </div>
                ))}
            </div>

            {/* Legend */}
            <div className={styles.legend}>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.health}`} />
                    <span>HP Remaining</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.damage}`} />
                    <span>Damage Taken</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.dead}`}>
                        💀
                    </div>
                    <span>Deceased</span>
                </div>
            </div>

            {/* X-axis labels */}
            <div className={styles.xAxis}>
                {skyline.buckets.filter((_, i) => i % 20 === 0).map((bucket, i) => (
                    <span
                        key={i}
                        className={styles.xAxisLabel}
                        style={{ left: `${(bucket.percentile / 100) * 100}%` }}
                    >
                        P{bucket.percentile}
                    </span>
                ))}
            </div>
        </div>
    )
}

export default PartyOverview
