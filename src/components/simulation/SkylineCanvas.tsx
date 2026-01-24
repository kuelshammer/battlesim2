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
} from '@/model/skylineTypes';
import {
    setupCanvas,
    clearCanvas,
    drawPlaceholder,
    generateAnalysisAriaLabel,
    DEFAULT_BACKGROUND_COLOR,
} from '@/components/utils/skylineCanvasUtils';

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

        // Clear canvas with background
        clearCanvas(ctx, config.width!, config.height!, DEFAULT_BACKGROUND_COLOR);

        // TODO: Render implementation will be added by dependent components:
        // - HP Skyline (2xt)
        // - Death Bar overlay (ycg)
        // - Resource Skyline (5sj)
        // - Crosshair interaction (pd0)

        // For now, draw placeholder text
        drawPlaceholder(
            ctx,
            config.width!,
            config.height!,
            `Skyline Canvas Base - ${data.buckets.length} buckets, ${data.partySize} characters`
        );
    }, [data, config]);

    /**
     * Handle mouse move for hover interactions
     */
    const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const x = e.clientX - rect.left;
        // const y = e.clientY - rect.top; // Reserved for future vertical interactions

        // Basic bucket calculation - assumes buckets are evenly distributed
        const bucketWidth = config.width! / data.buckets.length;
        const bucketIndex = Math.floor(x / bucketWidth) + 1; // 1-based indexing

        // For now, don't calculate character hover in base component
        interactionRef.current = {
            hoveredBucket: bucketIndex >= 1 && bucketIndex <= data.buckets.length ? bucketIndex : null,
            hoveredCharacter: null,
        };

        onHover?.(interactionRef.current);
        requestAnimationFrame(render);
    }, [data.buckets.length, config.width, onHover, render]);

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
    const handleClick = useCallback(() => {
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
                aria-label={generateAnalysisAriaLabel('Skyline Spectrogram', data.totalRuns, data.buckets.length)}
            />
        </div>
    );
});

SkylineCanvas.displayName = 'SkylineCanvas';

export default SkylineCanvas;
