# E2E Tests with Puppeteer & Vitest

This directory contains End-to-End tests for the BattleSim application using Puppeteer and Vitest.

## Structure

```
e2e/
├── config/
│   ├── vitest.e2e.config.ts    # Vitest configuration for E2E tests
│   └── setup.ts                 # Global test setup (Puppeteer launch)
├── pages/                       # Page Object Models (POM)
│   ├── BasePage.ts             # Base page with common utilities
│   ├── SimulationPage.ts       # Main simulation interface
│   ├── CreatureModal.ts        # Creature creation/editing dialog
│   └── ResultsPanel.ts         # Simulation results display
├── specs/                       # Test specifications
│   ├── smoke.e2e.ts           # Basic smoke tests
│   ├── combat-flow.e2e.ts     # Combat workflow tests
│   └── persistence.e2e.ts     # State management tests
├── fixtures/                    # Test data
│   └── basic-combat.json       # Sample combat scenario
└── screenshots/                 # Failure screenshots (auto-generated)
```

## Running Tests

### Prerequisites

1. Build the WASM module:
   ```bash
   npm run build:wasm:dev
   ```

2. Start the development server (in a separate terminal):
   ```bash
   npm run dev
   ```

### Run E2E Tests

```bash
# Run all E2E tests
npm run test:e2e

# Run with UI (for debugging)
npm run test:e2e:ui

# Run in visible mode (headful) for debugging
npm run test:e2e:debug
```

### Environment Variables

- `E2E_BASE_URL`: Base URL for tests (default: `http://localhost:3000`)
- `E2E_HEADLESS`: Run headless (default: `true`, set to `false` for debugging)

Example:
```bash
E2E_HEADLESS=false npm run test:e2e
```

## Page Object Model

The tests use the Page Object Model pattern for maintainable test code:

```typescript
import { SimulationPage } from '../pages/SimulationPage';

const page = new SimulationPage(global.page);
await page.goto();
await page.clickAddCreature();
// ... assertions
```

## Writing New Tests

1. Create a new test file in `e2e/specs/` with `.e2e.ts` extension
2. Import the necessary page objects
3. Write tests using Vitest syntax

Example:
```typescript
import { describe, it, expect } from 'vitest';
import { SimulationPage } from '../pages/SimulationPage';

describe('My Feature', () => {
  it('should do something', async () => {
    const page = new SimulationPage(global.page);
    await page.goto();
    // ... test code
  });
});
```

## Test Data

Place test fixtures in `e2e/fixtures/` as JSON files. Import them in tests:

```typescript
import myFixture from '../fixtures/my-scenario.json';
```

## Debugging

When tests fail, screenshots are automatically saved to `e2e/screenshots/`.

For interactive debugging:
```bash
npm run test:e2e:debug
```

This will:
1. Launch Puppeteer in visible (non-headless) mode
2. Start Vitest with inspector support
3. Allow you to step through tests

## CI/CD Integration

For CI, ensure the following:
1. WASM is built before tests run
2. Next.js server is started on the expected port
3. `E2E_HEADLESS=true` is set

Example GitHub Actions step:
```yaml
- name: Build WASM
  run: npm run build:wasm:dev

- name: Start server
  run: npm run dev &
  run: npx wait-on http://localhost:3000

- name: Run E2E tests
  run: npm run test:e2e
```

## Current Test Coverage

- **Smoke Tests**: Application loads, WASM initializes
- **Combat Flow**: Add creatures, run simulation, verify results
- **Persistence**: LocalStorage save/restore, multi-creature scenarios

## Future Enhancements

- [ ] Visual regression testing
- [ ] Performance benchmarks
- [ ] Accessibility testing with axe-core
- [ ] Mobile viewport testing
- [ ] Import/Export file download/upload testing
