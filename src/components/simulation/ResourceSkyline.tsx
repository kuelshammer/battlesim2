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
    valueToColor,
    DEFAULT_SKYLINE_COLORS,
    SkylineInteractionState,
} from '@/model/skylineTypes';

export interface ResourceSkylineProps {
    /** Full analysis data or single character bucket data */
    data: SkylineAnalysis | CharacterBucketData[];
    onHover?: (state: SkylineInteractionState) => void;
    onBucketClick?: (bucket: any) => void;
    className?: string;
    characterFilter?: string[];
    // Single character mode props
    characterName?: string;
    characterId?: string;
    width?: number;
    height?: number;
    colors?: typeof DEFAULT_SKYLINE_COLORS;
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

    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas || characterData.length === 0) return;

        const ctx = setupCanvas(canvas, width, height);
        if (!ctx) return;

        const padding = { top: 20, right: 10, bottom: 30, left: 40 };
        const chartWidth = width - padding.left - padding.right;
        const chartHeight = height - padding.top - padding.bottom;

        // Clear canvas with dark background
        ctx.fillStyle = 'rgba(26, 26, 26, 0.95)';
        ctx.fillRect(0, 0, width, height);

        // Draw grid lines
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
        ctx.lineWidth = 1;
        [0, 25, 50, 75, 100].forEach(pct => {
            const y = padding.top + chartHeight - (pct / 100) * chartHeight;
            ctx.beginPath();
            ctx.moveTo(padding.left, y);
            ctx.lineTo(width - padding.right, y);
            ctx.stroke();

            // Y-axis labels
            ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
            ctx.font = '10px Courier New';
            ctx.textAlign = 'right';
            ctx.fillText(`${pct}%`, padding.left - 5, y + 3);
        });

        // Draw resource area chart using vertical segments
        const bucketWidth = chartWidth / characterData.length;

        // Create path for area fill
        ctx.beginPath();
        ctx.moveTo(padding.left, padding.top + chartHeight);

        characterData.forEach((bucket, i) => {
            const x = padding.left + i * bucketWidth;
            const resourceHeight = (bucket.resourcePercent / 100) * chartHeight;
            const y = padding.top + chartHeight - resourceHeight;

            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });

        // Complete area and fill
        const lastX = padding.left + (characterData.length - 1) * bucketWidth;
        ctx.lineTo(lastX, padding.top + chartHeight);
        ctx.closePath();

        const avgResource = characterData.reduce((sum, b) => sum + b.resourcePercent, 0) / characterData.length;
        const baseColor = valueToColor(avgResource, colors.resources);

        ctx.fillStyle = baseColor + '40';
        ctx.fill();

        // Draw each bucket segment
        characterData.forEach((bucket, i) => {
            const x = padding.left + i * bucketWidth;
            const resourceHeight = (bucket.resourcePercent / 100) * chartHeight;
            const y = padding.top + chartHeight - resourceHeight;

            const color = valueToColor(bucket.resourcePercent, colors.resources);

            ctx.fillStyle = color;
            ctx.fillRect(x, y, bucketWidth - 1, resourceHeight);

            if (hoveredBucket === i + 1) {
                ctx.strokeStyle = 'rgba(212, 175, 55, 0.8)';
                ctx.lineWidth = 2;
                ctx.strokeRect(x, y, bucketWidth - 1, resourceHeight);
            }
        });

        // Crosshair
        if (hoveredBucket && hoveredBucket >= 1 && hoveredBucket <= characterData.length) {
            const i = hoveredBucket - 1;
            const x = padding.left + i * bucketWidth + bucketWidth / 2;

            ctx.strokeStyle = 'rgba(212, 175, 55, 0.8)';
            ctx.lineWidth = 1;
            ctx.setLineDash([4, 4]);
            ctx.beginPath();
            ctx.moveTo(x, padding.top);
            ctx.lineTo(x, height - padding.bottom);
            ctx.stroke();
            ctx.setLineDash([]);
        }

        // X-axis labels
        ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
        ctx.font = '10px Courier New';
        ctx.textAlign = 'center';

        [1, 50, 100].forEach(pct => {
            const idx = pct - 1;
            if (idx < characterData.length) {
                const x = padding.left + idx * bucketWidth + bucketWidth / 2;
                ctx.fillText(`${pct}%`, x, height - padding.bottom + 15);
            }
        });
    }, [characterData, width, height, colors, hoveredBucket]);

    React.useEffect(() => {
        render();
    }, [render]);

    return (
        <div className={styles.characterCard}>
            <div className={styles.characterName}>{characterName}</div>
            <canvas
                ref={canvasRef}
                width={width}
                height={height}
                className={styles.skylineCanvas}
                onMouseMove={(e) => {
                    const rect = e.currentTarget.getBoundingClientRect();
                    const x = e.clientX - rect.left;
                    const padding = { left: 40, right: 10 };
                    const chartWidth = width - padding.left - padding.right;
                    const bucketWidth = chartWidth / characterData.length;
                    const bucket = Math.floor((x - padding.left) / bucketWidth) + 1;
                    if (bucket >= 1 && bucket <= characterData.length) {
                        onHover?.(bucket);
                    }
                }}
                onMouseLeave={() => onHover?.(null)}
                role="img"
                aria-label={`${characterName} Resource skyline across ${characterData.length} percentile buckets`}
            />
        </div>
    );
});

SingleCharacterResourceSkyline.displayName = 'SingleCharacterResourceSkyline';

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

        const charIds = characterFilter || data.buckets[0].characters.map(c => c.id);

        return charIds.map(charId => {
            const characterData: CharacterBucketData[] = data.buckets.map(bucket => {
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
    }, [data.buckets, characterFilter]);

    const handleBucketHover = useCallback((bucketIndex: number, charId: string) => {
        setHoveredBucket(bucketIndex);
        setHoveredCharId(charId);
        onHover?.({ hoveredBucket: bucketIndex, hoveredCharacter: charId });
    }, [onHover]);

    const handleBucketClick = useCallback((bucket: any) => {
        onBucketClick?.(bucket);
    }, [onBucketClick]);

    const canvasWidth = 200;
    const canvasHeight = 150;

    return (
        <div className={`${styles.resourceSkylineContainer} ${className || ''}`}>
            <div className={styles.skylineTitle}>
                Resource Skyline - {data.totalRuns} runs, {data.buckets.length} buckets
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
