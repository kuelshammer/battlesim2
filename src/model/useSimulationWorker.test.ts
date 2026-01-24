import { renderHook, act } from '@testing-library/react';
import { useSimulationWorker } from './useSimulationWorker';
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the SimulationWorkerController using vi.hoisted to avoid TDZ issues
const { startSimulationMock, autoAdjustEncounterMock, cancelMock, restartMock, terminateMock } = vi.hoisted(() => ({
  startSimulationMock: vi.fn(),
  autoAdjustEncounterMock: vi.fn(),
  cancelMock: vi.fn(),
  restartMock: vi.fn(),
  terminateMock: vi.fn(),
}));

vi.mock('@/worker/simulation.worker.controller', () => {
  return {
    SimulationWorkerController: class {
      startSimulation = startSimulationMock;
      autoAdjustEncounter = autoAdjustEncounterMock;
      cancel = cancelMock;
      restart = restartMock;
      terminate = terminateMock;
    }
  };
});

describe('useSimulationWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should initialize with default state', () => {
    const { result } = renderHook(() => useSimulationWorker());
    expect(result.current.isRunning).toBe(false);
    expect(result.current.genId).toBe(0);
    expect(result.current.isCancelled).toBe(false);
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

  it('should cancel existing simulation when starting new one', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.runSimulation([], []);
    });

    expect(startSimulationMock).toHaveBeenCalledWith(
      [],
      [],
      1,
      51,
      undefined,
      expect.any(Function)
    );

    // Start another simulation
    act(() => {
      result.current.runSimulation([], []);
    });

    // Should have canceled first
    expect(cancelMock).toHaveBeenCalled();
  });

  it('should handle cancellation messages', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.runSimulation([], []);
    });

    act(() => {
      // Simulate cancellation from controller
      const onResult = startSimulationMock.mock.calls[0][5];
      onResult({
        type: 'cancelled',
        genId: 1
      });
    });

    expect(result.current.isRunning).toBe(false);
    expect(result.current.isCancelled).toBe(true);
  });

  it('should cancel simulation when cancel is called', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.runSimulation([], []);
    });

    act(() => {
      result.current.cancel();
    });

    expect(cancelMock).toHaveBeenCalled();
    expect(result.current.isRunning).toBe(false);
    expect(result.current.isCancelled).toBe(true);
  });

  it('should handle auto-adjust cancellation', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.autoAdjustEncounter([], [], [], 0);
    });

    act(() => {
      result.current.cancel();
    });

    expect(cancelMock).toHaveBeenCalled();
  });

  it('should handle abort signals properly in simulation', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.runSimulation([], []);
    });

    // Simulate abort signal triggering cancellation
    act(() => {
      const onResult = startSimulationMock.mock.calls[0][5];
      onResult({
        type: 'cancelled',
        genId: 1
      });
    });

    expect(result.current.isRunning).toBe(false);
    expect(result.current.isCancelled).toBe(true);
  });

  it('should handle errors from controller', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.runSimulation([], []);
    });

    act(() => {
      const onResult = startSimulationMock.mock.calls[0][5];
      onResult({
        type: 'errored',
        genId: 1,
        error: 'Simulation failed'
      });
    });

    expect(result.current.isRunning).toBe(false);
    expect(result.current.error).toBe('Simulation failed');
  });

  it('should expose restart helper', () => {
    const { result } = renderHook(() => useSimulationWorker());

    act(() => {
      result.current.terminateAndRestart();
    });

    expect(terminateMock).toHaveBeenCalled();
  });

  it('should terminate controller on unmount', () => {
    const { unmount } = renderHook(() => useSimulationWorker());

    unmount();

    expect(terminateMock).toHaveBeenCalled();
  });

  it('should handle rapid consecutive simulation starts (abort/restart sequence)', () => {
    const { result } = renderHook(() => useSimulationWorker());

    // Start first simulation
    act(() => {
      result.current.runSimulation([], []);
    });

    // Immediately start second simulation (should cancel first)
    act(() => {
      result.current.runSimulation([], []);
    });

    // Should have canceled twice (once for each runSimulation call) and started twice
    expect(cancelMock).toHaveBeenCalledTimes(2);
    expect(startSimulationMock).toHaveBeenCalledTimes(2);
  });

  it('should handle restart after cancellation', () => {
    const { result } = renderHook(() => useSimulationWorker());

    // Start simulation
    act(() => {
      result.current.runSimulation([], []);
    });

    // Cancel it
    act(() => {
      result.current.cancel();
    });

    // Start new simulation after cancellation
    act(() => {
      result.current.runSimulation([], []);
    });

    // cancel called: once by runSimulation (first), once by cancel(), once by runSimulation (second) = 3 times
    expect(cancelMock).toHaveBeenCalledTimes(3);
    expect(startSimulationMock).toHaveBeenCalledTimes(2);
    expect(result.current.isRunning).toBe(true);
    expect(result.current.isCancelled).toBe(false);
  });

  it('should prevent overlapping runs during async operations', () => {
    const { result } = renderHook(() => useSimulationWorker());

    // Mock the controller to simulate async behavior
    let resolveFirstSimulation: () => void;
    const firstSimulationPromise = new Promise<void>((resolve) => {
      resolveFirstSimulation = resolve;
    });

    startSimulationMock.mockImplementationOnce((players, timeline, genId, maxK, seed, onResult) => {
      // Simulate async operation that takes time
      firstSimulationPromise.then(() => {
        onResult({
          type: 'completed',
          genId,
          results: [],
          analysis: {},
          events: [],
          kFactor: 1,
          isFinal: true
        });
      });
    });

    // Start first simulation
    act(() => {
      result.current.runSimulation([], []);
    });

    // Immediately try to start second simulation
    act(() => {
      result.current.runSimulation([], []);
    });

    // cancel called: once by first runSimulation, once by second runSimulation = 2 times
    expect(cancelMock).toHaveBeenCalledTimes(2);

    // Resolve the first simulation
    act(() => {
      resolveFirstSimulation();
    });

    // Second simulation should still be running
    expect(result.current.isRunning).toBe(true);
    expect(result.current.genId).toBe(2);
  });

  it('should handle errors in abort/restart sequences', () => {
    const { result } = renderHook(() => useSimulationWorker());

    // Start simulation that will error
    act(() => {
      result.current.runSimulation([], []);
    });

    // Simulate error
    act(() => {
      const onResult = startSimulationMock.mock.calls[0][5];
      onResult({
        type: 'errored',
        genId: 1,
        error: 'Test error'
      });
    });

    expect(result.current.error).toBe('Test error');
    expect(result.current.isRunning).toBe(false);

    // Start new simulation after error
    act(() => {
      result.current.runSimulation([], []);
    });

    expect(result.current.error).toBe(null);
    expect(result.current.isRunning).toBe(true);
    expect(result.current.genId).toBe(2);
  });
});
