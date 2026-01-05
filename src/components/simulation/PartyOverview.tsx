import React, { FC, useRef, useEffect, useMemo, useState, useCallback } from 'react'
import { SkylineAnalysis, PlayerSlot, CharacterBucketData } from '@/model/model'
import styles from './PartyOverview.module.scss'
import { useCrosshair, useCrosshairBucketRegistration } from './CrosshairContext'
import CrosshairLine, { CrosshairTooltip } from './CrosshairLine'

interface PartyOverviewProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
    playerNames?: Map<string, string>
    className?: string
}

/**
 * Robust character lookup to handle prefixing and ID/Name mismatches
 */
export const findCharacterInBucket = (bucketCharacters: CharacterBucketData[], playerId: string, fallbackIndex?: number) => {
    if (!playerId || typeof playerId !== 'string' || playerId.trim() === '') {
        if (fallbackIndex !== undefined && bucketCharacters && bucketCharacters[fallbackIndex]) {
            return bucketCharacters[fallbackIndex]
        }
        return undefined
    }

    const pid = playerId.toLowerCase()
    
    // 1. Try exact match on ID or Name
    let found = bucketCharacters.find(c => {
        if (!c) return false
        const cid = (c.id || '').toLowerCase()
        const cname = (c.name || '').toLowerCase()
        return cid === pid || cname === pid
    })

    // 2. Try partial match
    if (!found) {
        found = bucketCharacters.find(c => {
            if (!c) return false
            const cid = (c.id || '').toLowerCase()
            const cname = (c.name || '').toLowerCase()
            return (cid && cid.includes(pid)) || 
                   (pid && pid.includes(cid)) || 
                   (cname && cname.includes(pid)) || 
                   (pid && pid.includes(cname))
        })
    }

    // 3. Fallback to index if still not found and index is provided
    if (!found && fallbackIndex !== undefined && bucketCharacters && bucketCharacters[fallbackIndex]) {
        return bucketCharacters[fallbackIndex]
    }

    return found
}

/**
 * PartyOverview displays two grouped stacked bar charts (Vitality and Power).
 */
