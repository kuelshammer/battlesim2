/**
 * SkylineCanvas - Base Canvas component for Skyline Spectrogram UI
 *
 * Handles:
 * - High-DPI (Retina) scaling
 * - Canvas rendering lifecycle
 * - Mouse interaction (hover, crosshair)
 * - Performance optimization
 */

import React, { useRef, useEffect, memo, useCallback } from 'react';
import styles from './skylineCanvas.module.scss';
import {
    SkylineCanvasProps,
    DEFAULT_SKYLINE_COLORS,
    SkylineInteractionState,
    valueToColor,
} from '@/model/skylineTypes';

const DEFAULT_CONFIG: SkylineCanvasProps['config'] = {
    width: 800,
    height: 400,
    padding: { top: 20, right: 20, bottom: 40, left: 60 },
    colors: {
        hp: DEFAULT_SKYLINE_COLORS,
        resources: DEFAULT_SKYLINE_COLORS,
        death: '#ff4444',
        grid: 'rgba(255, 255, 255, 0.1)',
        crosshair: 'rgba(212, 175, 55, 0.8)',
    },
};

/**
 * Get high-DPI scaled canvas dimensions
 */
function getCanvasSize(canvas: HTMLCanvasElement, width: number, height: number) {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();

    return {
        // Display size (CSS pixels)
        displayWidth: width,
        displayHeight: height,
        // Actual canvas size (scaled for DPI)
        canvasWidth: width * dpr,
        canvasHeight: height * dpr,
        pixelRatio: dpr,
    };
}

/**
 * Setup canvas with high-DPI scaling
 */
function setupCanvas(canvas: HTMLCanvasElement, width: number, height: number) {
    const dpr = window.devicePixelRatio || 1;

    // Set display size (CSS pixels)
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    // Set actual canvas size (scaled)
    canvas.width = width * dpr;
    canvas.height = height * dpr;

    // Normalize coordinate system
    const ctx = canvas.getContext('2d');
    if (ctx) {
        ctx.scale(dpr, dpr);
    }

    return ctx;
}

const SkylineCanvas: React.FC<SkylineCanvasProps> = memo(({
    data,
    config: partialConfig,
    onHover,
    onBucketClick,
    className,
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const interactionRef = useRef<SkylineInteractionState>({
        hoveredBucket: null,
        hoveredCharacter: null,
    });

    const config = { ...DEFAULT_CONFIG, ...partialConfig };

    /**
     * Main render function - draws the Skyline visualization
     */
    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = setupCanvas(canvas, config.width!, config.height!);
        if (!ctx) return;

        // Clear canvas
        ctx.clearRect(0, 0, config.width!, config.height!);

        // Draw background
        ctx.fillStyle = 'rgba(26, 26, 26, 0.95)';
        ctx.fillRect(0, 0, config.width!, config.height!);

        // TODO: Render implementation will be added by dependent components:
        // - HP Skyline (2xt)
        // - Death Bar overlay (ycg)
        // - Resource Skyline (5sj)
        // - Crosshair interaction (pd0)

        // For now, draw placeholder text
        ctx.fillStyle = 'rgba(212, 175, 55, 0.5)';
        ctx.font = '14px Courier New';
        ctx.textAlign = 'center';
        ctx.fillText(
            `Skyline Canvas Base - ${data.buckets.length} buckets, ${data.partySize} characters`,
            config.width! / 2,
            config.height! / 2
        );
    }, [data, config]);

    /**
     * Handle mouse move for hover/crosshair
     */
    const handleMouseMove = useCallback((event: React.MouseEvent<HTMLCanvasElement>) => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;

        // TODO: Calculate bucket from X position
        // const bucket = Math.floor((x - padding.left) / availableWidth * 100) + 1;

        // interactionRef.current.hoveredBucket = bucket;
        // onHover?.(interactionRef.current);

        // Trigger re-render for crosshair
        // requestAnimationFrame(render);
    }, [config, onHover]);

    /**
     * Handle mouse leave
     */
    const handleMouseLeave = useCallback(() => {
        interactionRef.current = {
            hoveredBucket: null,
            hoveredCharacter: null,
        };
        onHover?.(interactionRef.current);
        requestAnimationFrame(render);
    }, [onHover, render]);

    /**
     * Handle click for bucket selection
     */
    const handleClick = useCallback((event: React.MouseEvent<HTMLCanvasElement>) => {
        const bucketIndex = interactionRef.current.hoveredBucket;
        if (bucketIndex && bucketIndex >= 1 && bucketIndex <= data.buckets.length) {
            onBucketClick?.(data.buckets[bucketIndex - 1]);
        }
    }, [data.buckets, onBucketClick]);

    // Initial render and re-render on data/config change
    useEffect(() => {
        render();
    }, [render]);

    // Handle window resize
    useEffect(() => {
        const handleResize = () => {
            render();
        };

        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, [render]);

    return (
        <div className={`${styles.canvasContainer} ${className || ''}`}>
            <canvas
                ref={canvasRef}
                width={config.width}
                height={config.height}
                className={styles.canvas}
                onMouseMove={handleMouseMove}
                onMouseLeave={handleMouseLeave}
                onClick={handleClick}
                role="img"
                aria-label={`Skyline Spectrogram showing ${data.totalRuns} simulation runs across ${data.buckets.length} percentile buckets`}
            />
        </div>
    );
});

SkylineCanvas.displayName = 'SkylineCanvas';

export default SkylineCanvas;
