import { FC, useMemo } from 'react'
import { SkylineAnalysis, PlayerSlot } from '@/model/model'
import { findCharacterInBucket } from './PartyOverview'
import styles from './PlayerGraphs.module.scss'

interface PlayerGraphsProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
}

/**
 * PlayerGraphs displays detailed individual statistics for each player.
 */
const PlayerGraphs: FC<PlayerGraphsProps> = ({ skyline, partySlots }) => {
    const gridColumns = partySlots.length <= 2 ? 1
        : partySlots.length <= 4 ? 2
        : partySlots.length <= 6 ? 3
        : 4

    return (
        <div className={styles.playerGraphs}>
            <h4 className={styles.title}>Individual Performance Breakdown</h4>

            <div className={styles.grid} style={{ gridTemplateColumns: `repeat(${gridColumns}, 1fr)` }}>
                {partySlots.map((slot, playerIdx) => {
                    // Create a list of buckets with this specific player's data attached
                    const playerBuckets = skyline.buckets.map((bucket) => {
                        const character = findCharacterInBucket(bucket.characters, slot.playerId, playerIdx)
                        return { percentile: bucket.percentile, character: character || null }
                    })

                    // Sort these buckets based on THIS player's performance
                    // Criteria: Dead (worst) -> Death Round (earlier is worse) -> HP% (lower is worse) -> Resource% (lower is worse)
                    playerBuckets.sort((a, b) => {
                        const charA = a.character
                        const charB = b.character

                        // 1. Death Status (Dead < Alive)
                        // If one is dead and other is alive, the dead one comes first (lower index)
                        const deadA = charA?.isDead ?? false
                        const deadB = charB?.isDead ?? false
                        if (deadA !== deadB) return deadA ? -1 : 1

                        // 2. Death Round (Earlier < Later)
                        if (deadA && deadB) {
                            const roundA = charA?.deathRound ?? 0
                            const roundB = charB?.deathRound ?? 0
                            if (roundA !== roundB) return roundA - roundB
                        }

                        // 3. HP % (Lower < Higher)
                        const hpA = charA?.hpPercent ?? 0
                        const hpB = charB?.hpPercent ?? 0
                        if (hpA !== hpB) return hpA - hpB

                        // 4. Resource % (Lower < Higher)
                        const resA = charA?.resourcePercent ?? 0
                        const resB = charB?.resourcePercent ?? 0
                        return resA - resB
                    })

                    const deathCount = playerBuckets.filter((b) => b.character?.isDead).length
                    const deathRate = (deathCount / skyline.buckets.length) * 100
                    const hpValues = playerBuckets.map((b) => b.character?.hpPercent ?? 0).filter((hp) => hp > 0)
                    const avgHp = hpValues.length > 0 ? hpValues.reduce((sum, hp) => sum + hp, 0) / hpValues.length : 0

                    const ehpValue = Math.round(slot.survivabilityScore)
                    const displayEHP = isNaN(ehpValue) ? '---' : ehpValue

                    return (
                        <div key={`${slot.playerId}-${slot.position}`} className={styles.playerCard}>
                            <div className={styles.cardHeader}>
                                <div className={styles.playerInfo}>
                                    <h5 className={styles.playerName}>{slot.playerId || `Player ${playerIdx + 1}`}</h5>
                                    <div className={styles.survivabilityBadge}>EHP: {displayEHP}</div>
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