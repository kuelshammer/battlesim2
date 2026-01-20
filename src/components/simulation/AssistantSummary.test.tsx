import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import AssistantSummary from './AssistantSummary';
import React from 'react';
import { PacingData } from './pacingUtils';

describe('AssistantSummary Component', () => {
    it('should display "Balanced" message for green status', () => {
        const pacingData: PacingData = {
            actualSegments: [
                { type: 'combat', percent: 25, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            plannedSegments: [
                { type: 'combat', percent: 25, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            vitalitySegments: [
                { type: 'combat', percent: 20, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 20, label: 'Enc 2', id: '2' }
            ],
            powerSegments: [
                { type: 'combat', percent: 15, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 15, label: 'Enc 2', id: '2' }
            ],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 2,
            finalResources: 50,
            finalVitality: 75,
            finalPower: 80,
            actualCosts: [25, 25],
            cumulativeDrifts: [0, 0],
            plannedTimeline: [100, 75, 50],
            vitalityTimeline: [100, 80, 75],
            powerTimeline: [100, 85, 80],
            labels: ['Start', 'Enc 1', 'Enc 2']
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Balanced/i)).toBeDefined();
        expect(screen.getByText(/50%/)).toBeDefined();
    });

    it('should display "Minor Pacing Drift" for yellow status', () => {
        const pacingData: PacingData = {
            actualSegments: [
                { type: 'combat', percent: 32, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            plannedSegments: [
                { type: 'combat', percent: 25, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            vitalitySegments: [
                { type: 'combat', percent: 20, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 20, label: 'Enc 2', id: '2' }
            ],
            powerSegments: [
                { type: 'combat', percent: 15, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 15, label: 'Enc 2', id: '2' }
            ],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 2,
            finalResources: 43,
            finalVitality: 70,
            finalPower: 75,
            actualCosts: [32, 25],
            cumulativeDrifts: [7, 7],
            plannedTimeline: [100, 75, 50],
            vitalityTimeline: [100, 80, 70],
            powerTimeline: [100, 85, 75],
            labels: ['Start', 'Enc 1', 'Enc 2']
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Minor Pacing Drift/i)).toBeDefined();
    });

    it('should display "Overtuned" for delta > 10%', () => {
        const pacingData: PacingData = {
            actualSegments: [
                { type: 'combat', percent: 40, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            plannedSegments: [
                { type: 'combat', percent: 25, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 25, label: 'Enc 2', id: '2' }
            ],
            vitalitySegments: [
                { type: 'combat', percent: 20, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 20, label: 'Enc 2', id: '2' }
            ],
            powerSegments: [
                { type: 'combat', percent: 15, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 15, label: 'Enc 2', id: '2' }
            ],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 2,
            finalResources: 35,
            finalVitality: 60,
            finalPower: 65,
            actualCosts: [40, 25],
            cumulativeDrifts: [15, 15],
            plannedTimeline: [100, 75, 50],
            vitalityTimeline: [100, 80, 60],
            powerTimeline: [100, 85, 65],
            labels: ['Start', 'Enc 1', 'Enc 2']
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Overtuned/i)).toBeDefined();
    });

    it('should display "Impossible Day" for red status', () => {
        const pacingData: PacingData = {
            actualSegments: [
                { type: 'combat', percent: 60, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 50, label: 'Enc 2', id: '2' }
            ],
            plannedSegments: [
                { type: 'combat', percent: 50, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 50, label: 'Enc 2', id: '2' }
            ],
            vitalitySegments: [
                { type: 'combat', percent: 20, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 20, label: 'Enc 2', id: '2' }
            ],
            powerSegments: [
                { type: 'combat', percent: 15, label: 'Enc 1', id: '1' },
                { type: 'combat', percent: 15, label: 'Enc 2', id: '2' }
            ],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 2,
            finalResources: 0,
            finalVitality: 20,
            finalPower: 25,
            actualCosts: [60, 50],
            cumulativeDrifts: [10, 60],
            plannedTimeline: [100, 50, 0],
            vitalityTimeline: [100, 80, 20],
            powerTimeline: [100, 85, 25],
            labels: ['Start', 'Enc 1', 'Enc 2']
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Impossible Day/i)).toBeDefined();
    });
});
