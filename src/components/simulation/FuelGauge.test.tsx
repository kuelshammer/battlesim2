import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import FuelGauge from './FuelGauge';
import React from 'react';
import { PacingData } from './pacingUtils';

describe('FuelGauge Component', () => {
    it('should render planned segments based on weights', () => {
        const pacingData: PacingData = {
            actualSegments: [{ type: 'combat', percent: 25, label: 'Enc 1', id: '1' }],
            plannedSegments: [{ type: 'combat', percent: 25, label: 'Enc 1', id: '1' }],
            vitalitySegments: [{ type: 'combat', percent: 20, label: 'Enc 1', id: '1' }],
            powerSegments: [{ type: 'combat', percent: 10, label: 'Enc 1', id: '1' }],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 1,
            finalResources: 75,
            finalVitality: 80,
            finalPower: 90,
            actualCosts: [25],
            cumulativeDrifts: [0],
            plannedTimeline: [100, 75],
            vitalityTimeline: [100, 80],
            powerTimeline: [100, 90],
            labels: ['Start', 'Enc 1']
        };
        render(<FuelGauge pacingData={pacingData} />);
        expect(screen.getByText(/Daily Budget Plan/i)).toBeDefined();
    });

    it('should render vitality and power attrition bars', () => {
        const pacingData: PacingData = {
            actualSegments: [{ type: 'combat', percent: 25, label: 'Enc 1', id: '1' }],
            plannedSegments: [{ type: 'combat', percent: 25, label: 'Enc 1', id: '1' }],
            vitalitySegments: [{ type: 'combat', percent: 20, label: 'Enc 1', id: '1' }],
            powerSegments: [{ type: 'combat', percent: 10, label: 'Enc 1', id: '1' }],
            grandTotalBudget: 1000,
            initialEhp: 1000,
            totalRecovery: 0,
            totalWeight: 1,
            finalResources: 75,
            finalVitality: 80,
            finalPower: 90,
            actualCosts: [25],
            cumulativeDrifts: [0],
            plannedTimeline: [100, 75],
            vitalityTimeline: [100, 80],
            powerTimeline: [100, 90],
            labels: ['Start', 'Enc 1']
        };
        render(<FuelGauge pacingData={pacingData} />);
        expect(screen.getByText(/Vitality Attrition/i)).toBeDefined();
        expect(screen.getByText(/Power Attrition/i)).toBeDefined();
    });
});