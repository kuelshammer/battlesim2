import { describe, it, expect, beforeAll } from 'vitest';
import { SimulationPage } from '../pages/SimulationPage';

/**
 * Smoke Tests - Verify basic application functionality
 */
describe('E2E: Smoke Tests', () => {
  let simulationPage: SimulationPage;

  beforeAll(() => {
    simulationPage = new SimulationPage(global.page);
  });

  it('should load the application without errors', async () => {
    await simulationPage.goto();

    // Verify WASM is loaded
    const wasmReady = await global.page.evaluate(() => {
      return typeof (window as any).simulationWasm !== 'undefined';
    });
    expect(wasmReady).toBe(true);

    // Verify main UI elements are present
    const hasCreatureList = await simulationPage.exists('[data-testid="creature-list"]');
    expect(hasCreatureList).toBe(true);
  });

  it('should display initial empty state', async () => {
    await simulationPage.goto();

    const creatureCount = await simulationPage.getCreatureCount();
    expect(creatureCount).toBe(0);
  });

  it('should have no error messages on load', async () => {
    await simulationPage.goto();

    const hasError = await simulationPage.hasError();
    expect(hasError).toBe(false);
  });
});
