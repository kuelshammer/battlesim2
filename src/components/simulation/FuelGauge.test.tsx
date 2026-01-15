import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import FuelGauge from './FuelGauge';
import React from 'react';

describe('FuelGauge Component', () => {
    it('should render planned segments based on weights', () => {
        const pacingData: Record<string, unknown> = {
            plannedSegments: [{ type: 'combat', percent: 25, label: 'Enc 1' }],
            vitalitySegments: [{ type: 'combat', percent: 20, label: 'Enc 1' }],
            powerSegments: [{ type: 'combat', percent: 10, label: 'Enc 1' }]
        };
        render(<FuelGauge pacingData={pacingData} />);
        expect(screen.getByText(/Daily Budget Plan/i)).toBeDefined();
    });

    it('should render vitality and power attrition bars', () => {
        const pacingData: Record<string, unknown> = {
            plannedSegments: [{ type: 'combat', percent: 25, label: 'Enc 1' }],
            vitalitySegments: [{ type: 'combat', percent: 20, label: 'Enc 1' }],
            powerSegments: [{ type: 'combat', percent: 10, label: 'Enc 1' }]
        };
        render(<FuelGauge pacingData={pacingData} />);
        expect(screen.getByText(/Vitality Attrition/i)).toBeDefined();
        expect(screen.getByText(/Power Attrition/i)).toBeDefined();
    });
});