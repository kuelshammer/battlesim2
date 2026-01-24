# Testing Guidelines

## Vitest Mock Initialization

### ALWAYS use vi.hoisted() for mock functions

❌ WRONG - Causes TDZ errors:
```typescript
const mockController = { startSimulation: vi.fn() };
vi.mock('./controller', () => ({
  Controller: class { start = mockController.startSimulation }
}));
```

✅ CORRECT:
```typescript
const { startSimulationMock } = vi.hoisted(() => ({
  startSimulationMock: vi.fn()
}));
vi.mock('./controller', () => ({
  Controller: class { start = startSimulationMock }
}));
```

## CI Configuration
- Tests run with retry: 2
- Sequential execution (threads: false)
- Timeout: 10000ms
- Test retries on failure in CI workflow
- Test logs uploaded on CI failure

## Running Tests

### Standard test run
```bash
npm test
```

### Stable test run (with retries, sequential)
```bash
npm run test:stable
```

### CI test run (verbose, with retries)
```bash
npm run test:ci
```

## Common Pitfalls

### 1. Temporal Dead Zone (TDZ) Errors
When using `vi.mock()`, any variables referenced in the mock factory must be declared **before** the mock. Use `vi.hoisted()` to lift variable declarations above the mock.

### 2. Worker Mocks
The global test setup (`tests/vitest.setup.ts`) provides a default Worker mock. Don't mock Worker in individual test files unless you need custom behavior.

### 3. Test Isolation
Always use `beforeEach(() => vi.clearAllMocks())` to ensure tests don't leak state.
