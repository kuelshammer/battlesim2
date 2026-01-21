import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAutoSimulation } from './useAutoSimulation';

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

// Mock the worker hook
vi.mock('@/model/useSimulationWorker', () => ({
    useSimulationWorker: vi.fn(() => ({
        isRunning: false,
        progress: 0,
        kFactor: 0,
        maxK: 51,
        results: null,
        analysis: null,
        events: null,
        error: null,
        optimizedResult: null,
        genId: 0,
        runSimulation: vi.fn(),
        autoAdjustEncounter: vi.fn(),
        clearOptimizedResult: vi.fn(),
        terminateAndRestart: vi.fn(),
    }))
}));

describe('useAutoSimulation', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        localStorageMock.clear();
    });

    it('should initialize with default state', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        expect(result.current.simulationResults).toEqual([]);
        expect(result.current.simulationEvents).toEqual([]);
        expect(result.current.needsResimulation).toBe(false);
        expect(result.current.isStale).toBe(false);
        expect(result.current.highPrecision).toBe(false);
        expect(result.current.isEditing).toBe(false);
        expect(result.current.canSave).toBe(false);
    });

    it('should set high precision mode', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        act(() => {
            result.current.setHighPrecision(true);
        });

        expect(result.current.highPrecision).toBe(true);
    });

    it('should set editing state', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        act(() => {
            result.current.setIsEditing(true);
        });

        expect(result.current.isEditing).toBe(true);
    });

    it('should expose setter functions', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        expect(typeof result.current.setSaving).toBe('function');
        expect(typeof result.current.setLoading).toBe('function');
        expect(typeof result.current.setIsEditing).toBe('function');
    });

    it('should trigger resimulation', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        act(() => {
            result.current.triggerResimulation();
        });

        expect(result.current.needsResimulation).toBe(true);
        expect(result.current.isStale).toBe(true);
    });

    it('should expose worker methods', () => {
        const { result } = renderHook(() =>
            useAutoSimulation([], [], true, true)
        );

        expect(result.current.worker).toBeDefined();
        expect(typeof result.current.worker.runSimulation).toBe('function');
        expect(typeof result.current.worker.autoAdjustEncounter).toBe('function');
    });
});