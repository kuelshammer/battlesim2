import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import Simulation from './simulation';
import React from 'react';

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
    useSimulationWorker: () => ({
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
    })
}));

// Mock FontAwesomeIcon
vi.mock('@fortawesome/react-fontawesome', () => ({
    FontAwesomeIcon: () => <span data-testid="fa-icon" />
}));

describe('Simulation Component', () => {
    it('should render the Auto-Adjust button for combat encounters', () => {
        render(<Simulation />);
        
        // By default there is one empty combat
        const adjustBtn = screen.getByTitle(/Optimize this encounter/i);
        expect(adjustBtn).toBeDefined();
        expect(adjustBtn.textContent).toContain('Auto-Adjust');
    });
});
