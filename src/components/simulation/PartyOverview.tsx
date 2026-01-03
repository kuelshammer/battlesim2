import React, { FC, useRef, useEffect, useMemo, useState } from 'react'
import { SkylineAnalysis, PlayerSlot } from '@/model/model'
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
    const sortedPlayers = useMemo(() => {
        return [...partySlots].sort((a, b) => b.survivabilityScore - a.survivabilityScore)
    }, [partySlots])

    // 2. Triage Sort for Runs (X-Axis)
    const sortedBuckets = useMemo(() => {
        const buckets = [...skyline.buckets]
        return buckets.sort((a, b) => {
            const survivorsA = partySize - a.deathCount
            const survivorsB = partySize - b.deathCount
            if (survivorsA !== survivorsB) return survivorsA - survivorsB
            return a.partyHpPercent - b.partyHpPercent
        })
    }, [skyline.buckets, partySize])

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas || width === 0) return

        const ctx = canvas.getContext('2d')
        if (!ctx) return

        // Constants
        const RUNS_COUNT = 100
        const GRAPH_HEIGHT = 120
        const LABEL_HEIGHT = 20
        const TOTAL_HEIGHT = GRAPH_HEIGHT + LABEL_HEIGHT
        const AXIS_Y = GRAPH_HEIGHT / 2
        
        const dpr = window.devicePixelRatio || 1
        canvas.width = width * dpr
        canvas.height = TOTAL_HEIGHT * dpr
        canvas.style.width = `${width}px`
        canvas.style.height = `${TOTAL_HEIGHT}px`
        ctx.scale(dpr, dpr)

        ctx.clearRect(0, 0, width, TOTAL_HEIGHT)

        // Sizing
        const gapSize = 2 // Gap between buckets
        const totalGapSpace = (RUNS_COUNT - 1) * gapSize
        const availableWidth = width - totalGapSpace
        const groupWidth = availableWidth / RUNS_COUNT
        const stripeWidth = groupWidth / partySize
        
        // Draw background
        ctx.fillStyle = '#0a0a0a'
        ctx.fillRect(0, 0, width, GRAPH_HEIGHT)
        
        // Axis line
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.4)'
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(0, AXIS_Y)
        ctx.lineTo(width, AXIS_Y)
        ctx.stroke()

        // Render Runs
        sortedBuckets.forEach((bucket, runIdx) => {
            const groupX = runIdx * (groupWidth + gapSize)
            
            sortedPlayers.forEach((playerSlot, playerIdx) => {
                // Safer lookup to prevent crash
                const charData = bucket.characters.find(c => {
                    const pid = playerSlot.playerId || ''
                    const cid = c.id || ''
                    const cname = c.name || ''
                    return cid === pid || 
                           cname === pid || 
                           (pid && cid && pid.includes(cid)) || 
                           (cid && pid && cid.includes(pid))
                })
                
                const stripeX = groupX + (playerIdx * stripeWidth)
                const drawWidth = Math.max(0.5, stripeWidth) 

                if (!charData) {
                    ctx.fillStyle = '#1f2937' 
                    ctx.fillRect(stripeX, 0, drawWidth, GRAPH_HEIGHT)
                    return
                }

                const panelHalf = AXIS_Y
                
                // HP Panel (Above Axis)
                if (charData.isDead) {
                    ctx.fillStyle = '#000000'
                    ctx.fillRect(stripeX, 0, drawWidth, panelHalf)
                } else {
                    const hpPct = Math.max(0, Math.min(100, charData.hpPercent)) / 100
                    const hpHeight = panelHalf * hpPct
                    const dmgHeight = panelHalf * (1 - hpPct)
                    
                    ctx.fillStyle = '#22c55e' // Vibrant Green
                    ctx.fillRect(stripeX, AXIS_Y - hpHeight, drawWidth, hpHeight)
                    
                    ctx.fillStyle = '#ef4444' // Vibrant Red
                    ctx.fillRect(stripeX, 0, drawWidth, dmgHeight)
                }

                // Resource Panel (Below Axis)
                const resPct = Math.max(0, Math.min(100, charData.resourcePercent)) / 100
                const resHeight = panelHalf * resPct
                const spentHeight = panelHalf * (1 - resPct)
                
                ctx.fillStyle = '#3b82f6' // Vibrant Blue
                ctx.fillRect(stripeX, AXIS_Y, drawWidth, resHeight)
                
                ctx.fillStyle = '#eab308' // Vibrant Yellow
                ctx.fillRect(stripeX, AXIS_Y + resHeight, drawWidth, spentHeight)
            })

            // Draw labels for every 20th bucket
            if (runIdx % 20 === 0 || runIdx === 99) {
                ctx.fillStyle = 'rgba(232, 224, 208, 0.5)'
                ctx.font = '10px "Courier New", monospace'
                ctx.textAlign = 'center'
                const labelX = groupX + groupWidth / 2
                ctx.fillText(`P${runIdx === 99 ? 100 : runIdx}`, labelX, GRAPH_HEIGHT + 14)
                
                // Tick mark
                ctx.strokeStyle = 'rgba(212, 175, 55, 0.3)'
                ctx.beginPath()
                ctx.moveTo(labelX, GRAPH_HEIGHT)
                ctx.lineTo(labelX, GRAPH_HEIGHT + 4)
                ctx.stroke()
            }
        })

    }, [skyline, partySlots, width, sortedPlayers, sortedBuckets, partySize])

    return (
        <div className={`${styles.partyOverview} ${className || ''}`}>
            <div className={styles.header}>
                <h4 className={styles.title}>Survival Spectrogram</h4>
                <div className={styles.subtext}>
                    100 Timelines • Sorted by Fatality
                </div>
            </div>

            <div className={styles.legend}>
                <span className={styles.legendLabel}>Cohort:</span>
                <div className={styles.legendGroup}>
                    {sortedPlayers.map((p, i) => (
                        <span key={`${p.playerId}-${p.position}`} className={styles.playerTag}>
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