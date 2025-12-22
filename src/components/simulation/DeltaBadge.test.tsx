import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import DeltaBadge from './DeltaBadge';
import React from 'react';

describe('DeltaBadge Component', () => {
    it('should display "Major Under" for delta < -10%', () => {
        render(<DeltaBadge targetCost={20} actualCost={5} />);
        // Delta = 5 - 20 = -15%
        expect(screen.getByText(/Undertuned/i)).toBeDefined();
        expect(screen.getByText(/-15%/)).toBeDefined();
    });

    it('should display "Minor Under" for delta between -10% and -5%', () => {
        render(<DeltaBadge targetCost={20} actualCost={12} />);
        // Delta = 12 - 20 = -8%
        expect(screen.getByText(/Slightly Easy/i)).toBeDefined();
        expect(screen.getByText(/-8%/)).toBeDefined();
    });

    it('should display "Perfect" for delta between -5% and +5%', () => {
        render(<DeltaBadge targetCost={20} actualCost={22} />);
        // Delta = 22 - 20 = +2%
        expect(screen.getByText(/On Target/i)).toBeDefined();
        expect(screen.getByText(/\+2%/)).toBeDefined();
    });

    it('should display "Minor Over" for delta between +5% and +10%', () => {
        render(<DeltaBadge targetCost={20} actualCost={28} />);
        // Delta = 28 - 20 = +8%
        expect(screen.getByText(/Minor Drift/i)).toBeDefined();
        expect(screen.getByText(/\+8%/)).toBeDefined();
    });

    it('should display "Major Over" for delta > +10%', () => {
        render(<DeltaBadge targetCost={20} actualCost={35} />);
        // Delta = 35 - 20 = +15%
        expect(screen.getByText(/Overtuned/i)).toBeDefined();
        expect(screen.getByText(/\+15%/)).toBeDefined();
    });

    it('should display cumulative drift if provided', () => {
        render(<DeltaBadge targetCost={20} actualCost={22} cumulativeDrift={4} />);
        expect(screen.getByText(/Total Day Drift: \+4%/i)).toBeDefined();
    });
});
