/**
 * CrosshairLine - Vertical line overlay synchronized across all charts
 *
 * Renders a vertical dashed line at the hovered bucket position
 * Shows bucket number and median values in tooltip
 */

import React, { memo, useRef, useEffect, useCallback } from 'react';
import { useCrosshair } from './CrosshairContext';
import styles from './crosshairLine.module.scss';

export interface CrosshairLineProps {
    /** Width of the chart (must match parent canvas width) */
    width: number;
    /** Height of the chart (must match parent canvas height) */
    height: number;
    /** Chart identifier (for logging/debugging) */
    chartId?: string;
    /** Left padding (where chart data starts) */
    padding?: { top: number; right: number; bottom: number; left: number };
    className?: string;
}

/**
 * Crosshair line overlay component
 */
const CrosshairLine: React.FC<CrosshairLineProps> = memo(({
    width,
    height,
    chartId = 'unknown',
    padding = { top: 20, right: 10, bottom: 30, left: 40 },
    className,
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const { state } = useCrosshair();

    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = setupCanvas(canvas, width, height);
        if (!ctx) return;

        // Clear entire canvas
        ctx.clearRect(0, 0, width, height);

        // Only render if we have a valid hover position
        if (state.xPosition === null || state.bucketIndex === null) {
            return;
        }

        const chartWidth = width - padding.left - padding.right;
        const x = padding.left + state.xPosition;

        // Draw vertical crosshair line
        ctx.strokeStyle = 'rgba(212, 175, 55, 0.8)';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 4]);

        // Draw line from top to bottom
        ctx.beginPath();
        ctx.moveTo(x, padding.top);
        ctx.lineTo(x, height - padding.bottom);
        ctx.stroke();

        ctx.setLineDash([]);

        // Draw bucket indicator at bottom
        ctx.fillStyle = 'rgba(212, 175, 55, 0.9)';
        ctx.font = 'bold 10px Courier New';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';

        // Background for label
        const label = `P${state.bucketIndex}`;
        const labelWidth = ctx.measureText(label).width + 8;

        ctx.fillStyle = 'rgba(26, 26, 26, 0.9)';
        ctx.fillRect(x - labelWidth / 2, height - padding.bottom, labelWidth, 16);

        // Label text
        ctx.fillStyle = '#d4af37';
        ctx.fillText(label, x, height - padding.bottom + 2);

    }, [state, width, height, padding]);

    useEffect(() => {
        render();
    }, [render]);

    return (
        <div className={`${styles.crosshairOverlay} ${className || ''}`}>
            <canvas
                ref={canvasRef}
                width={width}
                height={height}
                className={styles.crosshairCanvas}
                style={{ pointerEvents: 'none' }}
                aria-hidden="true"
            />
        </div>
    );
});

CrosshairLine.displayName = 'CrosshairLine';

/**
 * Setup canvas with high-DPI scaling
 */
function setupCanvas(canvas: HTMLCanvasElement, width: number, height: number) {
    const dpr = window.devicePixelRatio || 1;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    const ctx = canvas.getContext('2d');
    if (ctx) {
        ctx.scale(dpr, dpr);
    }
    return ctx;
}

/**
 * Crosshair tooltip showing bucket details
 */
export interface CrosshairTooltipProps {
    /** Bucket data to display */
    bucketData: CrosshairState['bucketData'];
    /** Position for tooltip */
    x: number;
    y: number;
    className?: string;
}

export const CrosshairTooltip: React.FC<CrosshairTooltipProps> = memo(({
    bucketData,
    x,
    y,
    className,
}) => {
    if (!bucketData || !bucketData.bucket) {
        return null;
    }

    const { bucket, characters } = bucketData;

    return (
        <div
            className={`${styles.tooltip} ${className || ''}`}
            style={{ left: x, top: y }}
            role="tooltip"
        >
            <div className={styles.tooltipHeader}>
                <span className={styles.percentile}>Percentile {bucket.percentile}</span>
                <span className={styles.deaths}>{bucket.deathCount} deaths</span>
            </div>

            <div className={styles.characterList}>
                {Object.values(characters).map(char => (
                    <div key={char.name} className={styles.characterRow}>
                        <div className={styles.charName}>{char.name}</div>
                        <div className={styles.metrics}>
                            <span className={`${styles.metric} ${styles.hp}`}>
                                HP: {char.hp?.hpPercent.toFixed(0) || 'N/A'}%
                            </span>
                            <span className={`${styles.metric} ${styles.resources}`}>
                                Res: {char.resources?.resourcePercent.toFixed(0) || 'N/A'}%
                            </span>
                        </div>
                        {(char.hp?.deathRound || char.resources?.deathRound) && (
                            <div className={styles.deathRow}>
                                💀 Round {char.hp?.deathRound || char.resources?.deathRound}
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
});

CrosshairTooltip.displayName = 'CrosshairTooltip';

export default CrosshairLine;
