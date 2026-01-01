import { FC, useRef, useEffect } from 'react'
import { SkylineAnalysis, Combattant } from '@/model/model'
import { SkylineCanvasConfig } from '@/model/skylineTypes'
import styles from './SkylineHeatmap.module.scss'

interface SkylineHeatmapProps {
    skyline: SkylineAnalysis
    players: Combattant[]
}

const SkylineHeatmap: FC<SkylineHeatmapProps> = ({ skyline, players }) => {
    const canvasRef = useRef<HTMLCanvasElement>(null)

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return

        const ctx = canvas.getContext('2d')
        if (!ctx) return

        // Clear canvas
        ctx.clearRect(0, 0, canvas.width, canvas.height)

        const { width, height } = canvas
        const padding = { top: 30, right: 20, bottom: 40, left: 50 }
        const chartWidth = width - padding.left - padding.right
        const chartHeight = height - padding.top - padding.bottom

        const numBuckets = skyline.buckets.length
        const bucketWidth = chartWidth / numBuckets
        const barWidth = bucketWidth * 0.35
        const gapWidth = bucketWidth * 0.1

        // Find matching characters for each player
        const playerData = players.map(player => {
            const charData = skyline.buckets.map(bucket =>
                bucket.characters.find(c => c.id === player.creature.id || c.name === player.creature.name)
            ).filter(Boolean)

            return {
                name: player.creature.name,
                data: charData
            }
        }).filter(p => p.data.length > 0)

        if (playerData.length === 0) return

        const playersPerRow = Math.ceil(Math.sqrt(playerData.length))
        const rowHeight = chartHeight / Math.ceil(playerData.length / playersPerRow)

        // Colors
        const colors = {
            green: '#44ff44',   // remaining HP
            red: '#ff4444',     // damage taken
            blue: '#4488ff',    // remaining resources
            yellow: '#ffcc00'   // used resources
        }

        playerData.forEach((player, playerIdx) => {
            const row = Math.floor(playerIdx / playersPerRow)
            const col = playerIdx % playersPerRow

            const offsetX = padding.left + col * (chartWidth / playersPerRow)
            const offsetY = padding.top + row * (rowHeight + 20)

            // Draw player name
            ctx.fillStyle = '#fff'
            ctx.font = 'bold 12px sans-serif'
            ctx.textAlign = 'left'
            ctx.fillText(player.name, offsetX, offsetY - 10)

            const cellWidth = (chartWidth / playersPerRow) - 10
            const cellHeight = rowHeight - 30
            const effectiveBucketWidth = cellWidth / numBuckets
            const effectiveBarWidth = effectiveBucketWidth * 0.35
            const effectiveGapWidth = effectiveBucketWidth * 0.1

            // Draw axes
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)'
            ctx.lineWidth = 1
            ctx.beginPath()
            ctx.moveTo(offsetX, offsetY + cellHeight)
            ctx.lineTo(offsetX + cellWidth, offsetY + cellHeight) // X axis
            ctx.stroke()

            // Draw Y axis labels
            ctx.fillStyle = 'rgba(255, 255, 255, 0.7)'
            ctx.font = '10px sans-serif'
            ctx.textAlign = 'right'
            ctx.fillText('100%', offsetX - 5, offsetY + 5)
            ctx.fillText('50%', offsetX - 5, offsetY + cellHeight / 2 + 3)
            ctx.fillText('0%', offsetX - 5, offsetY + cellHeight)

            player.data.forEach((charData: any, bucketIdx: number) => {
                if (!charData) return

                const x = offsetX + bucketIdx * effectiveBucketWidth

                // HP bar (left bar in each bucket)
                const hpHeight = (charData.hpPercent / 100) * cellHeight
                const damageHeight = cellHeight - hpHeight

                // Draw damage (red, bottom)
                ctx.fillStyle = colors.red
                ctx.fillRect(x, offsetY + cellHeight - damageHeight, effectiveBarWidth, damageHeight)

                // Draw remaining HP (green, top)
                ctx.fillStyle = colors.green
                ctx.fillRect(x, offsetY, effectiveBarWidth, hpHeight)

                // Resources bar (right bar in each bucket)
                const resX = x + effectiveBarWidth + effectiveGapWidth
                const resHeight = (charData.resourcePercent / 100) * cellHeight
                const usedHeight = cellHeight - resHeight

                // Draw used resources (yellow, bottom)
                ctx.fillStyle = colors.yellow
                ctx.fillRect(resX, offsetY + cellHeight - usedHeight, effectiveBarWidth, usedHeight)

                // Draw remaining resources (blue, top)
                ctx.fillStyle = colors.blue
                ctx.fillRect(resX, offsetY, effectiveBarWidth, resHeight)
            })

            // Draw X-axis labels (every 20 buckets)
            ctx.fillStyle = 'rgba(255, 255, 255, 0.7)'
            ctx.font = '9px sans-serif'
            ctx.textAlign = 'center'
            for (let i = 0; i <= numBuckets; i += 20) {
                const x = offsetX + (i - 1) * effectiveBucketWidth + effectiveBucketWidth
                ctx.fillText(`P${i}`, x, offsetY + cellHeight + 12)
            }
        })

        // Draw legend
        const legendY = height - 15
        const legendItems = [
            { color: colors.green, label: 'HP' },
            { color: colors.red, label: 'Damage' },
            { color: colors.blue, label: 'Resources' },
            { color: colors.yellow, label: 'Used' }
        ]

        legendItems.forEach((item, idx) => {
            const x = padding.left + idx * 100
            ctx.fillStyle = item.color
            ctx.fillRect(x, legendY, 12, 12)
            ctx.fillStyle = 'rgba(255, 255, 255, 0.8)'
            ctx.font = '11px sans-serif'
            ctx.textAlign = 'left'
            ctx.fillText(item.label, x + 18, legendY + 10)
        })

    }, [skyline, players])

    const canvasWidth = 1000
    const canvasHeight = 300 + Math.ceil(Math.sqrt(players.length) / 2) * 200

    return (
        <div className={styles.skylineHeatmap}>
            <h4>Skyline Heatmap (All 100 Buckets)</h4>
            <canvas
                ref={canvasRef}
                width={canvasWidth}
                height={canvasHeight}
                className={styles.canvas}
            />
        </div>
    )
}

export default SkylineHeatmap