const PartyOverview: FC<PartyOverviewProps> = ({ skyline, partySlots, playerNames, className }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const containerRef = useRef<HTMLDivElement>(null)
    const [width, setWidth] = useState(0)
    
    // State for smoothed data
    const [displayBuckets, setDisplayBuckets] = useState(skyline.buckets)
    const animationRef = useRef<number | null>(null)

    const { state: crosshairState, setCrosshair, setHoveredCharacter, clearCrosshair } = useCrosshair()
    const [mousePos, setMousePos] = useState({ x: 0, y: 0 })

    useEffect(() => {
        if (!containerRef.current) return
        const updateWidth = () => setWidth(containerRef.current?.clientWidth || 0)
        const resizeObserver = new ResizeObserver(updateWidth)
        resizeObserver.observe(containerRef.current)
        updateWidth()
        return () => resizeObserver.disconnect()
    }, [])

    const partySize = skyline.partySize || partySlots.length
    const RUNS_COUNT = 100
    const BAND_HEIGHT = 60
    const LABEL_HEIGHT = 20
    const GAP_BETWEEN_BANDS = 0 // Total: 60 + 60 + 20 = 140px
    const TOTAL_HEIGHT = (BAND_HEIGHT * 2) + LABEL_HEIGHT
    const bucketGap = 1

    // Register buckets for tooltips
    useCrosshairBucketRegistration(`party-overview-${skyline.encounterIndex ?? 'overall'}`, skyline.buckets)

    const sortedPlayers = useMemo(() => {
        return [...partySlots].sort((a, b) => (b.survivabilityScore || 0) - (a.survivabilityScore || 0))
    }, [partySlots])

    // Smooth data transition
    useEffect(() => {
        const startTime = performance.now();
        const duration = 400; // 400ms transition
        const startBuckets = displayBuckets;
        const targetBuckets = skyline.buckets;

        // If counts differ (e.g. initial load), skip animation
        if (startBuckets.length !== targetBuckets.length) {
            setDisplayBuckets(targetBuckets);
            return;
        }

        const animate = (currentTime: number) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            
            // Easing function (easeOutQuad)
            const ease = progress * (2 - progress);

            if (progress < 1) {
                const interpolated = targetBuckets.map((target, bIdx) => {
                    const start = startBuckets[bIdx];
                    return {
                        ...target,
                        partyHpPercent: start.partyHpPercent + (target.partyHpPercent - start.partyHpPercent) * ease,
                        partyResourcePercent: start.partyResourcePercent + (target.partyResourcePercent - start.partyResourcePercent) * ease,
                        characters: target.characters.map((tChar, cIdx) => {
                            const sChar = start.characters.find(c => c.id === tChar.id) || tChar;
                            return {
                                ...tChar,
                                hpPercent: sChar.hpPercent + (tChar.hpPercent - sChar.hpPercent) * ease,
                                resourcePercent: sChar.resourcePercent + (tChar.resourcePercent - sChar.resourcePercent) * ease
                            }
                        })
                    };
                });
                setDisplayBuckets(interpolated);
                animationRef.current = requestAnimationFrame(animate);
            } else {
                setDisplayBuckets(targetBuckets);
            }
        };

        animationRef.current = requestAnimationFrame(animate);
        return () => {
            if (animationRef.current) cancelAnimationFrame(animationRef.current);
        };
    }, [skyline.buckets]);

    const sortedBuckets = useMemo(() => {
        return [...displayBuckets].sort((a, b) => {
            const sA = partySize - a.deathCount
            const sB = partySize - b.deathCount
            if (sA !== sB) return sA - sB
            if (a.partyHpPercent !== b.partyHpPercent) return a.partyHpPercent - b.partyHpPercent
            return a.partyResourcePercent - b.partyResourcePercent
        })
    }, [displayBuckets, partySize])

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas || width === 0) return
        const ctx = canvas.getContext('2d')
        if (!ctx) return

        const dpr = window.devicePixelRatio || 1
        
        canvas.width = width * dpr
        canvas.height = TOTAL_HEIGHT * dpr
        canvas.style.width = `${width}px`
        canvas.style.height = `${TOTAL_HEIGHT}px`
        ctx.scale(dpr, dpr)

        ctx.clearRect(0, 0, width, TOTAL_HEIGHT)

        // Backgrounds
        ctx.fillStyle = '#0a0a0a'
        ctx.fillRect(0, 0, width, BAND_HEIGHT) // Vitality band
        ctx.fillRect(0, BAND_HEIGHT, width, BAND_HEIGHT) // Power band

        const groupWidth = (width - (RUNS_COUNT - 1) * bucketGap) / RUNS_COUNT

        const hoveredBucket = crosshairState.bucketIndex;
        const hoveredCharId = crosshairState.hoveredCharacterId;

        sortedBuckets.forEach((bucket, runIdx) => {
            const groupX = runIdx * (groupWidth + bucketGap)
            const isBucketHovered = hoveredBucket === runIdx + 1;
            const stripeWidth = groupWidth / partySize
            
            sortedPlayers.forEach((playerSlot, playerIdx) => {
                const charData = findCharacterInBucket(bucket.characters, playerSlot.playerId, playerIdx)
                const stripeX = groupX + (playerIdx * stripeWidth)
                const drawWidth = Math.max(0.5, stripeWidth)

                if (!charData) return

                const isCharHovered = hoveredCharId === charData.id;
                const opacity = (hoveredCharId && !isCharHovered && !isBucketHovered) ? 0.3 : 1.0;

                // --- Vitality Band (HP) ---
                if (charData.isDead) {
                    ctx.fillStyle = isBucketHovered ? '#ff0000' : (isCharHovered ? '#ff4d4d' : `rgba(0, 0, 0, ${opacity})`)
                    ctx.fillRect(stripeX, 0, drawWidth, BAND_HEIGHT)
                } else {
                    const hpPct = Math.max(0, Math.min(100, charData.hpPercent)) / 100
                    const hpH = BAND_HEIGHT * hpPct
                    
                    ctx.fillStyle = isBucketHovered ? '#44ff44' : `rgba(34, 197, 94, ${opacity})`
                    ctx.fillRect(stripeX, BAND_HEIGHT - hpH, drawWidth, hpH)
                    
                    ctx.fillStyle = isBucketHovered ? '#ff4444' : `rgba(239, 68, 68, ${opacity})`
                    ctx.fillRect(stripeX, 0, drawWidth, BAND_HEIGHT - hpH)
                }

                // --- Power Band (Resources) ---
                const resPct = Math.max(0, Math.min(100, charData.resourcePercent)) / 100
                const resH = BAND_HEIGHT * resPct
                
                ctx.fillStyle = isBucketHovered ? '#66b2ff' : `rgba(59, 130, 246, ${opacity})`
                ctx.fillRect(stripeX, BAND_HEIGHT, drawWidth, resH)
                
                ctx.fillStyle = isBucketHovered ? '#ffff66' : `rgba(234, 179, 8, ${opacity})`
                ctx.fillRect(stripeX, BAND_HEIGHT + resH, drawWidth, BAND_HEIGHT - resH)
            })

            // Labels
            if (runIdx % 20 === 0 || runIdx === 99) {
                const labelX = groupX + groupWidth / 2
                ctx.fillStyle = isBucketHovered ? '#d4af37' : 'rgba(232, 224, 208, 0.5)'
                ctx.font = isBucketHovered ? 'bold 10px "Courier New", monospace' : '10px "Courier New", monospace'
                ctx.textAlign = 'center'
                ctx.fillText(`P${runIdx === 99 ? 100 : runIdx}`, labelX, TOTAL_HEIGHT - 4)
            }
        })

        // Axis divider lines
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.3)'
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(0, BAND_HEIGHT)
        ctx.lineTo(width, BAND_HEIGHT)
        ctx.stroke()

    }, [sortedBuckets, sortedPlayers, partySize, width, crosshairState.bucketIndex, crosshairState.hoveredCharacterId])

    const handleMouseMove = useCallback((e: React.MouseEvent) => {
        const rect = e.currentTarget.getBoundingClientRect()
        const x = e.clientX - rect.left
        
        // Find bucket
        const bucketWidth = (width - (RUNS_COUNT - 1) * bucketGap) / RUNS_COUNT + bucketGap
        const bucketIdx = Math.floor(x / bucketWidth)
        
        if (bucketIdx >= 0 && bucketIdx < RUNS_COUNT) {
            // Find specific character stripe within bucket
            const stripeX = x % bucketWidth
            let charId: string | null = null
            const groupWidth = bucketWidth - bucketGap
            if (stripeX < groupWidth) {
                const stripeWidth = groupWidth / partySize
                const playerIdx = Math.floor(stripeX / stripeWidth)
                const bucket = sortedBuckets[bucketIdx]
                const playerSlot = sortedPlayers[playerIdx]
                const charData = findCharacterInBucket(bucket.characters, playerSlot?.playerId, playerIdx)
                charId = charData?.id || null
            }

            setCrosshair(bucketIdx + 1, x)
            setHoveredCharacter(charId)
            setMousePos({ x: e.clientX, y: e.clientY })
        } else {
            clearCrosshair()
        }
    }, [partySize, width, setCrosshair, setHoveredCharacter, clearCrosshair, sortedBuckets, sortedPlayers])

    return (
        <div className={`${styles.partyOverview} ${className || ''}`}>
            <div className={styles.header}>
                <h4 className={styles.title}>Survival Spectrogram</h4>
                <div className={styles.subtext}>100 Timelines • Grouped by Player</div>
            </div>

            <div className={styles.legend}>
                <span className={styles.legendLabel}>Cohort:</span>
                <div className={styles.legendGroup}>
                    {sortedPlayers.map((p, i) => {
                        const displayName = playerNames?.get(p.playerId) || p.playerId || `Player ${i + 1}`
                        const isHovered = crosshairState.hoveredCharacterId === p.playerId
                        return (
                            <span 
                                key={`${p.playerId}-${p.position}`} 
                                className={`${styles.playerTag} ${isHovered ? styles.hovered : ''}`}
                                onMouseEnter={() => setHoveredCharacter(p.playerId)}
                                onMouseLeave={() => setHoveredCharacter(null)}
                            >
                                {i === 0 && <span className={styles.roleIcon}>🛡️</span>}
                                {i === sortedPlayers.length - 1 && <span className={styles.roleIcon}>⚡</span>}
                                {displayName}
                            </span>
                        )
                    })}
                </div>
            </div>

            <div 
                ref={containerRef} 
                className={styles.canvasContainer}
                onMouseMove={handleMouseMove}
                onMouseLeave={clearCrosshair}
            >
                <div className={styles.labelVitality}>Vitality</div>
                <div className={styles.labelPower}>Power</div>
                <canvas ref={canvasRef} />
                <CrosshairLine 
                    width={width} 
                    height={TOTAL_HEIGHT} 
                    padding={{ top: 0, right: 0, bottom: 0, left: 0 }}
                />
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

            <CrosshairTooltip 
                bucketData={crosshairState.bucketData}
                x={mousePos.x}
                y={mousePos.y}
            />
        </div>
    )
}

export default PartyOverview