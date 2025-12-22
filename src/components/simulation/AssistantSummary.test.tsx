import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import AssistantSummary from './AssistantSummary';
import React from 'react';

describe('AssistantSummary Component', () => {
    it('should display "Balanced" message for green status', () => {
        // Target: 25% each (Total 50%)
        render(<AssistantSummary actualCosts={[25, 25]} targetWeights={[1, 1, 2]} finalResources={50} />);
        expect(screen.getByText(/Balanced/i)).toBeDefined();
        expect(screen.getByText(/50%/)).toBeDefined();
    });

    it('should display "Minor Pacing Drift" for yellow status', () => {
        // Target: 25% each. Actual: 32% (Delta +7)
        render(<AssistantSummary actualCosts={[32, 25]} targetWeights={[1, 1, 2]} finalResources={43} />);
        expect(screen.getByText(/Minor Pacing Drift/i)).toBeDefined();
    });

    it('should display "Overtuned" for delta > 10%', () => {
        // Target: 25% each. Actual: 40% (Delta +15)
        render(<AssistantSummary actualCosts={[40, 25]} targetWeights={[1, 1, 2]} finalResources={35} />);
        expect(screen.getByText(/Overtuned/i)).toBeDefined();
    });

    it('should display "Impossible Day" for red status', () => {
        render(<AssistantSummary actualCosts={[60, 50]} targetWeights={[1, 1]} finalResources={0} />);
        expect(screen.getByText(/Impossible Day/i)).toBeDefined();
    });
});
