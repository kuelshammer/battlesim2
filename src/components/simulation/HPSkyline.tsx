/**
 * HPSkyline - HP area chart visualization for Skyline Spectrogram UI
 *
 * HEIGHT = % HP remaining (Y-axis: 0-100%)
 * COLOR = diverging red-blue palette (colorblind-safe)
 * X-axis = 100 buckets (1% worst to 100% best)
 *
 * Per-character: 4 side-by-side cards, each with HP skyline
 */

import React, { memo, useCallback, useMemo } from 'react';
import styles from './hpSkyline.module.scss';
import {
    SkylineAnalysis,
    CharacterBucketData,
    valueToColor,
    DEFAULT_SKYLINE_COLORS,
    SkylineInteractionState,
    ColorScale,
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

export interface HPSkylineProps {
    /** Full analysis data or single character bucket data */
    data: SkylineAnalysis | CharacterBucketData[];
    onHover?: (state: SkylineInteractionState) => void;
    onBucketClick?: (bucket: PercentileBucket) => void;
    className?: string;
    /**
     * Character IDs to display (default: all characters from data)
     */
    characterFilter?: string[];
    // Single character mode props
    characterName?: string;
    characterId?: string;
    width?: number;
    height?: number;
    colors?: typeof DEFAULT_SKYLINE_COLORS;
}

export interface SingleCharacterSkylineProps {
    characterData: CharacterBucketData[];
    characterName: string;
    width: number;
    height: number;
    colors: { hp: ColorScale };
    onHover?: (bucket: number | null) => void;
    hoveredBucket?: number | null;
}

/**
 * Render HP skyline for a single character
 */
export const SingleCharacterSkyline: React.FC<SingleCharacterSkylineProps> = memo(({
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
                        hpPercent: lerp(start.hpPercent, target.hpPercent, easedProgress)
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

        // Calculate average HP for area fill color
        const avgHp = displayData.reduce((sum, b) => sum + b.hpPercent, 0) / displayData.length;
        const baseColor = valueToColor(avgHp, colors.hp);

        // Draw area fill
        drawAreaFill(ctx, dims, displayData.map(d => d.hpPercent), baseColor + '40');

        // Draw vertical bars with per-bucket colors
        drawVerticalBars(ctx, dims, displayData.map(d => d.hpPercent), colors.hp);

        // Draw highlights for hovered bucket
        if (hoveredBucket && hoveredBucket >= 1 && hoveredBucket <= displayData.length) {
            const idx = hoveredBucket - 1;
            drawBucketHighlight(
                ctx,
                dims,
                hoveredBucket,
                displayData.length,
                displayData[idx].hpPercent
            );
            drawCrosshair(ctx, dims, hoveredBucket, displayData.length);
        }

        // Draw X-axis labels
        drawXAxisLabels(ctx, dims, displayData.length);
    }, [displayData, width, height, colors, hoveredBucket]);

    // Re-render on data/hover change
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
                aria-label={generateAriaLabel('HP', characterData.length, characterName)}
            />
        </div>
    );
});

SingleCharacterSkyline.displayName = 'SingleCharacterSkyline';

/**
 * Main HPSkyline component
 */
const HPSkyline: React.FC<HPSkylineProps> = memo(({
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
            <SingleCharacterSkyline
                characterData={data}
                characterName={characterName || 'Unknown'}
                width={width || 200}
                height={height || 150}
                colors={{ hp: colors }}
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

    // Transform data: buckets -> characters
    const characterArrays = useMemo(() => {
        if (!analysisData.buckets || analysisData.buckets.length === 0) return [];

        // Get character IDs from first bucket
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

    const handleBucketClick = useCallback((bucket: PercentileBucket) => { // eslint-disable-line @typescript-eslint/no-unused-vars
        onBucketClick?.(bucket);
    }, [onBucketClick]);

    const canvasWidth = 200; // Per character
    const canvasHeight = 150;

    return (
        <div className={`${styles.hpSkylineContainer} ${className || ''}`}>
            <div className={styles.skylineTitle}>
                HP Skyline - {analysisData.totalRuns} runs, {analysisData.buckets.length} buckets
            </div>

            <div className={styles.charactersGrid}>
                {characterArrays.map(({ charId, characterName, data: charData }) => (
                    <SingleCharacterSkyline
                        key={charId}
                        characterData={charData}
                        characterName={characterName}
                        width={canvasWidth}
                        height={canvasHeight}
                        colors={{ hp: DEFAULT_SKYLINE_COLORS }}
                        onHover={(bucket) => {
                            setHoveredBucket(bucket);
                            if (bucket === null) {
                                setHoveredCharId(null);
                                onHover?.({ hoveredBucket: null, hoveredCharacter: null });
                            } else {
                                setHoveredCharId(charId);
                                onHover?.({ hoveredBucket: bucket, hoveredCharacter: charId });
                            }
                        }}
                        hoveredBucket={hoveredCharId === charId ? hoveredBucket : null}
                    />
                ))}
            </div>
        </div>
    );
});

HPSkyline.displayName = 'HPSkyline';

export default HPSkyline;
