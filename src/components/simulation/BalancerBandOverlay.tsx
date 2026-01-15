/**
 * BalancerBandOverlay - Horizontal band overlay showing target ranges for encounter tiers
 *
 * Shows target HP/resource ranges:
 * - Safe: 70-100% (green band)
 * - Challenging: 50-80% (yellow band)
 * - Boss: 20-60% (red band)
 *
 * Provides visual feedback when bucket values fall outside target range
 */

import React, { memo, useRef, useEffect, useCallback } from 'react';
import styles from './balancerBandOverlay.module.scss';

export interface BalancerBandOverlayProps {
    /** Width of the chart */
    width: number;
    /** Height of the chart */
    height: number;
    /** Chart identifier */
    chartId?: string;
    /** Chart padding */
    padding?: { top: number; right: number; bottom: number; left: number };
    /** Current encounter tier */
    tier: 'Safe' | 'Challenging' | 'Boss' | 'Failed';
    /** Highlight out-of-range buckets */
    showOutOfRange?: boolean;
    /** Bucket data to check against target range */
    bucketData?: { bucketIndex: number; value: number }[] | null;
    className?: string;
}

/**
 * Target ranges for each encounter tier (HP % or resource %)
 */
const TIER_RANGES = {
    Safe: { min: 70, max: 100, color: 'rgba(107, 207, 127, 0.15)', borderColor: 'rgba(107, 207, 127, 0.4)' },
    Challenging: { min: 50, max: 80, color: 'rgba(212, 175, 55, 0.15)', borderColor: 'rgba(212, 175, 55, 0.4)' },
    Boss: { min: 20, max: 60, color: 'rgba(255, 107, 107, 0.15)', borderColor: 'rgba(255, 107, 107, 0.4)' },
    Failed: { min: 0, max: 30, color: 'rgba(150, 150, 150, 0.1)', borderColor: 'rgba(150, 150, 150, 0.3)' },
};

/**
 * Balancer band overlay component
 */
const BalancerBandOverlay: React.FC<BalancerBandOverlayProps> = memo(({
    width,
    height,
    chartId, // eslint-disable-line @typescript-eslint/no-unused-vars
    padding = { top: 20, right: 10, bottom: 30, left: 40 },
    tier,
    showOutOfRange = false,
    bucketData,
    className,
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = setupCanvas(canvas, width, height);
        if (!ctx) return;

        // Clear entire canvas
        ctx.clearRect(0, 0, width, height);

        const chartWidth = width - padding.left - padding.right;
        const chartHeight = height - padding.top - padding.bottom;

        // Get target range for current tier
        const range = TIER_RANGES[tier];

        // Calculate Y positions for the band
        const minY = padding.top + chartHeight - (range.max / 100) * chartHeight;
        const maxY = padding.top + chartHeight - (range.min / 100) * chartHeight;
        const bandHeight = maxY - minY;

        // Draw target band
        ctx.fillStyle = range.color;
        ctx.fillRect(padding.left, minY, chartWidth, bandHeight);

        // Draw band border
        ctx.strokeStyle = range.borderColor;
        ctx.lineWidth = 1;
        ctx.setLineDash([2, 2]);

        // Top border
        ctx.beginPath();
        ctx.moveTo(padding.left, minY);
        ctx.lineTo(padding.left + chartWidth, minY);
        ctx.stroke();

        // Bottom border
        ctx.beginPath();
        ctx.moveTo(padding.left, maxY);
        ctx.lineTo(padding.left + chartWidth, maxY);
        ctx.stroke();

        ctx.setLineDash([]);

        // Draw tier label
        ctx.fillStyle = range.borderColor;
        ctx.font = 'bold 11px Georgia';
        ctx.textAlign = 'left';
        ctx.textBaseline = 'middle';
        ctx.fillText(`${tier} Zone`, padding.left + 5, minY + bandHeight / 2);

        // Draw range labels on Y-axis
        ctx.font = '9px Courier New';
        ctx.fillStyle = 'rgba(232, 224, 208, 0.7)';
        ctx.textAlign = 'right';
        ctx.textBaseline = 'middle';

        // Max value label
        ctx.fillText(`${range.max}%`, padding.left - 5, minY);
        // Min value label
        ctx.fillText(`${range.min}%`, padding.left - 5, maxY);

        // Highlight out-of-range buckets if enabled
        if (showOutOfRange && bucketData && bucketData.length > 0) {
            const bucketWidth = chartWidth / bucketData.length;

            bucketData.forEach(({ bucketIndex, value }) => {
                // Check if value is outside target range
                if (value < range.min || value > range.max) {
                    const x = padding.left + (bucketIndex - 1) * bucketWidth;
                    const y = padding.top + chartHeight - (value / 100) * chartHeight;

                    // Draw warning indicator
                    ctx.fillStyle = 'rgba(255, 107, 107, 0.8)';
                    ctx.beginPath();
                    ctx.arc(x + bucketWidth / 2, y, 3, 0, Math.PI * 2);
                    ctx.fill();
                }
            });
        }

    }, [width, height, padding, tier, showOutOfRange, bucketData]);

    useEffect(() => {
        render();
    }, [render]);

    return (
        <div className={`${styles.bandOverlay} ${className || ''}`}>
            <canvas
                ref={canvasRef}
                width={width}
                height={height}
                className={styles.bandCanvas}
                style={{ pointerEvents: 'none' }}
                aria-hidden="true"
            />
        </div>
    );
});

BalancerBandOverlay.displayName = 'BalancerBandOverlay';

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

export default BalancerBandOverlay;
