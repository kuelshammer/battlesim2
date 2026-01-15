/**
 * SkylineSpectrogram - Main container for Skyline visualization
 *
 * Assembles:
 * - 4 HPSkyline charts (one per character)
 * - 4 ResourceSkyline charts (one per character)
 * - 4 DeathBar overlays
 * - CrosshairLine (synchronized across all charts)
 * - BalancerBandOverlay (target zones)
 * - AccessibilityToggle (high-contrast mode)
 */

import React, { memo, useMemo } from 'react';
import { CrosshairProvider, useCrosshairBucketRegistration } from './CrosshairContext';
import { AccessibilityProvider, useAccessibility } from './AccessibilityContext';
import HPSkyline from './HPSkyline';
import ResourceSkyline from './ResourceSkyline';
import DeathBar from './DeathBar';
import CrosshairLine from './CrosshairLine';
import BalancerBandOverlay from './BalancerBandOverlay';
import AccessibilityToggle from './AccessibilityToggle';
import { SkylineAnalysis } from '@/model/skylineTypes';
import styles from './skylineSpectrogram.module.scss';

export interface SkylineSpectrogramProps {
    /** Skyline data from WASM analysis */
    data: SkylineAnalysis;
    /** Chart width (default: 600px) */
    width?: number;
    /** Chart height (default: 200px) */
    height?: number;
    /** Current encounter tier for balancer overlay */
    encounterTier?: 'Safe' | 'Challenging' | 'Boss' | 'Failed';
    /** Show balancer band overlay */
    showBalancerBand?: boolean;
    /** Show accessibility controls */
    showAccessibilityControls?: boolean;
    /** Additional className */
    className?: string;
}

/**
 * Inner component with providers
 */
