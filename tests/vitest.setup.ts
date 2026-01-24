import { vi, beforeEach } from 'vitest';

// Mock Worker globally for all tests
class MockWorker {
  onmessage: ((e: MessageEvent) => void) | null = null;
  postMessage = vi.fn();
  terminate = vi.fn();
}

global.Worker = vi.fn().mockImplementation(() => new MockWorker());

// Reset mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
});
