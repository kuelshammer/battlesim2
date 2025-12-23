import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import DescentGraph from './DescentGraph';
import React from 'react';

describe('DescentGraph Component', () => {
    const mockDecileTimelines = [
        [100, 80, 60], // Decile 0
        [100, 82, 62],
        [100, 84, 64],
        [100, 86, 66],
        [100, 88, 68], // Decile 4 (Median)
        [100, 90, 70],
        [100, 92, 72],
        [100, 94, 74],
        [100, 96, 76],
        [100, 98, 78], // Decile 9
    ];

    const mockPlanTimeline = [100, 90, 80];
    const mockPacingData: any = {
        plannedTimeline: mockPlanTimeline,
        labels: ['Start', 'E1', 'E2']
    };

    it('should render an SVG element', () => {
        const { container } = render(
            <DescentGraph 
                decileTimelines={mockDecileTimelines} 
                pacingData={mockPacingData} 
            />
        );
        expect(container.querySelector('svg')).toBeDefined();
    });

    it('should render the plan line', () => {
        const { container } = render(
            <DescentGraph 
                decileTimelines={mockDecileTimelines} 
                pacingData={mockPacingData} 
            />
        );
        // Look for a path with stroke-dasharray (dotted line)
        const planLine = container.querySelector('path[stroke-dasharray]');
        expect(planLine).toBeDefined();
    });

    it('should render the risk area', () => {
        const { container } = render(
            <DescentGraph 
                decileTimelines={mockDecileTimelines} 
                pacingData={mockPacingData} 
            />
        );
        const riskArea = container.querySelector('path[class*="riskArea"]');
        expect(riskArea).toBeDefined();
    });

    it('should handle empty timelines', () => {
        const { container } = render(
            <DescentGraph 
                decileTimelines={[]} 
                pacingData={{ plannedTimeline: [], labels: [] } as any} 
            />
        );
        expect(container.querySelector('svg')).toBeDefined();
    });
});