const SkylineSpectrogramInner: React.FC<SkylineSpectrogramProps> = memo(({
    data,
    width = 600,
    height = 200,
    encounterTier = 'Challenging',
    showBalancerBand = true,
    showAccessibilityControls = true,
    className,
}) => {
    const { highContrast } = useAccessibility();

    // Register bucket data for crosshair tooltips
    useCrosshairBucketRegistration('skyline-main', data.buckets);

    // Calculate chart dimensions
    const chartWidth = width;
    const chartHeight = height;

    // Extract character data
    const characters = useMemo(() => {
        if (data.buckets.length === 0) return [];
        const firstBucket = data.buckets[0];
        return firstBucket.characters;
    }, [data.buckets]);

    // Transform data for each character
    const characterHPData = useMemo(() => {
        return characters.map(char => {
            return data.buckets.map(bucket => {
                const charData = bucket.characters.find(c => c.id === char.id);
                return charData || {
                    name: char.name,
                    id: char.id,
                    maxHp: char.maxHp || 100,
                    hpPercent: 100,
                    resourcePercent: 100,
                    resourceBreakdown: char.resourceBreakdown,
                    deathRound: null,
                    isDead: false,
                };
            });
        });
    }, [data.buckets, characters]);

    const characterResourceData = useMemo(() => {
        return characters.map(char => {
            return data.buckets.map(bucket => {
                const charData = bucket.characters.find(c => c.id === char.id);
                return charData || {
                    name: char.name,
                    id: char.id,
                    maxHp: char.maxHp || 100,
                    hpPercent: 100,
                    resourcePercent: 100,
                    resourceBreakdown: char.resourceBreakdown,
                    deathRound: null,
                    isDead: false,
                };
            });
        });
    }, [data.buckets, characters]);

    // Default colors (Okabe-Ito palette)
    const colors = {
        hp: {
            low: '#d73027',     // Red (0%)
            midLow: '#fc8d59',  // Orange-Red (25%)
            mid: '#f0f0f0',     // Off-white (50%)
            midHigh: '#91bfdb', // Light Blue (75%)
            high: '#4575b4',    // Deep Blue (100%)
        },
        resources: {
            low: '#d73027',
            midLow: '#fc8d59',
            mid: '#f0f0f0',
            midHigh: '#91bfdb',
            high: '#4575b4',
        },
    };

    return (
        <div className={`${styles.skylineSpectrogram} ${highContrast ? styles.highContrast : ''} ${className || ''}`}>
            {/* Header with controls */}
            <div className={styles.header}>
                <h3 className={styles.title}>
                    {data.encounterIndex !== null
                        ? `Encounter ${data.encounterIndex + 1} Skyline`
                        : 'Skyline Analysis'}
                </h3>
                <span className={styles.subtitle}>
                    {data.totalRuns} iterations · {data.partySize} characters
                </span>
            </div>

            {/* Accessibility controls */}
            {showAccessibilityControls && (
                <div className={styles.accessibilitySection}>
                    <AccessibilityToggle />
                </div>
            )}

            {/* Charts grid */}
            <div className={styles.chartsGrid}>
                {characters.map((char, idx) => (
                    <div key={char.id} className={styles.characterColumn}>
                        {/* Character name */}
                        <div className={styles.characterName}>{char.name}</div>

                        {/* HP Skyline */}
                        <div className={styles.chartRow}>
                            <span className={styles.chartLabel}>HP</span>
                            <div className={styles.chartWrapper}>
                                <HPSkyline
                                    data={characterHPData[idx]}
                                    characterName={char.name}
                                    characterId={char.id}
                                    width={chartWidth}
                                    height={chartHeight / 2 - 10}
                                    colors={colors.hp}
                                />
                                <DeathBar
                                    data={characterHPData[idx]}
                                    width={chartWidth}
                                />
                                {showBalancerBand && (
                                    <BalancerBandOverlay
                                        width={chartWidth}
                                        height={chartHeight / 2 - 10}
                                        tier={encounterTier}
                                        showOutOfRange={true}
                                        bucketData={data.buckets.map((b, i) => ({
                                            bucketIndex: i + 1,
                                            value: b.characters.find(c => c.id === char.id)?.hpPercent || 100,
                                        }))}
                                    />
                                )}
                                <CrosshairLine
                                    width={chartWidth}
                                    height={chartHeight / 2 - 10}
                                    chartId={`hp-${char.id}`}
                                />
                            </div>
                        </div>

                        {/* Resource Skyline */}
                        <div className={styles.chartRow}>
                            <span className={styles.chartLabel}>RES</span>
                            <div className={styles.chartWrapper}>
                                <ResourceSkyline
                                    data={characterResourceData[idx]}
                                    characterName={char.name}
                                    characterId={char.id}
                                    width={chartWidth}
                                    height={chartHeight / 2 - 10}
                                    colors={colors.resources}
                                />
                                <DeathBar
                                    data={characterResourceData[idx]}
                                    width={chartWidth}
                                />
                                {showBalancerBand && (
                                    <BalancerBandOverlay
                                        width={chartWidth}
                                        height={chartHeight / 2 - 10}
                                        tier={encounterTier}
                                        showOutOfRange={true}
                                        bucketData={data.buckets.map((b, i) => ({
                                            bucketIndex: i + 1,
                                            value: b.characters.find(c => c.id === char.id)?.resourcePercent || 100,
                                        }))}
                                    />
                                )}
                                <CrosshairLine
                                    width={chartWidth}
                                    height={chartHeight / 2 - 10}
                                    chartId={`res-${char.id}`}
                                />
                            </div>
                        </div>
                    </div>
                ))}
            </div>

            {/* Legend */}
            <div className={styles.legend}>
                <span className={styles.legendItem}>
                    <span className={`${styles.legendColor} ${styles.legendLow}`}></span>
                    0% (Depleted)
                </span>
                <span className={styles.legendItem}>
                    <span className={`${styles.legendColor} ${styles.legendMid}`}></span>
                    50% (Half)
                </span>
                <span className={styles.legendItem}>
                    <span className={`${styles.legendColor} ${styles.legendHigh}`}></span>
                    100% (Full)
                </span>
                {highContrast && (
                    <span className={styles.legendItem}>
                        <span className={styles.legendPattern}>▦</span>
                        Pattern Overlay
                    </span>
                )}
            </div>
        </div>
    );
});

SkylineSpectrogramInner.displayName = 'SkylineSpectrogramInner';

/**
 * SkylineSpectrogram container with providers
 */
const SkylineSpectrogram: React.FC<SkylineSpectrogramProps> = memo((props) => {
    return (
        <CrosshairProvider>
            <AccessibilityProvider initialHighContrast={false}>
                <SkylineSpectrogramInner {...props} />
            </AccessibilityProvider>
        </CrosshairProvider>
    );
});

SkylineSpectrogram.displayName = 'SkylineSpectrogram';

export default SkylineSpectrogram;
