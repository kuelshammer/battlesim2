import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import FuelGauge from './FuelGauge';
import React from 'react';

describe('FuelGauge Component', () => {
    it('should render planned segments based on weights', () => {
        render(<FuelGauge plannedWeights={[1, 1, 2]} actualCosts={[20, 20, 40]} />);
        // Planned: 25%, 25%, 50%
        // We can check if elements with certain styles or labels exist
        expect(screen.getByText(/The Plan/i)).toBeDefined();
    });

    it('should render actual segments based on costs', () => {
        render(<FuelGauge plannedWeights={[1, 1, 2]} actualCosts={[20, 30, 40]} />);
        // Actual: 20%, 30%, 40% (Total 90%)
        expect(screen.getByText(/The Reality/i)).toBeDefined();
    });

    it('should show "Tank Empty" if total cost > 100%', () => {
        render(<FuelGauge plannedWeights={[1, 1]} actualCosts={[60, 50]} />);
        // Total Actual: 110%
        expect(screen.getByText(/Tank Empty/i)).toBeDefined();
    });

    it('should handle zero total weight', () => {
        render(<FuelGauge plannedWeights={[]} actualCosts={[20]} />);
        expect(screen.getByText(/The Plan/i)).toBeDefined();
    });
});
