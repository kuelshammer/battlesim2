import React, { FC, useRef, useEffect, useMemo, useState } from 'react'
import { SkylineAnalysis, PlayerSlot, CharacterBucketData } from '@/model/model'
import styles from './PartyOverview.module.scss'

interface PartyOverviewProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
    className?: string
}

/**
 * Robust character lookup to handle prefixing and ID/Name mismatches
 */
export const findCharacterInBucket = (bucketCharacters: CharacterBucketData[], playerId: string) => {
    return bucketCharacters.find(c => {
        const pid = playerId.toLowerCase()
        const cid = (c.id || '').toLowerCase()
        const cname = (c.name || '').toLowerCase()
        return cid === pid || 
               cname === pid || 
               (pid && cid && pid.includes(cid)) || 
               (cid && pid && cid.includes(pid))
    })
}

/**
 * PartyOverview displays two grouped stacked bar charts (Vitality and Power).
 * X-axis: 100 runs sorted by Overall Party Success.
 */
const PartyOverview: FC<PartyOverviewProps> = ({ skyline, partySlots, className }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const [width, setWidth] = useState(0)

    useEffect(() => {
        if (!containerRef.current) return
        const updateWidth = () => setWidth(containerRef.current?.clientWidth || 0)
        const resizeObserver = new ResizeObserver(updateWidth)
        resizeObserver.observe(containerRef.current)
        updateWidth()
        return () => resizeObserver.disconnect()
    }, [])

    const partySize = skyline.partySize || partySlots.length

    const sortedPlayers = useMemo(() => {
        return [...partySlots].sort((a, b) => b.survivabilityScore - a.survivabilityScore)
    }, [partySlots])

    const sortedBuckets = useMemo(() => {
        return [...skyline.buckets].sort((a, b) => {
            const sA = partySize - a.deathCount
            const sB = partySize - b.deathCount
            if (sA !== sB) return sA - sB
            if (a.partyHpPercent !== b.partyHpPercent) return a.partyHpPercent - b.partyHpPercent
            return a.partyResourcePercent - b.partyResourcePercent
        })
    }, [skyline.buckets, partySize])

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas || width === 0) return
        const ctx = canvas.getContext('2d')
        if (!ctx) return

        const RUNS_COUNT = 100
        const BAND_HEIGHT = 60
        const LABEL_HEIGHT = 20
        const GAP_BETWEEN_BANDS = 10
        const TOTAL_HEIGHT = (BAND_HEIGHT * 2) + LABEL_HEIGHT + GAP_BETWEEN_BANDS
        const dpr = window.devicePixelRatio || 1
        
        canvas.width = width * dpr
        canvas.height = TOTAL_HEIGHT * dpr
        canvas.style.width = `${width}px`
        canvas.style.height = `${TOTAL_HEIGHT}px`
        ctx.scale(dpr, dpr)

        ctx.clearRect(0, 0, width, TOTAL_HEIGHT)

        const bucketGap = 2
        const totalGapSpace = (RUNS_COUNT - 1) * bucketGap
        const groupWidth = (width - totalGapSpace) / RUNS_COUNT
        const stripeWidth = groupWidth / partySize

        // Backgrounds
        ctx.fillStyle = '#050505'
        ctx.fillRect(0, 0, width, BAND_HEIGHT) // Vitality band
        ctx.fillRect(0, BAND_HEIGHT + GAP_BETWEEN_BANDS, width, BAND_HEIGHT) // Power band

        sortedBuckets.forEach((bucket, runIdx) => {
            const groupX = runIdx * (groupWidth + bucketGap)
            
            sortedPlayers.forEach((playerSlot, playerIdx) => {
                const charData = findCharacterInBucket(bucket.characters, playerSlot.playerId)
                const stripeX = groupX + (playerIdx * stripeWidth)
                const drawWidth = Math.max(0.5, stripeWidth)

                if (!charData) {
                    // Fallback visual if no data found
                    ctx.fillStyle = '#111'
                    ctx.fillRect(stripeX, 0, drawWidth, BAND_HEIGHT)
                    ctx.fillRect(stripeX, BAND_HEIGHT + GAP_BETWEEN_BANDS, drawWidth, BAND_HEIGHT)
                    return
                }

                // --- Vitality Band (HP) ---
                if (charData.isDead) {
                    ctx.fillStyle = '#000000'
                    ctx.fillRect(stripeX, 0, drawWidth, BAND_HEIGHT)
                } else {
                    const hpPct = Math.max(0, Math.min(100, charData.hpPercent)) / 100
                    const hpH = BAND_HEIGHT * hpPct
                    ctx.fillStyle = '#22c55e' // Life
                    ctx.fillRect(stripeX, BAND_HEIGHT - hpH, drawWidth, hpH)
                    ctx.fillStyle = '#ef4444' // Wounds
                    ctx.fillRect(stripeX, 0, drawWidth, BAND_HEIGHT - hpH)
                }

                // --- Power Band (Resources) ---
                const resPct = Math.max(0, Math.min(100, charData.resourcePercent)) / 100
                const resH = BAND_HEIGHT * resPct
                ctx.fillStyle = '#3b82f6' // Power
                ctx.fillRect(stripeX, BAND_HEIGHT + GAP_BETWEEN_BANDS, drawWidth, resH)
                ctx.fillStyle = '#eab308' // Spent
                ctx.fillRect(stripeX, BAND_HEIGHT + GAP_BETWEEN_BANDS + resH, drawWidth, BAND_HEIGHT - resH)
            })

            // Labels
            if (runIdx % 20 === 0 || runIdx === 99) {
                const labelX = groupX + groupWidth / 2
                ctx.fillStyle = 'rgba(232, 224, 208, 0.5)'
                ctx.font = '10px "Courier New", monospace'
                ctx.textAlign = 'center'
                ctx.fillText(`P${runIdx === 99 ? 100 : runIdx}`, labelX, TOTAL_HEIGHT - 4)
            }
        })

        // Axis divider lines
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.2)'
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(0, BAND_HEIGHT)
        ctx.lineTo(width, BAND_HEIGHT)
        ctx.moveTo(0, BAND_HEIGHT + GAP_BETWEEN_BANDS)
        ctx.lineTo(width, BAND_HEIGHT + GAP_BETWEEN_BANDS)
        ctx.stroke()

    }, [sortedBuckets, sortedPlayers, width, partySize])

    return (
        <div className={`${styles.partyOverview} ${className || ''}`}>
            <div className={styles.header}>
                <h4 className={styles.title}>Survival Spectrogram</h4>
                <div className={styles.subtext}>100 Timelines • Grouped by Player</div>
            </div>

            <div className={styles.legend}>
                <span className={styles.legendLabel}>Cohort (Tank → Glass):</span>
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
                <div className={styles.labelVitality}>Vitality</div>
                <div className={styles.labelPower}>Power</div>
                <canvas ref={canvasRef} />
            </div>
            
            <div className={styles.colorKey}>
                <div className={styles.keyItem}><div className={`${styles.swatch} ${styles.green}`} /> Life</div>
                <div className={styles.keyItem}><div className={`${styles.swatch} ${styles.red}`} /> Wounds</div>
                <div className={styles.separator} />
                <div className={styles.keyItem}><div className={`${styles.swatch} ${styles.blue}`} /> Power</div>
                <div className={styles.keyItem}><div className={`${styles.swatch} ${styles.yellow}`} /> Spent</div>
                <div className={styles.separator} />
                <div className={styles.keyItem}><div className={`${styles.swatch} ${styles.dead}`} /> Fallen</div>
            </div>
        </div>
    )
}

export default PartyOverview