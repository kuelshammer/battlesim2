import { FC, useRef, useEffect } from 'react'
import { SkylineAnalysis, PlayerSlot } from '@/model/model'
import styles from './PartyOverview.module.scss'

interface PartyOverviewProps {
    skyline: SkylineAnalysis
    partySlots: PlayerSlot[]
}

/**
 * PartyOverview displays a horizontal spectrogram of HP and resources across 100 runs.
 *
 * Layout:
 * - X-axis: 100 percentile buckets (P1 left/worst → P100 right/best)
 * - Each bucket is a vertical column:
 *   - Width: player_count pixels (e.g., 4 players = 4px wide)
 *   - Height: 100px (50px above axis for HP, 50px below for resources)
 *   - 1px spacing between columns
 * - ABOVE axis: HP bars stacked by player (1px each)
 *   - Order: Tank (top) → Glass Cannon (bottom)
 *   - Green = near axis (remaining HP), Red = away (damage)
 * - BELOW axis: Resource bars stacked by player (1px each)
 *   - Same player order
 *   - Blue = near axis (remaining), Yellow = away (spent)
 */
const PartyOverview: FC<PartyOverviewProps> = ({ skyline, partySlots }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null)

    const partySize = partySlots.length

    // Calculate canvas dimensions
    // Width: playerSize pixels per bucket * 100 buckets + spacing
    const canvasWidth = partySize * skyline.buckets.length + (skyline.buckets.length - 1)
    const canvasHeight = 100 // Exactly 100px as specified

    console.log('[PartyOverview] Component render:', {
        partySize,
        bucketCount: skyline.buckets.length,
        canvasWidth,
        canvasHeight,
        partySlots: partySlots.map(p => ({ id: p.playerId, score: p.survivabilityScore })),
    })

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return

        const ctx = canvas.getContext('2d')
        if (!ctx) return

        console.log('[PartyOverview] BEFORE sizing:', {
            canvasWidth,
            canvasHeight,
            partySize,
            bucketsLength: skyline.buckets.length,
            partySlotsLength: partySlots.length,
        })

        // Set canvas size (must be set on the element, not just in CSS)
        canvas.width = canvasWidth
        canvas.height = canvasHeight

        // Also set CSS style explicitly to prevent scaling
        canvas.style.width = `${canvasWidth}px`
        canvas.style.height = `${canvasHeight}px`

        // Force a reflow to ensure sizes are applied
        canvas.getBoundingClientRect()

        console.log('[PartyOverview] AFTER sizing:', {
            canvasWidthAttr: canvas.width,
            canvasHeightAttr: canvas.height,
            canvasStyleWidth: canvas.style.width,
            canvasStyleHeight: canvas.style.height,
            clientWidth: canvas.clientWidth,
            clientHeight: canvas.clientHeight,
        })

        // Clear canvas
        ctx.clearRect(0, 0, canvasWidth, canvasHeight)

        const axisY = canvasHeight / 2 // Middle horizontal line at 50px
        const playerHeight = 1 // 1px per player bar
        const columnWidth = partySize // player_count pixels wide per bucket

        console.log('[PartyOverview] Drawing parameters:', {
            axisY,
            playerHeight,
            columnWidth,
            sampleBucket: skyline.buckets[0],
            partySlotsCount: partySlots.length,
        })

        // Fill background to make canvas visible
        ctx.fillStyle = 'rgba(0, 0, 0, 0.5)'
        ctx.fillRect(0, 0, canvasWidth, canvasHeight)

        // Draw axis line
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)'
        ctx.lineWidth = 1
        ctx.beginPath()
        ctx.moveTo(0, axisY)
        ctx.lineTo(canvasWidth, axisY)
        ctx.stroke()

        // Draw each percentile bucket
        skyline.buckets.forEach((bucket, bucketIdx) => {
            const x = bucketIdx * (columnWidth + 1) // +1 for spacing

            // Draw player bars (ordered by survivability: Tank top → Glass Cannon bottom)
            partySlots.forEach((slot, playerIdx) => {
                const character = bucket.characters.find(
                    (c) => c.id === slot.playerId || c.name === slot.playerId
                )

                if (!character) return

                // Y position: above axis for HP, below for resources
                // Players stacked from top (Tank) to bottom (Glass Cannon)
                // Each player gets exactly 1px height
                const hpBarY = axisY - (playerIdx + 1) * playerHeight
                const resBarY = axisY + playerIdx * playerHeight

                // HP bar (above axis) - 1px high, divided horizontally into green/red
                const hpPercent = character.hpPercent
                if (character.isDead) {
                    // Dead = black bar
                    ctx.fillStyle = '#0f172a'
                    ctx.fillRect(x, hpBarY, columnWidth, playerHeight)
                } else {
                    // Green portion (remaining HP) - from left
                    const greenWidth = (hpPercent / 100) * columnWidth
                    ctx.fillStyle = '#22c55e'
                    ctx.fillRect(x, hpBarY, greenWidth, playerHeight)

                    // Red portion (damage taken) - from right
                    const redWidth = columnWidth - greenWidth
                    if (redWidth > 0) {
                        ctx.fillStyle = '#ef4444'
                        ctx.fillRect(x + greenWidth, hpBarY, redWidth, playerHeight)
                    }
                }

                // Resources bar (below axis) - 1px high, divided horizontally into blue/yellow
                const resPercent = character.resourcePercent || 0
                // Blue portion (remaining resources) - from left
                const blueWidth = (resPercent / 100) * columnWidth
                ctx.fillStyle = '#4488ff'
                ctx.fillRect(x, resBarY, blueWidth, playerHeight)

                // Yellow portion (resources spent) - from right
                const yellowWidth = columnWidth - blueWidth
                if (yellowWidth > 0) {
                    ctx.fillStyle = '#ffcc00'
                    ctx.fillRect(x + blueWidth, resBarY, yellowWidth, playerHeight)
                }
            })
        })

        // Draw X-axis labels (every 20 buckets)
        ctx.fillStyle = 'rgba(255, 255, 255, 0.7)'
        ctx.font = '9px sans-serif'
        ctx.textAlign = 'center'
        skyline.buckets.forEach((bucket, idx) => {
            if (bucket.percentile % 20 === 0) {
                const x = idx * (columnWidth + 1) + columnWidth / 2
                ctx.fillText(`P${bucket.percentile}`, x, canvasHeight - 2)
            }
        })

    }, [skyline, partySlots, canvasWidth, canvasHeight, partySize])

    return (
        <div className={styles.partyOverview}>
            <h4 className={styles.title}>Party Overview - HP & Resources (P1 → P100)</h4>

            {/* Player order legend */}
            <div className={styles.playerLegend}>
                <span className={styles.legendLabel}>Top:</span>
                {partySlots.map((slot, idx) => (
                    <span key={slot.playerId} className={styles.playerName}>
                        {slot.playerId}
                        {idx < partySlots.length - 1 && ' → '}
                    </span>
                ))}
                <span className={styles.legendLabel}>:Bottom</span>
            </div>

            {/* Spectrogram canvas */}
            <div className={styles.spectrogramContainer}>
                <canvas
                    ref={canvasRef}
                    width={canvasWidth}
                    height={canvasHeight}
                    className={styles.spectrogram}
                />
            </div>

            {/* Color legend */}
            <div className={styles.colorLegend}>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.green}`} />
                    <span>HP Remaining</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.red}`} />
                    <span>Damage Taken</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.blue}`} />
                    <span>Resources Left</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.yellow}`} />
                    <span>Resources Spent</span>
                </div>
                <div className={styles.legendItem}>
                    <div className={`${styles.legendSwatch} ${styles.black}`} />
                    <span>Deceased</span>
                </div>
            </div>
        </div>
    )
}

export default PartyOverview
