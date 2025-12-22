import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import Simulation from './simulation';
import React from 'react';
import { useSimulationWorker } from '@/model/useSimulationWorker';
import { Creature } from '@/model/model';

// Mock localStorage
const localStorageMock = (() => {
    let store: Record<string, string> = {};
    return {
        getItem: vi.fn((key: string) => store[key] || null),
        setItem: vi.fn((key: string, value: string) => {
            store[key] = value.toString();
        }),
        clear: vi.fn(() => {
            store = {};
        }),
        removeItem: vi.fn((key: string) => {
            delete store[key];
        }),
    };
})();

Object.defineProperty(window, 'localStorage', {
    value: localStorageMock,
});

// Mock the hook
vi.mock('@/model/useSimulationWorker', () => ({
    useSimulationWorker: vi.fn()
}));

// Mock FontAwesomeIcon
vi.mock('@fortawesome/react-fontawesome', () => ({
    FontAwesomeIcon: () => <span data-testid="fa-icon" />
}));

// Mock useStoredState to provide a monster
vi.mock('@/model/utils', async () => {
    const actual = await vi.importActual('@/model/utils') as any;
    return {
        ...actual,
        useStoredState: vi.fn((key, defaultValue, parser) => {
            if (key === 'timeline') {
                return [[{
                    type: 'combat',
                    id: 'combat-1',
                    monsters: [{ id: 'm1', name: 'Test Monster', count: 1, hp: 50, ac: 10, actions: [], mode: 'monster' }],
                    playersSurprised: false,
                    monstersSurprised: false,
                }], vi.fn()];
            }
            return [defaultValue, vi.fn()];
        })
    };
});

describe('Simulation Component', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should render the Auto-Adjust button for combat encounters', () => {
        vi.mocked(useSimulationWorker).mockReturnValue({
            isRunning: false,
            progress: 0,
            completed: 0,
            total: 0,
            results: null,
            analysis: null,
            events: null,
            error: null,
            optimizedResult: null,
            runSimulation: vi.fn(),
            autoAdjustEncounter: vi.fn(),
            clearOptimizedResult: vi.fn(),
            terminateAndRestart: vi.fn(),
        } as any);

        render(<Simulation />);
        
        const adjustBtn = screen.getByTitle(/Optimize this encounter/i);
        expect(adjustBtn).toBeDefined();
        expect(adjustBtn.textContent).toContain('Auto-Adjust');
    });

    it('should render the Adjustment Preview modal when optimizedResult is present', async () => {
        const mockOptimizedResult = {
            monsters: [
                { id: 'm1', name: 'Test Monster', count: 1, hp: 100, ac: 15, actions: [], mode: 'monster' }
            ],
            analysis: {
                scenarioName: 'Optimized',
                totalRuns: 2511,
                deciles: [],
                battleDurationRounds: 5
            }
        };

        // Mock return value for when it has result
        vi.mocked(useSimulationWorker).mockReturnValue({
            isRunning: false,
            progress: 100,
            completed: 1,
            total: 1,
            results: null,
            analysis: null,
            events: null,
            error: null,
            optimizedResult: mockOptimizedResult,
            runSimulation: vi.fn(),
            autoAdjustEncounter: vi.fn(),
            clearOptimizedResult: vi.fn(),
            terminateAndRestart: vi.fn(),
        } as any);

        render(<Simulation />);

        // Click the button to set selectedEncounterIndex
        const adjustBtn = screen.getByTitle(/Optimize this encounter/i);
        fireEvent.click(adjustBtn);
        
        expect(screen.queryByText(/Adjustment Preview/i)).not.toBeNull();
        expect(screen.queryAllByText(/Test Monster/i).length).toBeGreaterThan(1);
        // Check that the optimized HP (100) is shown
        expect(screen.queryByText('100')).not.toBeNull();
    });

    it('should apply optimized results when Apply Changes is clicked', async () => {
        const mockOptimizedResult = {
            monsters: [
                { id: 'm1', name: 'Test Monster', count: 1, hp: 100, ac: 15, actions: [], mode: 'monster' }
            ],
            analysis: {
                scenarioName: 'Optimized',
                totalRuns: 2511,
                deciles: [],
                battleDurationRounds: 5
            }
        };

        const clearOptimizedResult = vi.fn();

        vi.mocked(useSimulationWorker).mockReturnValue({
            isRunning: false,
            optimizedResult: mockOptimizedResult,
            clearOptimizedResult,
            autoAdjustEncounter: vi.fn(),
        } as any);

        render(<Simulation />);
        
        const adjustBtn = screen.getByTitle(/Optimize this encounter/i);
        fireEvent.click(adjustBtn);

        const applyBtn = screen.getByText(/Apply Changes/i);
        fireEvent.click(applyBtn);

        expect(clearOptimizedResult).toHaveBeenCalled();
    });
});