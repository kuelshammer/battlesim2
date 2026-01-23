import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the Worker to avoid constructor issues
class MockWorker {
  onmessage: ((e: MessageEvent) => void) | null = null;
  postMessage = vi.fn();
  terminate = vi.fn();
}

global.Worker = vi.fn().mockImplementation(() => new MockWorker());

// Mock the controller to avoid Worker creation
const mockController = {
  startSimulation: vi.fn(),
  autoAdjustEncounter: vi.fn(),
  cancel: vi.fn()
};

vi.mock('./simulation.worker.controller', () => ({
  SimulationWorkerController: class MockSimulationWorkerController {
    startSimulation = mockController.startSimulation;
    autoAdjustEncounter = mockController.autoAdjustEncounter;
    cancel = mockController.cancel;
  }
}));

// Mock setTimeout to execute once to avoid infinite recursion in tests
let timeoutCount = 0;
vi.stubGlobal('setTimeout', (fn: () => void) => {
  if (timeoutCount < 2) {
    timeoutCount++;
    fn();
  }
});

const postMessageMock = vi.fn();
global.self = {
  postMessage: postMessageMock,
  onmessage: null
} as any;

// Import after mocks are set up
import { handleMessage } from './simulation.worker';

describe('SimulationWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    timeoutCount = 0;
  });

  it('should call startSimulation for simulation requests', async () => {
    mockController.startSimulation = vi.fn().mockImplementation((players, timeline, genId, maxK, seed, onResult) => {
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

    const event = {
      data: {
        type: 'START_SIMULATION',
        players: [],
        timeline: [],
        genId: 1
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(mockController.startSimulation).toHaveBeenCalledWith(
      [],
      [],
      1,
      51,
      undefined,
      expect.any(Function)
    );

    expect(postMessageMock).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'SIMULATION_UPDATE',
        genId: 1,
        kFactor: 1,
        isFinal: true
      })
    );
  });

  it('should handle simulation cancellation', async () => {
    mockController.cancel = vi.fn();

    const event = {
      data: {
        type: 'CANCEL_SIMULATION'
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(mockController.cancel).toHaveBeenCalled();
  });

  it('should handle auto-adjust encounter', async () => {
    mockController.autoAdjustEncounter = vi.fn().mockImplementation((players, monsters, timeline, encounterIndex, genId, onResult) => {
      onResult({
        type: 'completed',
        genId,
        result: { adjusted: true }
      });
    });

    const event = {
      data: {
        type: 'AUTO_ADJUST_ENCOUNTER',
        players: [],
        monsters: [],
        timeline: [],
        encounterIndex: 0,
        genId: 1
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(mockController.autoAdjustEncounter).toHaveBeenCalledWith(
      [],
      [],
      [],
      0,
      1,
      expect.any(Function)
    );

    expect(postMessageMock).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'AUTO_ADJUST_COMPLETE',
        genId: 1,
        result: { adjusted: true }
      })
    );
  });

  it('should handle simulation errors', async () => {
    mockController.startSimulation = vi.fn().mockImplementation((players, timeline, genId, maxK, seed, onResult) => {
      onResult({
        type: 'errored',
        genId,
        error: 'Test error'
      });
    });

    const event = {
      data: {
        type: 'START_SIMULATION',
        players: [],
        timeline: [],
        genId: 1
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(postMessageMock).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'SIMULATION_ERROR',
        genId: 1,
        error: 'Test error'
      })
    );
  });
});