import { renderHook, act } from '@testing-library/react';
import { useSimulationWorker } from './useSimulationWorker';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the Worker
class MockWorker {
  onmessage: ((e: MessageEvent) => void) | null = null;
  postMessage = vi.fn();
  terminate = vi.fn();
}

global.Worker = MockWorker as any;

describe('useSimulationWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should initialize with default state', () => {
    const { result } = renderHook(() => useSimulationWorker());
    expect(result.current.isRunning).toBe(false);
    expect(result.current.genId).toBe(0);
  });

  it('should increment genId when runSimulation is called', () => {
    const { result } = renderHook(() => useSimulationWorker());
    
    act(() => {
      result.current.runSimulation([], []);
    });

    expect(result.current.isRunning).toBe(true);
    expect(result.current.genId).toBe(1);

    act(() => {
      result.current.runSimulation([], []);
    });

    expect(result.current.genId).toBe(2);
  });
});
