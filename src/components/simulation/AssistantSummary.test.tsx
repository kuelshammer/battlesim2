import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import AssistantSummary from './AssistantSummary';
import React from 'react';

describe('AssistantSummary Component', () => {
    it('should display "Balanced" message for green status', () => {
        const pacingData: any = {
            actualSegments: [{ type: 'combat', percent: 25 }, { type: 'combat', percent: 25 }],
            plannedSegments: [{ type: 'combat', percent: 25 }, { type: 'combat', percent: 25 }],
            finalResources: 50
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Balanced/i)).toBeDefined();
        expect(screen.getByText(/50%/)).toBeDefined();
    });

    it('should display "Minor Pacing Drift" for yellow status', () => {
        const pacingData: any = {
            actualSegments: [{ type: 'combat', percent: 32 }, { type: 'combat', percent: 25 }],
            plannedSegments: [{ type: 'combat', percent: 25 }, { type: 'combat', percent: 25 }],
            finalResources: 43
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Minor Pacing Drift/i)).toBeDefined();
    });

    it('should display "Overtuned" for delta > 10%', () => {
        const pacingData: any = {
            actualSegments: [{ type: 'combat', percent: 40 }, { type: 'combat', percent: 25 }],
            plannedSegments: [{ type: 'combat', percent: 25 }, { type: 'combat', percent: 25 }],
            finalResources: 35
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Overtuned/i)).toBeDefined();
    });

    it('should display "Impossible Day" for red status', () => {
        const pacingData: any = {
            actualSegments: [{ type: 'combat', percent: 60 }, { type: 'combat', percent: 50 }],
            plannedSegments: [{ type: 'combat', percent: 50 }, { type: 'combat', percent: 50 }],
            finalResources: 0
        };
        render(<AssistantSummary pacingData={pacingData} />);
        expect(screen.getByText(/Impossible Day/i)).toBeDefined();
    });
});
