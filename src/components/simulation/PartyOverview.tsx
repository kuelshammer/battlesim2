import React, { FC, useRef, useEffect, useMemo, useState } from 'react'
import { SkylineAnalysis, PlayerSlot, PercentileBucket } from '@/model/model'
import styles from './PartyOverview.module.scss'

interface PartyOverviewProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
    className?: string
}

/**
 * PartyOverview displays a horizontal spectrogram of HP and resources across 100 runs.
 *
 * Layout:
 * - X-axis: 100 runs (sorted by Survivorship -> HP%)
 * - Each run is a vertical group of N stripes (where N is party size)
 * - Dynamic Sizing: Width scales to fit canvas
 * - ABOVE axis: HP bars stacked by Tankiness
 * - BELOW axis: Resource bars stacked by Tankiness
 * - Inner Group Sorting: Always Tank (Left) -> Glass Cannon (Right) within a run group
 */
const PartyOverview: FC<PartyOverviewProps> = ({ skyline, partySlots, className }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const [width, setWidth] = useState(0)

    // Handle resize
    useEffect(() => {
        if (!containerRef.current) return
        
        const updateWidth = () => {
            if (containerRef.current) {
                setWidth(containerRef.current.clientWidth)
            }
        }
        
        const resizeObserver = new ResizeObserver(updateWidth)
        resizeObserver.observe(containerRef.current)
        updateWidth()
        
        return () => resizeObserver.disconnect()
    }, [])

    const partySize = skyline.partySize || partySlots.length

    // 1. Sort Players by Survivability (Highest/Tank -> Lowest/Glass Cannon)
    // The user wants: "Highest on Left -> Lowest on Right" within a group
    // In our vertical stack logic (previous code), it was Top -> Bottom.
    // "Bottom Panel (Resources): Each stripe is a stacked bar." 
    // Wait, the user said: "For each of the 100 X-Axis buckets, draw a 'Group' of N vertical stripes"
    // So the players are SIDE-BY-SIDE in a group, not stacked vertically on top of each other?
    // "Inner-Group: Within each run group, the players must ALWAYS be ordered by their survivabilityScore (Highest on Left -> Lowest on Right)."
    // Yes, they are vertical stripes next to each other.
    
    // Top Panel (HP): Each stripe is a stacked bar (Green vs Red).
    // Bottom Panel (Resources): Each stripe is a stacked bar (Blue vs Yellow).
    
    const sortedPlayers = useMemo(() => {
        return [...partySlots].sort((a, b) => b.survivabilityScore - a.survivabilityScore)
    }, [partySlots])

    // 2. Triage Sort for Runs (X-Axis)
    // Primary: survivorCount (Ascending) -> TPKs on left
    // Secondary: totalPartyHpPercent (Ascending)
    const sortedBuckets = useMemo(() => {
        // Clone buckets to avoid mutating prop
        const buckets = [...skyline.buckets]
        
        return buckets.sort((a, b) => {
            // survivorCount = partySize - deathCount
            // So survivorCount Ascending == deathCount Descending
            const survivorsA = partySize - a.deathCount
            const survivorsB = partySize - b.deathCount
            
            if (survivorsA !== survivorsB) {
                return survivorsA - survivorsB
            }
            
            // Secondary: HP Percent
            return a.partyHpPercent - b.partyHpPercent
        })
    }, [skyline.buckets, partySize])

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas || width === 0) return

        const ctx = canvas.getContext('2d')
        if (!ctx) return

        // Constants
        const RUNS_COUNT = 100 // We enforce 100 runs
        const CANVAS_HEIGHT = 120 // 60px Top + 60px Bottom
        const AXIS_Y = CANVAS_HEIGHT / 2
        
        // Handle high-DPI displays
        const dpr = window.devicePixelRatio || 1
        canvas.width = width * dpr
        canvas.height = CANVAS_HEIGHT * dpr
        canvas.style.width = `${width}px`
        canvas.style.height = `${CANVAS_HEIGHT}px`
        ctx.scale(dpr, dpr)

        // Clear
        ctx.clearRect(0, 0, width, CANVAS_HEIGHT)

        // Calculate sizing
        // Total Groups = 100
        // Stripes per Group = PartySize
        // Total Stripes = 100 * PartySize
        // We also need gaps between groups.
        
        const gapSize = partySize > 6 ? 0.5 : 1
        const totalGapSpace = (RUNS_COUNT - 1) * gapSize
        const availableWidth = width - totalGapSpace
        const stripeWidth = availableWidth / (RUNS_COUNT * partySize)
        
        // Draw background/guides
        ctx.fillStyle = '#0a0a0a' // Deep void
        ctx.fillRect(0, 0, width, CANVAS_HEIGHT)
        
        // Axis line
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.3)' // Gold axis
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(0, AXIS_Y)
        ctx.lineTo(width, AXIS_Y)
        ctx.stroke()

        // Render Runs
        sortedBuckets.forEach((bucket, runIdx) => {
            // X position for this run group
            const groupX = runIdx * (partySize * stripeWidth + gapSize)
            
            // Iterate players in fixed survivability order
            sortedPlayers.forEach((playerSlot, playerIdx) => {
                const charData = bucket.characters.find(c => c.id === playerSlot.playerId || c.name === playerSlot.playerId)
                
                // Stripe X position
                const stripeX = groupX + (playerIdx * stripeWidth)
                
                // Safety check for sub-pixel rendering gaps
                const drawWidth = stripeWidth + 0.1 

                if (!charData) {
                    // Missing data placeholder
                    ctx.fillStyle = '#1f2937'
                    ctx.fillRect(stripeX, 0, drawWidth, CANVAS_HEIGHT)
                    return
                }

                // --- Top Panel: HP (Above Axis) ---
                const panelHeight = AXIS_Y
                
                if (charData.isDead) {
                    // Dead State
                    ctx.fillStyle = '#000000' // Void
                    ctx.fillRect(stripeX, 0, drawWidth, panelHeight) // Fill entire top panel
                } else {
                    const hpPct = Math.max(0, Math.min(100, charData.hpPercent)) / 100
                    const hpHeight = panelHeight * hpPct
                    const dmgHeight = panelHeight * (1 - hpPct)
                    
                    // Green (HP) - Bottom of the top panel (closer to axis)
                    ctx.fillStyle = '#10b981' // Emerald
                    ctx.fillRect(stripeX, AXIS_Y - hpHeight, drawWidth, hpHeight)
                    
                    // Red (Damage) - Top of the top panel (away from axis)
                    ctx.fillStyle = '#7f1d1d' // Deep Blood
                    ctx.fillRect(stripeX, 0, drawWidth, dmgHeight)
                }

                // --- Bottom Panel: Resources (Below Axis) ---
                const resPct = Math.max(0, Math.min(100, charData.resourcePercent)) / 100
                const resHeight = panelHeight * resPct
                const spentHeight = panelHeight * (1 - resPct)
                
                // Blue (Remaining) - Top of the bottom panel (closer to axis)
                ctx.fillStyle = '#2563eb' // Royal Blue
                ctx.fillRect(stripeX, AXIS_Y, drawWidth, resHeight)
                
                // Yellow (Spent) - Bottom of the bottom panel (away from axis)
                ctx.fillStyle = '#b45309' // Deep Amber
                ctx.fillRect(stripeX, AXIS_Y + resHeight, drawWidth, spentHeight)
            })
        })

    }, [skyline, partySlots, width, sortedPlayers, sortedBuckets, partySize])

    return (
        <div className={`${styles.partyOverview} ${className || ''}`}>
            <div className={styles.header}>
                <h4 className={styles.title}>Survival Spectrogram</h4>
                <div className={styles.subtext}>
                    100 Timelines • Sorted by Fate
                </div>
            </div>

            <div className={styles.legend}>
                <span className={styles.legendLabel}>Cohort:</span>
                <div className={styles.legendGroup}>
                    {sortedPlayers.map((p, i) => (
                        <span key={p.playerId} className={styles.playerTag}>
                            {i === 0 && <span className={styles.roleIcon}>🛡️</span>}
                            {i === sortedPlayers.length - 1 && <span className={styles.roleIcon}>⚡</span>}
                            {p.playerId}
                        </span>
                    ))}
                </div>
            </div>

            <div ref={containerRef} className={styles.canvasContainer}>
                <canvas ref={canvasRef} />
            </div>
            
            <div className={styles.colorKey}>
                <div className={styles.keyItem}>
                    <div className={`${styles.swatch} ${styles.green}`} /> Life
                </div>
                <div className={styles.keyItem}>
                    <div className={`${styles.swatch} ${styles.red}`} /> Wounds
                </div>
                <div className={styles.separator} />
                <div className={styles.keyItem}>
                    <div className={`${styles.swatch} ${styles.blue}`} /> Power
                </div>
                <div className={styles.keyItem}>
                    <div className={`${styles.swatch} ${styles.yellow}`} /> Spent
                </div>
                <div className={styles.separator} />
                <div className={styles.keyItem}>
                    <div className={`${styles.swatch} ${styles.dead}`} /> Fallen
                </div>
            </div>
        </div>
    )
}

export default PartyOverview