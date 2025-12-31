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
import SkylineCanvas from './SkylineCanvas';
import styles from './hpSkyline.module.scss';
import {
    SkylineAnalysis,
    CharacterBucketData,
    valueToColor,
    DEFAULT_SKYLINE_COLORS,
    SkylineInteractionState,
} from '@/model/skylineTypes';

export interface HPSkylineProps {
    data: SkylineAnalysis;
    onHover?: (state: SkylineInteractionState) => void;
    onBucketClick?: (bucket: any) => void;
    className?: string;
    /**
     * Character IDs to display (default: all characters from data)
     */
    characterFilter?: string[];
}

/**
 * Render HP skyline for a single character
 */
interface SingleCharacterSkylineProps {
    characterData: CharacterBucketData[];
    characterName: string;
    width: number;
    height: number;
    colors: typeof DEFAULT_SKYLINE_COLORS;
    onHover?: (bucketIndex: number) => void;
    hoveredBucket: number | null;
}

const SingleCharacterSkyline: React.FC<SingleCharacterSkylineProps> = memo(({
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

        // Draw grid lines (horizontal at 0%, 25%, 50%, 75%, 100%)
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

        // Draw HP area chart using vertical segments
        const bucketWidth = chartWidth / characterData.length;

        // Create path for area fill
        ctx.beginPath();
        ctx.moveTo(padding.left, padding.top + chartHeight); // Start at bottom-left

        characterData.forEach((bucket, i) => {
            const x = padding.left + i * bucketWidth;
            const hpHeight = (bucket.hpPercent / 100) * chartHeight;
            const y = padding.top + chartHeight - hpHeight;

            if (i === 0) {
                ctx.moveTo(x, y);
            } else {
                ctx.lineTo(x, y);
            }
        });

        // Complete the area path
        const lastX = padding.left + (characterData.length - 1) * bucketWidth;
        ctx.lineTo(lastX, padding.top + chartHeight);
        ctx.closePath();

        // Fill area with gradient (based on overall HP trend)
        const avgHp = characterData.reduce((sum, b) => sum + b.hpPercent, 0) / characterData.length;
        const baseColor = valueToColor(avgHp, colors.hp);

        ctx.fillStyle = baseColor + '40'; // 25% opacity
        ctx.fill();

        // Draw each bucket segment with its own color
        characterData.forEach((bucket, i) => {
            const x = padding.left + i * bucketWidth;
            const hpHeight = (bucket.hpPercent / 100) * chartHeight;
            const y = padding.top + chartHeight - hpHeight;

            // Get color for this bucket's HP value
            const color = valueToColor(bucket.hpPercent, colors.hp);

            // Draw vertical bar
            ctx.fillStyle = color;
            ctx.fillRect(x, y, bucketWidth - 1, hpHeight);

            // Highlight hovered bucket
            if (hoveredBucket === i + 1) {
                ctx.strokeStyle = 'rgba(212, 175, 55, 0.8)';
                ctx.lineWidth = 2;
                ctx.strokeRect(x, y, bucketWidth - 1, hpHeight);
            }
        });

        // Draw crosshair line if hovering
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

        // X-axis labels (percentiles at 1%, 50%, 100%)
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

    // Re-render on data/hover change
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
                aria-label={`${characterName} HP skyline across ${characterData.length} percentile buckets`}
            />
        </div>
    );
});

SingleCharacterSkyline.displayName = 'SingleCharacterSkyline';

/**
 * Setup canvas with high-DPI scaling (copied from base component)
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
 * Main HPSkyline component
 */
const HPSkyline: React.FC<HPSkylineProps> = memo(({
    data,
    onHover,
    onBucketClick,
    className,
    characterFilter,
}) => {
    const [hoveredBucket, setHoveredBucket] = React.useState<number | null>(null);
    const [hoveredCharId, setHoveredCharId] = React.useState<string | null>(null);

    // Transform data: buckets -> characters
    const characterArrays = useMemo(() => {
        if (data.buckets.length === 0) return [];

        // Get character IDs from first bucket
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

    const canvasWidth = 200; // Per character
    const canvasHeight = 150;

    return (
        <div className={`${styles.hpSkylineContainer} ${className || ''}`}>
            <div className={styles.skylineTitle}>
                HP Skyline - {data.totalRuns} runs, {data.buckets.length} buckets
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
                        onHover={(bucket) => handleBucketHover(bucket, charId)}
                        hoveredBucket={hoveredCharId === charId ? hoveredBucket : null}
                    />
                ))}
            </div>
        </div>
    );
});

HPSkyline.displayName = 'HPSkyline';

export default HPSkyline;
