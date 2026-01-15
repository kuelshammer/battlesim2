import { FC, useRef, useEffect } from 'react'
import { SkylineAnalysis, Combattant } from '@/model/model'
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

        const { width: canvasWidth } = canvas
        const padding = { top: 30, right: 20, bottom: 40, left: 60 }
        const chartHeight = 200 // Fixed height for the 100px upper + 100px lower

        const numBuckets = skyline.buckets.length
        const chartWidth = canvasWidth - padding.left - padding.right

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

        const playerSectionHeight = chartHeight + 60 // 200 for bars + 60 for labels/spacing
        const playersPerRow = Math.ceil(Math.sqrt(playerData.length))
        const rowHeight = playerSectionHeight

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
            const offsetY = padding.top + row * (rowHeight + 40)

            // Draw player name
            ctx.fillStyle = '#fff'
            ctx.font = 'bold 12px sans-serif'
            ctx.textAlign = 'left'
            ctx.fillText(player.name, offsetX, offsetY - 10)

            const cellWidth = (chartWidth / playersPerRow) - 20
            const effectiveBucketWidth = cellWidth / numBuckets
            const effectiveBarWidth = effectiveBucketWidth * 0.9

            // Draw x-axis line in the middle
            const axisY = offsetY + 100
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.5)'
            ctx.lineWidth = 2
            ctx.beginPath()
            ctx.moveTo(offsetX, axisY)
            ctx.lineTo(offsetX + cellWidth, axisY)
            ctx.stroke()

            // Draw Y axis labels
            ctx.fillStyle = 'rgba(255, 255, 255, 0.7)'
            ctx.font = '9px sans-serif'
            ctx.textAlign = 'right'
            ctx.fillText('100%', offsetX - 5, offsetY + 5) // HP top
            ctx.fillText('50%', offsetX - 5, offsetY + 50) // HP middle
            ctx.fillText('0%', offsetX - 5, axisY + 3) // HP bottom / axis
            ctx.fillText('50%', offsetX - 5, axisY + 53) // Resources middle
            ctx.fillText('100%', offsetX - 5, offsetY + 200) // Resources bottom

            player.data.forEach((charData: any, bucketIdx: number) => {
                if (!charData) return

                const x = offsetX + bucketIdx * effectiveBucketWidth

                // HP bar (upper square, grows UP from axis)
                // Green at bottom (near axis), red at top
                const hpHeight = (charData.hpPercent / 100) * 100
                const damageHeight = 100 - hpHeight

                // Draw green (remaining HP) from axis upward
                ctx.fillStyle = colors.green
                ctx.fillRect(x, axisY - hpHeight, effectiveBarWidth, hpHeight)

                // Draw red (damage taken) above green
                ctx.fillStyle = colors.red
                ctx.fillRect(x, offsetY, effectiveBarWidth, damageHeight)

                // Resources bar (lower square, grows DOWN from axis)
                // Blue near axis, yellow away from axis
                const resHeight = (charData.resourcePercent / 100) * 100
                const usedHeight = 100 - resHeight

                // Draw blue (remaining resources) from axis downward
                ctx.fillStyle = colors.blue
                ctx.fillRect(x, axisY, effectiveBarWidth, resHeight)

                // Draw yellow (used resources) below blue
                ctx.fillStyle = colors.yellow
                ctx.fillRect(x, axisY + resHeight, effectiveBarWidth, usedHeight)
            })

            // Draw X-axis labels (every 20 buckets)
            ctx.fillStyle = 'rgba(255, 255, 255, 0.7)'
            ctx.font = '8px sans-serif'
            ctx.textAlign = 'center'
            for (let i = 0; i <= numBuckets; i += 20) {
                const x = offsetX + (i - 1) * effectiveBucketWidth + effectiveBucketWidth / 2
                ctx.fillText(`P${i}`, x, offsetY + 215)
            }
        })

        // Draw legend
        const legendY = height - 15
        const legendItems = [
            { color: colors.green, label: 'HP (remaining)' },
            { color: colors.red, label: 'Damage taken' },
            { color: colors.blue, label: 'Resources (remaining)' },
            { color: colors.yellow, label: 'Resources used' }
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

    const canvasWidth = 1200 // Wide enough for 100 buckets side-by-side
    const playersPerRow = Math.ceil(Math.sqrt(players.length))
    const canvasHeight = 50 + (playersPerRow * 270) // 200px bars + 60px labels + spacing per row

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
