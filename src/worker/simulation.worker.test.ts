import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock the dependencies
vi.mock('simulation-wasm', () => ({
  default: vi.fn().mockResolvedValue({}),
  run_simulation_with_callback: vi.fn().mockReturnValue({
    results: [],
    analysis: {},
    firstRunEvents: []
  }),
  auto_adjust_encounter_wasm: vi.fn()
}));

// Mock the require for wasm file - this is tricky with 'require' in source
// We might need to handle the module resolution or let it fail and fix
// But let's try to mock the module that 'require' would resolve to.
// Since it is a relative path in node_modules or alias.

const postMessageMock = vi.fn();
global.self = {
  postMessage: postMessageMock,
  onmessage: null
} as any;

import { handleMessage } from './simulation.worker';
import * as wasm from 'simulation-wasm';

describe('SimulationWorker', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call run_simulation_with_callback with correct iterations', async () => {
    const iterations = 31;
    const event = {
      data: {
        type: 'START_SIMULATION',
        players: [],
        timeline: [],
        iterations
      }
    } as MessageEvent;

    await handleMessage(event);

    expect(wasm.run_simulation_with_callback).toHaveBeenCalledWith(
      expect.anything(),
      expect.anything(),
      iterations,
      expect.anything()
    );
  });
});
