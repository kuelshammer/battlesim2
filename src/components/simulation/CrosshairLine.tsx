/**
 * CrosshairLine - Vertical line overlay synchronized across all charts
 *
 * Renders a vertical dashed line at the hovered bucket position
 * Shows bucket number and median values in tooltip
 */

import React, { memo, useRef, useEffect, useCallback } from 'react';
import { useCrosshair, CrosshairState } from './CrosshairContext';
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
 * Crosshair tooltip showing bucket details with full resource breakdown
 */
export interface CrosshairTooltipProps {
    /** Bucket data to display */
    bucketData: CrosshairState['bucketData'];
    /** Position for tooltip */
    x: number;
    y: number;
    className?: string;
}

/**
 * Render spell slot breakdown as compact string
 * e.g., "L1: 2/4, L3: 1/2"
 */
function formatSpellSlots(slots: { level: number; remaining: number; max: number }[] | undefined): string | null {
    if (!slots || slots.length === 0) return null;
    return slots.map(s => `L${s.level}: ${s.remaining}/${s.max}`).join(', ');
}

/**
 * Render feature list as compact string
 */
function formatFeatures(features: string[] | undefined): string | null {
    if (!features || features.length === 0) return null;
    return features.join(', ');
}

export const CrosshairTooltip: React.FC<CrosshairTooltipProps> = memo(({
    bucketData,
    x,
    y,
    className,
}) => {
    const tooltipRef = useRef<HTMLDivElement>(null);
    const [style, setStyle] = useState<React.CSSProperties>({ opacity: 0 });

    useEffect(() => {
        if (!tooltipRef.current || !bucketData) return;

        const padding = 15;
        const tooltipWidth = tooltipRef.current.offsetWidth;
        const tooltipHeight = tooltipRef.current.offsetHeight;
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        let finalX = x + padding;
        let finalY = y + padding;

        // Flip horizontally if overflow
        if (finalX + tooltipWidth > viewportWidth) {
            finalX = x - tooltipWidth - padding;
        }

        // Flip vertically if overflow
        if (finalY + tooltipHeight > viewportHeight) {
            finalY = y - tooltipHeight - padding;
        }

        setStyle({
            left: Math.max(padding, finalX),
            top: Math.max(padding, finalY),
            opacity: 1
        });
    }, [x, y, bucketData]);

    if (!bucketData || !bucketData.bucket) {
        return null;
    }

    const { bucket, characters } = bucketData;

    return (
        <div
            ref={tooltipRef}
            className={`${styles.tooltip} ${className || ''}`}
            style={style}
            role="tooltip"
        >
            <div className={styles.tooltipHeader}>
                <span className={styles.percentile}>Percentile {bucket.percentile}</span>
                <span className={styles.deaths}>{bucket.deathCount} deaths</span>
            </div>

            <div className={styles.characterList}>
                {Object.values(characters).map(char => {
                    const hpData = char.hp;
                    const resData = char.resources;

                    // Extract resource breakdown
                    const breakdown = resData?.resourceBreakdown;

                    return (
                        <div key={char.name} className={styles.characterRow}>
                            <div className={styles.charName}>{char.name}</div>

                            {/* HP and overall resource percentage */}
                            <div className={styles.metrics}>
                                <span className={`${styles.metric} ${styles.hp}`}>
                                    HP: {hpData?.hpPercent.toFixed(0) || 'N/A'}%
                                </span>
                                <span className={`${styles.metric} ${styles.resources}`}>
                                    Res: {resData?.resourcePercent.toFixed(0) || 'N/A'}%
                                </span>
                            </div>

                            {/* Detailed resource breakdown */}
                            {breakdown && (
                                <div className={styles.resourceBreakdown}>
                                    {/* Spell slots */}
                                    {breakdown.spellSlots && breakdown.spellSlots.length > 0 && (
                                        <div className={styles.breakdownRow}>
                                            <span className={styles.breakdownLabel}>Slots:</span>
                                            <span className={styles.breakdownValue}>
                                                {formatSpellSlots(breakdown.spellSlots)}
                                            </span>
                                        </div>
                                    )}

                                    <div className={styles.breakdownGrid}>
                                        {/* Short rest features */}
                                        {breakdown.shortRestFeatures && breakdown.shortRestFeatures.length > 0 && (
                                            <div className={styles.breakdownRow}>
                                                <span className={styles.breakdownLabel}>Short Rest:</span>
                                                <span className={styles.breakdownValue}>
                                                    {formatFeatures(breakdown.shortRestFeatures)}
                                                </span>
                                            </div>
                                        )}

                                        {/* Long rest features */}
                                        {breakdown.longRestFeatures && breakdown.longRestFeatures.length > 0 && (
                                            <div className={styles.breakdownRow}>
                                                <span className={styles.breakdownLabel}>Long Rest:</span>
                                                <span className={styles.breakdownValue}>
                                                    {formatFeatures(breakdown.longRestFeatures)}
                                                </span>
                                            </div>
                                        )}

                                        {/* Hit dice */}
                                        {breakdown.hitDiceMax > 0 && (
                                            <div className={styles.breakdownRow}>
                                                <span className={styles.breakdownLabel}>Hit Dice:</span>
                                                <span className={styles.breakdownValue}>
                                                    {breakdown.hitDice}/{breakdown.hitDiceMax}
                                                </span>
                                            </div>
                                        )}

                                        {/* EHP */}
                                        {breakdown.maxEhp > 0 && (
                                            <div className={styles.breakdownRow}>
                                                <span className={styles.breakdownLabel}>EHP:</span>
                                                <span className={styles.breakdownValue}>
                                                    {breakdown.totalEhp.toFixed(0)}/{breakdown.maxEhp}
                                                </span>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            )}

                            {/* Death info */}
                            {(hpData?.deathRound || resData?.deathRound) && (
                                <div className={styles.deathRow}>
                                    💀 Round {hpData?.deathRound || resData?.deathRound}
                                </div>
                            )}
                        </div>
                    );
                })}
            </div>
        </div>
    );
});

CrosshairTooltip.displayName = 'CrosshairTooltip';

export default CrosshairLine;
