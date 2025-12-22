import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import EncounterResult from './encounterResult';
import React from 'react';
import { UIToggleProvider } from '@/model/uiToggleState';

describe('EncounterResult Component', () => {
    const mockValue = {
        stats: new Map(),
        rounds: [
            {
                team1: [],
                team2: []
            }
        ]
    } as any;

    it('should render the component', () => {
        render(
            <UIToggleProvider>
                <EncounterResult value={mockValue} />
            </UIToggleProvider>
        );
        expect(screen.getByText(/Detailed Analysis/i)).toBeDefined();
    });

    it('should render DeltaBadge when props are provided', () => {
        render(
            <UIToggleProvider>
                <EncounterResult 
                    value={mockValue} 
                    targetPercent={25} 
                    actualPercent={30} 
                />
            </UIToggleProvider>
        );
        // Delta = 30 - 25 = +5% (On Target)
        expect(screen.getByText(/On Target/i)).toBeDefined();
        expect(screen.getByText(/\+5%/)).toBeDefined();
    });
});
