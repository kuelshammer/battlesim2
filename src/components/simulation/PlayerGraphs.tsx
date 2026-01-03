import { FC, useMemo } from 'react'
import { SkylineAnalysis, PlayerSlot } from '@/model/model'
import styles from './PlayerGraphs.module.scss'

interface PlayerGraphsProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
}

/**
 * PlayerGraphs displays detailed individual statistics for each player.
 */
const PlayerGraphs: FC<PlayerGraphsProps> = ({ skyline, partySlots }) => {
    const partySize = skyline.partySize || partySlots.length

    // Use same sort as PartyOverview
    const sortedBuckets = useMemo(() => {
        return [...skyline.buckets].sort((a, b) => {
            const sA = partySize - a.deathCount
            const sB = partySize - b.deathCount
            if (sA !== sB) return sA - sB
            if (a.partyHpPercent !== b.partyHpPercent) return a.partyHpPercent - b.partyHpPercent
            return a.partyResourcePercent - b.partyResourcePercent
        })
    }, [skyline.buckets, partySize])

    const gridColumns = partySlots.length <= 2 ? 1
        : partySlots.length <= 4 ? 2
        : partySlots.length <= 6 ? 3
        : 4

    return (
        <div className={styles.playerGraphs}>
            <h4 className={styles.title}>Individual Performance Breakdown</h4>

            <div className={styles.grid} style={{ gridTemplateColumns: `repeat(${gridColumns}, 1fr)` }}>
                {partySlots.map((slot) => {
                    const playerBuckets = sortedBuckets.map((bucket) => {
                        const character = bucket.characters.find(
                            (c) => c.id === slot.playerId || c.name === slot.playerId
                        )
                        return { percentile: bucket.percentile, character: character || null }
                    })

                    const deathCount = playerBuckets.filter((b) => b.character?.isDead).length
                    const deathRate = (deathCount / sortedBuckets.length) * 100
                    const hpValues = playerBuckets.map((b) => b.character?.hpPercent ?? 0).filter((hp) => hp > 0)
                    const avgHp = hpValues.length > 0 ? hpValues.reduce((sum, hp) => sum + hp, 0) / hpValues.length : 0

                    return (
                        <div key={`${slot.playerId}-${slot.position}`} className={styles.playerCard}>
                            <div className={styles.cardHeader}>
                                <div className={styles.playerInfo}>
                                    <h5 className={styles.playerName}>{slot.playerId}</h5>
                                    <div className={styles.survivabilityBadge}>EHP: {Math.round(slot.survivabilityScore)}</div>
                                </div>
                                <div className={styles.roleIcon}>
                                    {slot.position === 0 && <span title="Shield Wall">🛡️</span>}
                                    {slot.position === partySlots.length - 1 && partySlots.length > 1 && <span title="Glass Cannon">⚡</span>}
                                </div>
                            </div>

                            <div className={styles.statsSummary}>
                                <div className={styles.statItem}><span className={styles.statLabel}>Casualty Rate:</span> <span className={deathRate > 50 ? styles.highDeath : styles.lowDeath}>{deathRate.toFixed(1)}%</span></div>
                                <div className={styles.statItem}><span className={styles.statLabel}>Avg Vitality:</span> <span className={styles.statValue}>{avgHp.toFixed(0)}%</span></div>
                            </div>

                            <div className={styles.hpVisualization}>
                                {playerBuckets.map(({ percentile, character }, idx) => {
                                    const hpPct = character?.hpPercent ?? 0
                                    const resPct = character?.resourcePercent ?? 0
                                    const isDead = character?.isDead ?? false

                                    return (
                                        <div key={idx} className={styles.barGroup}>
                                            <div 
                                                className={styles.hpBar} 
                                                style={{ 
                                                    height: '50%', 
                                                    bottom: '50%',
                                                    background: isDead ? '#000' : `linear-gradient(0deg, #22c55e ${hpPct}%, #ef4444 ${hpPct}%)` 
                                                }} 
                                            />
                                            <div 
                                                className={styles.resBar} 
                                                style={{ 
                                                    height: '50%', 
                                                    top: '50%',
                                                    background: `linear-gradient(180deg, #3b82f6 ${resPct}%, #eab308 ${resPct}%)` 
                                                }} 
                                            />
                                        </div>
                                    )
                                })}
                            </div>

                            <div className={styles.miniLegend}>
                                <span className={styles.legendLabel}>Worst Runs</span>
                                <span className={styles.legendLabel}>Best Runs</span>
                            </div>
                        </div>
                    )
                })}
            </div>
        </div>
    )
}

export default PlayerGraphs