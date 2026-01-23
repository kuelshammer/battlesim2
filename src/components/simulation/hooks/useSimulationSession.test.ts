import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSimulationSession } from './useSimulationSession';

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

// Mock uuid
vi.mock('uuid', () => ({
    v4: vi.fn(() => 'test-uuid')
}));

describe('useSimulationSession', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        localStorageMock.clear();
    });

    it('should initialize with default empty state', () => {
        const { result } = renderHook(() => useSimulationSession());

        expect(result.current.players).toEqual([]);
        expect(result.current.timeline).toHaveLength(1);
        expect(result.current.timeline[0].type).toBe('combat');
        expect(result.current.isEmptyResult).toBe(true);
        expect(result.current.encounterWeights).toEqual([2]); // Standard role weight
    });

    it('should create combat encounters', () => {
        const { result } = renderHook(() => useSimulationSession());

        act(() => {
            result.current.createCombat();
        });

        expect(result.current.timeline).toHaveLength(2);
        expect(result.current.timeline[1].type).toBe('combat');
        expect(result.current.timeline[1].id).toBe('test-uuid');
    });

    it('should create short rest encounters', () => {
        const { result } = renderHook(() => useSimulationSession());

        act(() => {
            result.current.createShortRest();
        });

        expect(result.current.timeline).toHaveLength(2);
        expect(result.current.timeline[1].type).toBe('shortRest');
        expect(result.current.timeline[1].id).toBe('test-uuid');
    });

    it('should update timeline items', () => {
        const { result } = renderHook(() => useSimulationSession());

        const newItem = {
            type: 'combat' as const,
            id: 'updated-id',
            monsters: [],
            monstersSurprised: true,
            playersSurprised: false,
            targetRole: 'Elite' as const,
        };

        act(() => {
            result.current.updateTimelineItem(0, newItem);
        });

        expect(result.current.timeline[0]).toEqual(newItem);
        expect(result.current.encounterWeights).toEqual([3]); // Elite role weight
    });

    it('should delete timeline items', () => {
        const { result } = renderHook(() => useSimulationSession());

        act(() => {
            result.current.createCombat();
        });

        expect(result.current.timeline).toHaveLength(2);

        act(() => {
            result.current.deleteTimelineItem(1);
        });

        expect(result.current.timeline).toHaveLength(1);
    });

    it('should swap timeline items', () => {
        const { result } = renderHook(() => useSimulationSession());

        act(() => {
            result.current.createShortRest();
        });

        const firstItem = result.current.timeline[0];
        const secondItem = result.current.timeline[1];

        act(() => {
            result.current.swapTimelineItems(0, 1);
        });

        expect(result.current.timeline[0]).toEqual(secondItem);
        expect(result.current.timeline[1]).toEqual(firstItem);
    });

    it('should clear adventuring day', () => {
        const { result } = renderHook(() => useSimulationSession());

        act(() => {
            result.current.createCombat();
        });

        expect(result.current.timeline).toHaveLength(2);
        expect(result.current.isEmptyResult).toBe(true); // No players, so still empty

        act(() => {
            result.current.clearAdventuringDay();
        });

        expect(result.current.players).toEqual([]);
        expect(result.current.timeline).toHaveLength(1);
        expect(result.current.timeline[0].type).toBe('combat');
        expect(result.current.isEmptyResult).toBe(true);
    });

    it('should compute encounter weights correctly', () => {
        const { result } = renderHook(() => useSimulationSession());

        expect(result.current.encounterWeights).toEqual([2]); // Default combat is Standard

        const eliteCombat = {
            type: 'combat' as const,
            id: 'elite-combat',
            monsters: [],
            monstersSurprised: false,
            playersSurprised: false,
            targetRole: 'Elite' as const,
        };

        act(() => {
            result.current.updateTimelineItem(0, eliteCombat);
        });

        expect(result.current.encounterWeights).toEqual([3]); // Elite weight
    });
});