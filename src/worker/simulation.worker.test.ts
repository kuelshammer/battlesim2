import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the dependencies
vi.mock('simulation-wasm', () => {
  const mockFinalize = vi.fn().mockReturnValue({
    results: [],
    analysis: {},
    firstRunEvents: []
  });

  const mockRunChunk = vi.fn().mockReturnValue(0.8);

  const MockChunkedSimulationRunner = vi.fn().mockImplementation(function() {
    return {
      run_chunk: mockRunChunk,
      finalize: mockFinalize
    };
  });

  return {
    default: vi.fn().mockResolvedValue({}),
    ChunkedSimulationRunner: MockChunkedSimulationRunner,
    auto_adjust_encounter_wasm: vi.fn()
  };
});

const postMessageMock = vi.fn();
global.self = {
  postMessage: postMessageMock,
  onmessage: null
} as any;

// Mock setTimeout to execute once to avoid infinite recursion in tests
let timeoutCount = 0;
vi.stubGlobal('setTimeout', (fn: Function) => {
  if (timeoutCount < 2) {
    timeoutCount++;
    fn();
  }
});

import { handleMessage } from './simulation.worker';
import { ChunkedSimulationRunner } from 'simulation-wasm';

describe('SimulationWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    timeoutCount = 0;
  });

  it('should use ChunkedSimulationRunner with correct iterations', async () => {
    const iterations = 100;
    const event = {
      data: {
        type: 'START_SIMULATION',
        players: [],
        timeline: [],
        iterations
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(ChunkedSimulationRunner).toHaveBeenCalledWith(
      expect.anything(),
      expect.anything(),
      iterations,
      undefined,
      1  // kFactor parameter (defaults to 1)
    );

    // We can't easily access the internal mock instances from outside if they are defined inside vi.mock
    // but the test above already verifies the constructor call which is the most important part.
  });
});
