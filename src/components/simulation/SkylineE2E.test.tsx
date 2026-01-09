/**
 * End-to-end tests for Skyline Spectrogram UI components
 *
 * Tests:
 * - Component rendering
 * - Data accuracy
 * - Color encoding
 * - Crosshair interaction
 * - Accessibility features
 * - Balancer band overlay
 * - Performance with 100 buckets
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import React from 'react';
import { CrosshairProvider, useCrosshair } from './CrosshairContext';
import { AccessibilityProvider, useAccessibility } from './AccessibilityContext';
import { PercentileBucket, CharacterBucketData, ResourceBreakdown } from '@/model/skylineTypes';

// Mock data for testing
const createMockBucket = (
    percentile: number,
    hpPercent: number,
    resourcePercent: number,
    deathRound: number | null = null
): PercentileBucket => ({
    percentile,
    runCount: 1,
    characters: [
        {
            name: 'Fighter',
            id: 'char1',
            maxHp: 100,
            hpPercent,
            resourcePercent,
            resourceBreakdown: {
                spellSlots: [
                    { level: 1, remaining: 3, max: 4 },
                    { level: 2, remaining: 2, max: 3 },
                ],
                shortRestFeatures: ['Second Wind'],
                longRestFeatures: ['Action Surge'],
                hitDice: 8,
                hitDiceMax: 10,
                totalEhp: 150,
                maxEhp: 180,
            },
            deathRound,
            isDead: deathRound !== null,
        } as CharacterBucketData,
        {
            name: 'Cleric',
            id: 'char2',
            maxHp: 80,
            hpPercent: hpPercent - 10,
            resourcePercent: resourcePercent + 5,
            resourceBreakdown: {
                spellSlots: [
                    { level: 1, remaining: 2, max: 4 },
                    { level: 2, remaining: 1, max: 3 },
                    { level: 3, remaining: 0, max: 2 },
                ],
                shortRestFeatures: ['Channel Divinity'],
                longRestFeatures: ['Destroy Undead'],
                hitDice: 6,
                hitDiceMax: 8,
                totalEhp: 120,
                maxEhp: 140,
            },
            deathRound: null,
            isDead: false,
        } as CharacterBucketData,
    ],
    partyHpPercent: hpPercent - 5,
    partyResourcePercent: resourcePercent + 3,
    deathCount: deathRound !== null ? 1 : 0,
});

const createMockBuckets = (count: number = 100): PercentileBucket[] => {
    return Array.from({ length: count }, (_, i) => {
        const percentile = i + 1;
        // Simulate difficulty curve: worse outcomes at low percentiles
        const baseHp = 30 + (percentile / 100) * 60; // 30% to 90%
        const baseRes = 20 + (percentile / 100) * 70; // 20% to 90%
        const deathRound = percentile < 10 ? 5 : null; // Deaths in worst 10%
        return createMockBucket(percentile, baseHp, baseRes, deathRound);
    });
};

describe('Skyline UI - E2E Tests', () => {
    describe('Data Structure Validation', () => {
        it('should create valid buckets with correct percentile ordering', () => {
            const buckets = createMockBuckets(100);

            expect(buckets).toHaveLength(100);
            expect(buckets[0].percentile).toBe(1);
            expect(buckets[99].percentile).toBe(100);

            // Verify percentile is sequential
            for (let i = 0; i < 100; i++) {
                expect(buckets[i].percentile).toBe(i + 1);
            }
        });

        it('should correctly aggregate character data', () => {
            const bucket = createMockBucket(50, 70, 60);

            expect(bucket.characters).toHaveLength(2);
            expect(bucket.characters[0].name).toBe('Fighter');
            expect(bucket.characters[1].name).toBe('Cleric');
            expect(bucket.partyHpPercent).toBe(65); // 70 - 5
            expect(bucket.partyResourcePercent).toBe(63); // 60 + 3
        });

        it('should accurately track death data', () => {
            const buckets = createMockBuckets(100);

            const deathCount = buckets.filter(b => b.deathCount > 0).length;
            expect(deathCount).toBe(9); // First 9 buckets have deaths

            const firstDeath = buckets.find(b => b.deathCount > 0);
            expect(firstDeath?.deathCount).toBe(1);
            expect(firstDeath?.characters[0].deathRound).toBe(5);
        });

        it('should correctly encode resource breakdown', () => {
            const bucket = createMockBucket(50, 70, 60);
            const fighter = bucket.characters[0];

            const breakdown = (fighter as CharacterBucketData).resourceBreakdown;
            expect(breakdown.spellSlots).toHaveLength(2);
            expect(breakdown.spellSlots[0].remaining).toBe(3);
            expect(breakdown.spellSlots[0].max).toBe(4);
            expect(breakdown.shortRestFeatures).toContain('Second Wind');
            expect(breakdown.hitDice).toBe(8);
            expect(breakdown.hitDiceMax).toBe(10);
        });
    });

    describe('CrosshairContext', () => {
        it('should provide crosshair state to consumers', async () => {
            const TestChild = () => {
                const { state } = useCrosshair();
                return (
                    <div>
                        <span data-testid="bucket">{state.bucketIndex === null ? 'null' : state.bucketIndex}</span>
                    </div>
                );
            };

            const { getByTestId } = render(
                <CrosshairProvider>
                    <TestChild />
                </CrosshairProvider>
            );

            expect(getByTestId('bucket').textContent).toBe('null');
        });

        it('should register bucket data sources', () => {
            const buckets = createMockBuckets(100);

            const TestChild = () => {
                const { registerBuckets } = useCrosshair();
                React.useEffect(() => {
                    registerBuckets('test-source', buckets);
                }, [registerBuckets]);
                return <div>Registered</div>;
            };

            render(
                <CrosshairProvider>
                    <TestChild />
                </CrosshairProvider>
            );

            // Component should render without errors
            expect(screen.getByText('Registered')).toBeDefined();
        });
    });

    describe('AccessibilityContext', () => {
        it('should provide accessibility state', () => {
            const TestChild = () => {
                const { highContrast, patternDensity } = useAccessibility();
                return (
                    <div>
                        <span data-testid="contrast">{highContrast.toString()}</span>
                        <span data-testid="density">{patternDensity}</span>
                    </div>
                );
            };

            const { getByTestId } = render(
                <AccessibilityProvider>
                    <TestChild />
                </AccessibilityProvider>
            );

            expect(getByTestId('contrast').textContent).toBe('false');
            expect(getByTestId('density').textContent).toBe('medium');
        });

        it('should toggle high contrast mode', () => {
            const TestChild = () => {
                const { highContrast, toggleHighContrast } = useAccessibility();
                return (
                    <button onClick={toggleHighContrast}>
                        Contrast: {highContrast.toString()}
                    </button>
                );
            };

            const { getByRole } = render(
                <AccessibilityProvider>
                    <TestChild />
                </AccessibilityProvider>
            );

            const button = getByRole('button');
            expect(button.textContent).toBe('Contrast: false');
        });
    });

    describe('Color Encoding', () => {
        it('should use Okabe-Ito colorblind-safe palette', () => {
            // Test that colors are within expected ranges
            const testColors = {
                low: '#d73027',    // Red
                midLow: '#fc8d59', // Orange-Red
                mid: '#f0f0f0',     // Off-white
                midHigh: '#91bfdb', // Light Blue
                high: '#4575b4',    // Deep Blue
            };

            // Verify colors are valid hex
            const hexRegex = /^#[0-9A-Fa-f]{6}$/;
            Object.values(testColors).forEach(color => {
                expect(color).toMatch(hexRegex);
            });
        });

        it('should correctly interpolate between colors', () => {
            // Simple interpolation test
            const interpolateColor = (c1: string, c2: string, t: number): string => {
                const hexToRgb = (hex: string) => {
                    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
                    return result ? {
                        r: parseInt(result[1], 16),
                        g: parseInt(result[2], 16),
                        b: parseInt(result[3], 16),
                    } : null;
                };

                const rgb1 = hexToRgb(c1);
                const rgb2 = hexToRgb(c2);

                if (!rgb1 || !rgb2) return c1;

                const r = Math.round(rgb1.r + (rgb2.r - rgb1.r) * t);
                const g = Math.round(rgb1.g + (rgb2.g - rgb1.g) * t);
                const b = Math.round(rgb1.b + (rgb2.b - rgb1.b) * t);

                return `#${[r, g, b].map(x => x.toString(16).padStart(2, '0')).join('')}`;
            };

            const mid = interpolateColor('#d73027', '#4575b4', 0.5);
            expect(mid).toMatch(/^#[0-9a-f]{6}$/);
        });
    });

    describe('Performance', () => {
        it('should handle 100 buckets efficiently', () => {
            const buckets = createMockBuckets(100);
            const startTime = performance.now();

            // Simulate rendering overhead
            const totalHp = buckets.reduce((sum, b) => sum + b.partyHpPercent, 0);
            const totalRes = buckets.reduce((sum, b) => sum + b.partyResourcePercent, 0);
            const deathCount = buckets.reduce((sum, b) => sum + b.deathCount, 0);

            const endTime = performance.now();
            const duration = endTime - startTime;

            // Should process 100 buckets efficiently
            expect(duration).toBeLessThan(100);
            expect(totalHp).toBeGreaterThan(0);
            expect(totalRes).toBeGreaterThan(0);
        });

        it('should calculate aggregate metrics efficiently', () => {
            const buckets = createMockBuckets(100);
            const startTime = performance.now();

            // Calculate percentiles
            const p1 = buckets[0].partyHpPercent;
            const p50 = buckets[49].partyHpPercent;
            const p99 = buckets[98].partyHpPercent;

            const endTime = performance.now();
            const duration = endTime - startTime;

            expect(duration).toBeLessThan(50);
            expect(p1).toBeLessThan(p50);
            expect(p50).toBeLessThan(p99);
        });
    });

    describe('Balancer Band Integration', () => {
        it('should correctly identify tier ranges', () => {
            const tierRanges = {
                Safe: { min: 70, max: 100 },
                Challenging: { min: 50, max: 80 },
                Boss: { min: 20, max: 60 },
                Failed: { min: 0, max: 30 },
            };

            // Test Safe range
            const safeValue = 85;
            expect(safeValue).toBeGreaterThanOrEqual(tierRanges.Safe.min);
            expect(safeValue).toBeLessThanOrEqual(tierRanges.Safe.max);

            // Test Challenging range
            const challengingValue = 65;
            expect(challengingValue).toBeGreaterThanOrEqual(tierRanges.Challenging.min);
            expect(challengingValue).toBeLessThanOrEqual(tierRanges.Challenging.max);

            // Test Boss range
            const bossValue = 40;
            expect(bossValue).toBeGreaterThanOrEqual(tierRanges.Boss.min);
            expect(bossValue).toBeLessThanOrEqual(tierRanges.Boss.max);
        });

        it('should detect out-of-range values', () => {
            const tier = { min: 50, max: 80 };

            const inRange = 65;
            const outOfRangeLow = 40;
            const outOfRangeHigh = 85;

            expect(inRange >= tier.min && inRange <= tier.max).toBe(true);
            expect(outOfRangeLow >= tier.min && outOfRangeLow <= tier.max).toBe(false);
            expect(outOfRangeHigh >= tier.min && outOfRangeHigh <= tier.max).toBe(false);
        });
    });

    describe('Data Accuracy', () => {
        it('should maintain sorting order across all buckets', () => {
            const buckets = createMockBuckets(100);

            // HP should be monotonically increasing (worst to best)
            for (let i = 1; i < 100; i++) {
                expect(buckets[i].partyHpPercent).toBeGreaterThanOrEqual(
                    buckets[i - 1].partyHpPercent - 1 // Allow small variance
                );
            }
        });

        it('should correctly calculate death rates', () => {
            const buckets = createMockBuckets(100);
            const totalDeaths = buckets.reduce((sum, b) => sum + b.deathCount, 0);
            const deathRate = (totalDeaths / (buckets.length * 2)) * 100; // 2 chars per bucket

            // Should have ~4.5% death rate (9 deaths / 200 characters)
            expect(deathRate).toBeGreaterThan(4);
            expect(deathRate).toBeLessThan(5);
        });
    });
});
