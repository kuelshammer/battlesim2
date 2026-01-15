import { renderHook, act } from '@testing-library/react';
import { useSimulationWorker } from './useSimulationWorker';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the Worker
const mockWorkerInstance = {
  onmessage: null as ((e: MessageEvent) => void) | null,
  postMessage: vi.fn(),
  terminate: vi.fn(),
};

global.Worker = vi.fn().mockImplementation(function() { return mockWorkerInstance; }) as unknown;

describe('useSimulationWorker Logs', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should receive analysis with decileLogs', async () => {
    const { result } = renderHook(() => useSimulationWorker());
    
    const mockAnalysis = {
      overall: { decileLogs: [[]] },
      encounters: [
        { decileLogs: [['event1', 'event2'], ['event3']] }
      ]
    };

    act(() => {
      // Trigger effect to create worker
      result.current.terminateAndRestart();
      
      mockWorkerInstance.onmessage!({
        data: {
          type: 'SIMULATION_UPDATE',
          results: [],
          analysis: mockAnalysis,
          events: [],
          kFactor: 51,
          isFinal: true
        }
      } as MessageEvent);
    });

    expect(result.current.analysis).toEqual(mockAnalysis);
  });
});
