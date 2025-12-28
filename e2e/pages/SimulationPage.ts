import { BasePage } from './BasePage';

/**
 * Simulation Page Object Model
 * Represents the main simulation interface
 */
export class SimulationPage extends BasePage {
  // Selectors
  private readonly selectors = {
    // Creature list
    creatureList: '[data-testid="creature-list"]',
    creatureItem: '[data-testid="creature-item"]',
    addCreatureBtn: '[data-testid="add-creature-btn"]',
    removeCreatureBtn: '[data-testid="remove-creature-btn"]',

    // Simulation controls
    runSimulationBtn: '[data-testid="run-simulation-btn"]',
    repetitionInput: '[data-testid="repetition-input"]',
    maxRoundsInput: '[data-testid="max-rounds-input"]',

    // Results
    resultsPanel: '[data-testid="results-panel"]',
    loadingIndicator: '[data-testid="simulation-loading"]',
    winRateDisplay: '[data-testid="win-rate"]',
    damageStats: '[data-testid="damage-stats"]',
    timeline: '[data-testid="timeline"]',

    // Error states
    errorMessage: '[data-testid="error-message"]',
    wasmError: '[data-testid="wasm-error"]',

    // Export/Import
    exportBtn: '[data-testid="export-btn"]',
    importBtn: '[data-testid="import-btn"]',
  };

  /**
   * Navigate to the simulation page
   */
  async goto(): Promise<void> {
    await super.goto('/');
    await this.waitForPageReady();
  }

  /**
   * Wait for the page to be fully loaded (WASM initialized)
   */
  async waitForPageReady(timeout: number = 15000): Promise<void> {
    await this.page.waitForFunction(
      () => {
        return (window as any).simulationWasm !== undefined &&
               document.querySelector('[data-testid="creature-list"]') !== null;
      },
      { timeout }
    );
  }

  /**
   * Get the number of creatures in the list
   */
  async getCreatureCount(): Promise<number> {
    await this.waitForVisible(this.selectors.creatureList);
    return await this.evaluate(() => {
      return document.querySelectorAll('[data-testid="creature-item"]').length;
    });
  }

  /**
   * Get all creature names
   */
  async getCreatureNames(): Promise<string[]> {
    await this.waitForVisible(this.selectors.creatureList);
    return await this.getTexts('[data-testid="creature-name"]');
  }

  /**
   * Click the "Add Creature" button
   */
  async clickAddCreature(): Promise<void> {
    await this.click(this.selectors.addCreatureBtn);
  }

  /**
   * Remove a creature by index
   */
  async removeCreature(index: number): Promise<void> {
    const buttons = await this.page.$$(this.selectors.removeCreatureBtn);
    if (index < buttons.length) {
      await buttons[index].click();
    } else {
      throw new Error(`Creature index ${index} out of bounds`);
    }
  }

  /**
   * Set the number of repetitions
   */
  async setRepetitions(count: number): Promise<void> {
    await this.page.waitForSelector(this.selectors.repetitionInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.repetitionInput,
      count
    );
  }

  /**
   * Set the maximum rounds
   */
  async setMaxRounds(rounds: number): Promise<void> {
    await this.page.waitForSelector(this.selectors.maxRoundsInput);
    await this.page.evaluate(
      (sel, val) => {
        const input = document.querySelector(sel) as HTMLInputElement;
        if (input) input.value = val.toString();
      },
      this.selectors.maxRoundsInput,
      rounds
    );
  }

  /**
   * Run the simulation
   */
  async runSimulation(): Promise<void> {
    await this.click(this.selectors.runSimulationBtn);
    // Wait for loading to complete
    await this.page.waitForFunction(
      () => {
        const loader = document.querySelector('[data-testid="simulation-loading"]');
        return loader === null;
      },
      { timeout: 60000 }
    );
  }

  /**
   * Check if simulation is running
   */
  async isSimulationRunning(): Promise<boolean> {
    return await this.evaluate(() => {
      return document.querySelector('[data-testid="simulation-loading"]') !== null;
    });
  }

  /**
   * Wait for simulation results
   */
  async waitForResults(timeout: number = 60000): Promise<void> {
    await this.waitForVisible(this.selectors.resultsPanel, timeout);
  }

  /**
   * Get win rate text
   */
  async getWinRate(): Promise<string> {
    await this.waitForVisible(this.selectors.winRateDisplay);
    return await this.getText(this.selectors.winRateDisplay);
  }

  /**
   * Get all damage statistics
   */
  async getDamageStats(): Promise<string[]> {
    await this.waitForVisible(this.selectors.damageStats);
    return await this.getTexts(`${this.selectors.damageStats} [data-testid="stat-row"]`);
  }

  /**
   * Check if there's an error message
   */
  async hasError(): Promise<boolean> {
    return await this.exists(this.selectors.errorMessage);
  }

  /**
   * Get error message text
   */
  async getErrorMessage(): Promise<string> {
    await this.waitForVisible(this.selectors.errorMessage);
    return await this.getText(this.selectors.errorMessage);
  }

  /**
   * Export the current state
   */
  async exportState(): Promise<string> {
    await this.click(this.selectors.exportBtn);
    // Wait a moment for download to trigger
    await this.page.waitForTimeout(1000);
    // In a real implementation, you'd capture the download
    return '';
  }

  /**
   * Import state from JSON
   */
  async importState(jsonData: object): Promise<void> {
    // Create a file input and upload
    await this.page.evaluate((data) => {
      const blob = new Blob([JSON.stringify(data)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const input = document.querySelector('[data-testid="import-input"]') as HTMLInputElement;
      if (input) {
        // This is a simplified approach - real implementation would need proper file handling
        console.log('Import data prepared:', data);
      }
      URL.revokeObjectURL(url);
    }, jsonData);
  }

  /**
   * Get LocalStorage state
   */
  async getLocalStorage(): Promise<Record<string, string>> {
    return await this.evaluate(() => {
      const items: Record<string, string> = {};
      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key) {
          items[key] = localStorage.getItem(key) || '';
        }
      }
      return items;
    });
  }

  /**
   * Set LocalStorage state
   */
  async setLocalStorage(data: Record<string, string>): Promise<void> {
    await this.evaluate((items) => {
      Object.entries(items).forEach(([key, value]) => {
        localStorage.setItem(key, value);
      });
    }, data);
    await this.page.reload({ waitUntil: 'networkidle0' });
    await this.waitForPageReady();
  }

  /**
   * Clear LocalStorage
   */
  async clearLocalStorage(): Promise<void> {
    await this.evaluate(() => {
      localStorage.clear();
    });
    await this.page.reload({ waitUntil: 'networkidle0' });
    await this.waitForPageReady();
  }
}
