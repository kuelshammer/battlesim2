/**
 * ResourceSkyline - Resource area chart visualization for Skyline Spectrogram UI
 *
 * HEIGHT = % Resources remaining (Y-axis: 0-100%)
 * COLOR = diverging red-blue palette (colorblind-safe)
 * X-axis = 100 buckets (1% worst to 100% best)
 *
 * Resources weighted sum: spell slots * 1.6^level + features + potions
 * Hover shows detailed breakdown per resource type
 */

import React, { memo, useCallback, useMemo } from 'react';
import styles from './resourceSkyline.module.scss';
import {
    SkylineAnalysis,
    CharacterBucketData,
    DEFAULT_SKYLINE_COLORS,
    SkylineInteractionState,
    ColorScale,
    valueToColor,
} from '@/model/skylineTypes';
import { PercentileBucket } from '@/model/model';
import {
    setupCanvas,
    getChartDimensions,
    clearCanvas,
    drawGridLines,
    drawXAxisLabels,
    drawCrosshair,
    drawAreaFill,
    drawVerticalBars,
    drawBucketHighlight,
    createMouseMoveHandler,
    createMouseLeaveHandler,
    generateAriaLabel,
    easeOutQuad,
    lerp,
    DEFAULT_PADDING,
    DEFAULT_BACKGROUND_COLOR,
    type ChartDimensions,
} from '@/components/utils/skylineCanvasUtils';

export interface ResourceSkylineProps {
    /** Full analysis data or single character bucket data */
    data: SkylineAnalysis | CharacterBucketData[];
    onHover?: (state: SkylineInteractionState) => void;
    onBucketClick?: (bucket: PercentileBucket) => void;
    className?: string;
    characterFilter?: string[];
    // Single character mode props
    characterName?: string;
    characterId?: string;
    width?: number;
    height?: number;
    colors?: typeof DEFAULT_SKYLINE_COLORS;
}

export interface SingleCharacterResourceSkylineProps {
    characterData: CharacterBucketData[];
    characterName: string;
    width: number;
    height: number;
    colors: { resources: ColorScale };
    onHover?: (bucket: number | null) => void;
    hoveredBucket?: number | null;
}

/**
 * Render resource skyline for a single character
 */
export const SingleCharacterResourceSkyline: React.FC<SingleCharacterResourceSkylineProps> = memo(({
    characterData,
    characterName,
    width,
    height,
    colors,
    onHover,
    hoveredBucket,
}) => {
    const canvasRef = React.useRef<HTMLCanvasElement>(null);
    const [displayData, setDisplayData] = React.useState(characterData);
    const animationRef = React.useRef<number | null>(null);

    // Smooth data transition
    React.useEffect(() => {
        const startTime = performance.now();
        const duration = 400;
        const startData = displayData;
        const targetData = characterData;

        if (startData.length !== targetData.length) {
            setDisplayData(targetData);
            return;
        }

        const animate = (currentTime: number) => {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            const easedProgress = easeOutQuad(progress);

            if (progress < 1) {
                const interpolated = targetData.map((target, i) => {
                    const start = startData[i];
                    return {
                        ...target,
                        resourcePercent: lerp(start.resourcePercent, target.resourcePercent, easedProgress)
                    };
                });
                setDisplayData(interpolated);
                animationRef.current = requestAnimationFrame(animate);
            } else {
                setDisplayData(targetData);
            }
        };

        animationRef.current = requestAnimationFrame(animate);
        return () => {
            if (animationRef.current) cancelAnimationFrame(animationRef.current);
        };
    }, [characterData]);

    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas || displayData.length === 0) return;

        const ctx = setupCanvas(canvas, width, height);
        if (!ctx) return;

        const dims: ChartDimensions = getChartDimensions(width, height, DEFAULT_PADDING);

        // Clear canvas with background
        clearCanvas(ctx, width, height, DEFAULT_BACKGROUND_COLOR);

        // Draw grid lines
        drawGridLines(ctx, dims);

        // Calculate average resource for area fill color
        const avgResource = displayData.reduce((sum, b) => sum + b.resourcePercent, 0) / displayData.length;
        const baseColor = valueToColor(avgResource, colors.resources);

        // Draw area fill
        drawAreaFill(ctx, dims, displayData.map(d => d.resourcePercent), baseColor + '40');

        // Draw vertical bars with per-bucket colors
        drawVerticalBars(ctx, dims, displayData.map(d => d.resourcePercent), colors.resources);

        // Draw highlights for hovered bucket
        if (hoveredBucket && hoveredBucket >= 1 && hoveredBucket <= displayData.length) {
            const idx = hoveredBucket - 1;
            drawBucketHighlight(
                ctx,
                dims,
                hoveredBucket,
                displayData.length,
                displayData[idx].resourcePercent
            );
            drawCrosshair(ctx, dims, hoveredBucket, displayData.length);
        }

        // Draw X-axis labels
        drawXAxisLabels(ctx, dims, displayData.length);
    }, [displayData, width, height, colors, hoveredBucket]);

    React.useEffect(() => {
        render();
    }, [render]);

    // Create mouse handlers using shared utilities
    const handleMouseMove = createMouseMoveHandler(
        { width, padding: DEFAULT_PADDING, bucketCount: characterData.length },
        (bucket) => {
            if (bucket !== null && bucket >= 1 && bucket <= characterData.length) {
                onHover?.(bucket);
            }
        }
    );

    const handleMouseLeave = createMouseLeaveHandler(onHover);

    return (
        <div className={styles.characterCard}>
            <div className={styles.characterName}>{characterName}</div>
            <canvas
                ref={canvasRef}
                width={width}
                height={height}
                className={styles.skylineCanvas}
                onMouseMove={handleMouseMove}
                onMouseLeave={handleMouseLeave}
                role="img"
                aria-label={generateAriaLabel('Resource', characterData.length, characterName)}
            />
        </div>
    );
});

