import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import DescentGraph from './DescentGraph';
import React from 'react';
import { DecileStats } from '@/model/model';
import { PacingData } from './pacingUtils';

describe('DescentGraph Component', () => {
    const mockDeciles: DecileStats[] = [
        {}, {}, { vitalityTimeline: [100, 80], powerTimeline: [100, 90] },
        {}, {}, {}, {},
        { vitalityTimeline: [100, 90], powerTimeline: [100, 95] }
    ] as unknown as DecileStats[];

    const mockPacingData: PacingData = {
        plannedTimeline: [100, 80],
        labels: ['Start', 'E1'],
        vitalityTimeline: [100, 85],
        powerTimeline: [100, 92]
    } as unknown as PacingData;

    it('should render an SVG element', () => {
        const { container } = render(
            <DescentGraph 
                deciles={mockDeciles} 
                pacingData={mockPacingData} 
            />
        );
        expect(container.querySelector('svg')).toBeDefined();
    });

    it('should render the plan line', () => {
        const { container } = render(
            <DescentGraph 
                deciles={mockDeciles} 
                pacingData={mockPacingData} 
            />
        );
        const planLine = container.querySelector('path[stroke-dasharray]');
        expect(planLine).toBeDefined();
    });

    it('should render the risk areas', () => {
        const { container } = render(
            <DescentGraph 
                deciles={mockDeciles} 
                pacingData={mockPacingData} 
            />
        );
        const riskAreas = container.querySelectorAll('path[class*="riskArea"]');
        expect(riskAreas.length).toBeGreaterThan(0);
    });

    it('should handle empty timelines', () => {
        const { container } = render(
            <DescentGraph 
                deciles={[]} 
                pacingData={{ plannedTimeline: [], labels: [], vitalityTimeline: [], powerTimeline: [] } as unknown as PacingData} 
            />
        );
        expect(container.querySelector('svg')).toBeDefined();
    });
});