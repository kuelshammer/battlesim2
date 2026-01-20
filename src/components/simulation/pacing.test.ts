import { describe, it, expect } from 'vitest';
import { calculatePacingData } from './pacingUtils';
import { TimelineEvent } from '@/model/model';

describe('calculatePacingData', () => {
    it('should return null if analysis is missing', () => {
        const result = calculatePacingData([], null, []);
        expect(result).toBe(null);
    });

    it('should correctly calculate Grand Total budget and segments', () => {
        // Initial 100, Enc 1 -40, Rest +20, Enc 2 -50
        // Grand Budget = 100 + 20 = 120
        const timeline: TimelineEvent[] = [
            { type: 'combat', id: 'enc1', monsters: [], targetRole: 'Standard' },
            { type: 'shortRest', id: 'rest1' },
            { type: 'combat', id: 'enc2', monsters: [], targetRole: 'Standard' }
        ];

        const analysis = {
            overall: {
                tdnw: 100,
                globalMedian: {
                    // Start 100%, After Enc1 60%, After Rest 80%, After Enc2 30%
                    resourceTimeline: [100, 60, 80, 30],
                    vitalityTimeline: [100, 70, 90, 40],
                    powerTimeline: [100, 50, 50, 10]
                }
            }
        } as any;

        const encounterWeights = [2, 2];

        const result = calculatePacingData(timeline, analysis, encounterWeights);

        expect(result).not.toBe(null);
        if (result) {
            expect(result.grandTotalBudget).toBe(120);
            expect(result.initialEhp).toBe(100);
            expect(result.totalRecovery).toBe(20);

            // Reality segments
            // Enc 1: 40/120 = 33.33%
            // Rest: 0% (Divider)
            // Enc 2: 50/120 = 41.66%
            expect(result.actualSegments[0].percent).toBeCloseTo(33.33, 1);
            expect(result.actualSegments[1].percent).toBe(0);
            expect(result.actualSegments[2].percent).toBeCloseTo(41.66, 1);

            // Vitality Segments (Drops: 30, -20, 50)
            expect(result.vitalitySegments[0].percent).toBe(30);
            expect(result.vitalitySegments[1].percent).toBe(0); // Recovery handled as negative drain? No, createSegments uses Math.max(0, change)
            expect(result.vitalitySegments[2].percent).toBe(50);

            // Power Segments (Drops: 50, 0, 40)
            expect(result.powerSegments[0].percent).toBe(50);
            expect(result.powerSegments[1].percent).toBe(0);
            expect(result.powerSegments[2].percent).toBe(40);

            // Combat-only actual costs
            expect(result.actualCosts[0]).toBeCloseTo(33.33, 1);
            expect(result.actualCosts[1]).toBeCloseTo(41.66, 1);

            // Plan segments (based on weights 2, 2)
            // Total weight 4. Enc 1 = 50%, Enc 2 = 50%
            expect(result.plannedSegments[0].percent).toBe(50);
            expect(result.plannedSegments[1].percent).toBe(0); 
            expect(result.plannedSegments[2].percent).toBe(50);

            // Drifts
            // Enc 1 drift: 33.33 - 50 = -16.66
            // Enc 2 drift: 41.66 - 50 = -8.33
            // Cumulative: [-16.66, -25.0]
            expect(result.cumulativeDrifts[0]).toBeCloseTo(-16.66, 1);
            expect(result.cumulativeDrifts[1]).toBeCloseTo(-25.0, 1);
        }
    });

    it('should handle extreme recoveries correctly', () => {
        const timeline: TimelineEvent[] = [
            { type: 'combat', id: 'enc1', monsters: [], targetRole: 'Standard' },
            { type: 'shortRest', id: 'rest1' }
        ];

        const analysis = {
            overall: {
                tdnw: 100,
                globalMedian: {
                    // Start 50%, After Enc1 20%, After Rest 100%
                    resourceTimeline: [50, 20, 100]
                }
            }
        } as any;

        const result = calculatePacingData(timeline, analysis, [1]);

        expect(result).not.toBe(null);
        if (result) {
            // Initial 50, Recovery 80. Grand Total = 130.
            expect(result.grandTotalBudget).toBe(130);
            expect(result.actualSegments[0].percent).toBeCloseTo(30 / 130 * 100, 1); // 23.07%
        }
    });
});
