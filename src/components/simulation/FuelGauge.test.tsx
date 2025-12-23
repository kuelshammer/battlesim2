import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import FuelGauge from './FuelGauge';
import React from 'react';

describe('FuelGauge Component', () => {
    it('should render planned segments based on weights', () => {
        const planned: any[] = [{ type: 'combat', percent: 25, label: 'Enc 1' }];
        const actual: any[] = [{ type: 'combat', percent: 20, label: 'Enc 1' }];
        render(<FuelGauge plannedSegments={planned} actualSegments={actual} />);
        expect(screen.getByText(/The Plan/i)).toBeDefined();
        expect(screen.getByText(/Enc 1: 25%/i)).toBeDefined();
    });

    it('should render actual segments based on costs', () => {
        const planned: any[] = [{ type: 'combat', percent: 25, label: 'Enc 1' }];
        const actual: any[] = [{ type: 'combat', percent: 20, label: 'Enc 1' }];
        render(<FuelGauge plannedSegments={planned} actualSegments={actual} />);
        expect(screen.getByText(/The Reality/i)).toBeDefined();
        expect(screen.getByText(/Enc 1: 20%/i)).toBeDefined();
    });

    it('should show "Tank Overdrawn" if total cost > 100%', () => {
        const planned: any[] = [{ type: 'combat', percent: 50 }];
        const actual: any[] = [{ type: 'combat', percent: 60 }, { type: 'combat', percent: 50 }];
        render(<FuelGauge plannedSegments={planned} actualSegments={actual} />);
        expect(screen.getByText(/Tank Overdrawn/i)).toBeDefined();
    });

    it('should handle zero total weight', () => {
        render(<FuelGauge plannedSegments={[]} actualSegments={[]} />);
        expect(screen.getByText(/The Plan/i)).toBeDefined();
    });
});