SingleCharacterResourceSkyline.displayName = 'SingleCharacterResourceSkyline';

/**
 * Main ResourceSkyline component
 */
const ResourceSkyline: React.FC<ResourceSkylineProps> = memo(({
    data,
    onHover,
    onBucketClick,
    className,
    characterFilter,
    characterName,
    characterId,
    width,
    height,
    colors = DEFAULT_SKYLINE_COLORS,
}) => {
    const [hoveredBucket, setHoveredBucket] = React.useState<number | null>(null);
    const [hoveredCharId, setHoveredCharId] = React.useState<string | null>(null);

    // If we are in single character mode (passed from SkylineSpectrogram)
    if (Array.isArray(data) && characterId) {
        return (
            <SingleCharacterResourceSkyline
                characterData={data}
                characterName={characterName || 'Unknown'}
                width={width || 200}
                height={height || 150}
                colors={{ resources: colors }}
                onHover={(bucket) => {
                    setHoveredBucket(bucket);
                    onHover?.({ hoveredBucket: bucket, hoveredCharacter: characterId });
                }}
                hoveredBucket={hoveredBucket}
            />
        );
    }

    // Otherwise, we are in multi-character mode
    const analysisData = data as SkylineAnalysis;

    const characterArrays = useMemo(() => {
        if (!analysisData.buckets || analysisData.buckets.length === 0) return [];

        const charIds = characterFilter || analysisData.buckets[0].characters.map(c => c.id);

        return charIds.map(charId => {
            const characterData: CharacterBucketData[] = analysisData.buckets.map(bucket => {
                const char = bucket.characters.find(c => c.id === charId);
                return char || {
                    name: 'Unknown',
                    id: charId,
                    maxHp: 0,
                    hpPercent: 0,
                    resourcePercent: 0,
                    resourceBreakdown: {
                        spellSlots: [],
                        shortRestFeatures: [],
                        longRestFeatures: [],
                        hitDice: 0,
                        hitDiceMax: 0,
                        totalEhp: 0,
                        maxEhp: 0,
                    },
                    deathRound: null,
                    isDead: false,
                };
            });

            const firstChar = characterData[0];
            return {
                charId,
                characterName: firstChar?.name || charId,
                data: characterData,
            };
        });
    }, [analysisData.buckets, characterFilter]);

    const handleBucketHover = useCallback((bucketIndex: number | null, charId: string) => {
        setHoveredBucket(bucketIndex);
        if (bucketIndex === null) {
            setHoveredCharId(null);
            onHover?.({ hoveredBucket: null, hoveredCharacter: null });
        } else {
            setHoveredCharId(charId);
            onHover?.({ hoveredBucket: bucketIndex, hoveredCharacter: charId });
        }
    }, [onHover]);

    const handleBucketClick = useCallback((bucket: PercentileBucket) => { // eslint-disable-line @typescript-eslint/no-unused-vars
        onBucketClick?.(bucket);
    }, [onBucketClick]);

    const canvasWidth = 200;
    const canvasHeight = 150;

    return (
        <div className={`${styles.resourceSkylineContainer} ${className || ''}`}>
            <div className={styles.skylineTitle}>
                Resource Skyline - {analysisData.totalRuns} runs, {analysisData.buckets.length} buckets
            </div>

            <div className={styles.charactersGrid}>
                {characterArrays.map(({ charId, characterName, data: charData }) => (
                    <SingleCharacterResourceSkyline
                        key={charId}
                        characterData={charData}
                        characterName={characterName}
                        width={canvasWidth}
                        height={canvasHeight}
                        colors={{ resources: DEFAULT_SKYLINE_COLORS }}
                        onHover={(bucket) => handleBucketHover(bucket, charId)}
                        hoveredBucket={hoveredCharId === charId ? hoveredBucket : null}
                    />
                ))}
            </div>
        </div>
    );
});

ResourceSkyline.displayName = 'ResourceSkyline';

export default ResourceSkyline;
