import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the dependencies
vi.mock('simulation-wasm', () => {
  const mockGetAnalysis = vi.fn().mockReturnValue({
    results: [],
    analysis: {},
    firstRunEvents: []
  });

  const mockRunChunk = vi.fn().mockReturnValue(100);

  const MockChunkedSimulationRunner = vi.fn().mockImplementation(function() {
    return {
      run_chunk: mockRunChunk,
      get_analysis: mockGetAnalysis
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
} as { postMessage: typeof postMessageMock; onmessage: null };

// Mock setTimeout to execute once to avoid infinite recursion in tests
let timeoutCount = 0;
vi.stubGlobal('setTimeout', (fn: () => void) => {
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

  it('should use ChunkedSimulationRunner correctly', async () => {
    const event = {
      data: {
        type: 'START_SIMULATION',
        players: [],
        timeline: [],
        genId: 1
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(ChunkedSimulationRunner).toHaveBeenCalledWith(
      expect.anything(),
      expect.anything(),
      undefined
    );
  });
});
