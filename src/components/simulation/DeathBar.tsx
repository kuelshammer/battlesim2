/**
 * DeathBar - 10px overlay strip showing deaths per percentile bucket
 *
 * Displays skull glyphs (💀) at X-coordinates where deaths occurred
 * Shows death round number on hover
 */

import React, { memo, useCallback, useMemo } from 'react';
import styles from './deathBar.module.scss';
import {
    SkylineAnalysis,
    CharacterBucketData,
} from '@/model/skylineTypes';

export interface DeathBarProps {
    data: SkylineAnalysis;
    width: number;
    characterFilter?: string[];
    onHover?: (bucketIndex: number, characterId: string, deathRound: number | null) => void;
    className?: string;
}

/**
 * Single character death bar
 */
interface CharacterDeathBarProps {
    characterData: CharacterBucketData[];
    characterName: string;
    width: number;
    onHover?: (bucketIndex: number, characterId: string, deathRound: number | null) => void;
}

const CharacterDeathBar: React.FC<CharacterDeathBarProps> = memo(({
    characterData,
    characterName,
    width,
    onHover,
}) => {
    const canvasRef = React.useRef<HTMLCanvasElement>(null);
    const [hoveredBucket, setHoveredBucket] = React.useState<number | null>(null);

    const render = useCallback(() => {
        const canvas = canvasRef.current;
        if (!canvas || characterData.length === 0) return;

        const ctx = setupCanvas(canvas, width, 20); // 20px height = 10px bar + padding
        if (!ctx) return;

        // Clear with transparent background
        ctx.clearRect(0, 0, width, 20);

        const barHeight = 10;
        const padding = { left: 40, right: 10 };
        const chartWidth = width - padding.left - padding.right;
        const bucketWidth = chartWidth / characterData.length;

        // Draw bar background
        ctx.fillStyle = 'rgba(26, 26, 26, 0.8)';
        ctx.fillRect(padding.left, 5, chartWidth, barHeight);

        // Find all buckets where death occurred
        characterData.forEach((bucket, i) => {
            if (bucket.isDead && bucket.deathRound !== null) {
                const x = padding.left + i * bucketWidth;
                const centerX = x + bucketWidth / 2;

                // Draw skull glyph (💀 as text)
                ctx.fillStyle = '#ff6b6b';
                ctx.font = '10px sans-serif';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                ctx.fillText('💀', centerX, 5 + barHeight / 2);

                // Highlight hovered death
                if (hoveredBucket === i + 1) {
                    ctx.strokeStyle = 'rgba(255, 107, 107, 0.8)';
                    ctx.lineWidth = 1;
                    ctx.strokeRect(x, 5, bucketWidth - 1, barHeight);
                }
            }
        });

        // Draw death count label on left
        const deathCount = characterData.filter(b => b.isDead).length;
        ctx.fillStyle = 'rgba(255, 255, 255, 0.6)';
        ctx.font = '9px Courier New';
        ctx.textAlign = 'right';
        ctx.fillText(`${deathCount}💀`, padding.left - 5, 5 + barHeight / 2 + 2);
    }, [characterData, width, hoveredBucket]);

    React.useEffect(() => {
        render();
    }, [render]);

    const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const padding = { left: 40, right: 10 };
        const chartWidth = width - padding.left - padding.right;
        const bucketWidth = chartWidth / characterData.length;

        const bucket = Math.floor((x - padding.left) / bucketWidth) + 1;
        if (bucket >= 1 && bucket <= characterData.length) {
            setHoveredBucket(bucket);
            const charData = characterData[bucket - 1];
            onHover?.(bucket, characterName, charData.deathRound);
        }
    }, [characterData, width, characterName, onHover]);

    const handleMouseLeave = useCallback(() => {
        setHoveredBucket(null);
        onHover?.(null, characterName, null);
    }, [characterName, onHover]);

    return (
        <div className={styles.deathBarContainer}>
            <div className={styles.characterName}>{characterName}</div>
            <canvas
                ref={canvasRef}
                width={width}
                height={20}
                className={styles.deathBar}
                onMouseMove={handleMouseMove}
                onMouseLeave={handleMouseLeave}
                role="img"
                aria-label={`${characterName} death bar showing deaths across ${characterData.length} buckets`}
            />
        </div>
    );
});

CharacterDeathBar.displayName = 'CharacterDeathBar';

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
 * Main DeathBar component
 */
const DeathBar: React.FC<DeathBarProps> = memo(({
    data,
    width,
    characterFilter,
    onHover,
    className,
}) => {
    const characterArrays = useMemo(() => {
        if (data.buckets.length === 0) return [];

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

    return (
        <div className={`${styles.deathBarGrid} ${className || ''}`}>
            {characterArrays.map(({ charId, characterName, data: charData }) => (
                <CharacterDeathBar
                    key={charId}
                    characterData={charData}
                    characterName={characterName}
                    width={width}
                    onHover={onHover}
                />
            ))}
        </div>
    );
});

DeathBar.displayName = 'DeathBar';

export default DeathBar;
