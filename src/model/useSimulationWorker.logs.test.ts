import { renderHook, act } from '@testing-library/react';
import { useSimulationWorker } from './useSimulationWorker';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the Worker (not directly used, but needed by controller)
class MockWorker {
  onmessage: ((e: MessageEvent) => void) | null = null;
  postMessage = vi.fn();
  terminate = vi.fn();
}

global.Worker = vi.fn().mockImplementation(() => new MockWorker());

// Create hoisted variables for tracking the callback
const { handleResultCallback, startSimulationMock } = vi.hoisted(() => {
  let callback: ((result: any) => void) | null = null;
  const mock = vi.fn().mockImplementation((players, timeline, genId, maxK, seed, onResult) => {
    callback = onResult;
  });
  return {
    handleResultCallback: () => callback,
    startSimulationMock: mock,
  };
});

vi.mock('@/worker/simulation.worker.controller', () => {
  class MockSimulationWorkerController {
    startSimulation = startSimulationMock;
    autoAdjustEncounter = vi.fn();
    cancel = vi.fn();
    terminate = vi.fn();
  }
  return {
    SimulationWorkerController: MockSimulationWorkerController
  };
});

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
      // Call runSimulation to set up the callback
      result.current.runSimulation([], [], 51);
    });

    // Now the callback should be set up
    const callback = handleResultCallback();
    expect(callback).toBeTruthy();

    act(() => {
      // Trigger the result callback with mock analysis
      callback!({
        type: 'completed',
        genId: result.current.genId,
        results: [],
        analysis: mockAnalysis,
        events: [],
        kFactor: 51,
        isFinal: true
      });
    });

    expect(result.current.analysis).toEqual(mockAnalysis);
  });
});
