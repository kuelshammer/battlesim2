import { BasePage } from './BasePage';

/**
 * Results Panel Page Object Model
 * Represents the simulation results display
 */
export class ResultsPanel extends BasePage {
  // Selectors
  private readonly selectors = {
    container: '[data-testid="results-panel"]',
    loading: '[data-testid="simulation-loading"]',
    empty: '[data-testid="results-empty"]',

    // Summary
    winRate: '[data-testid="win-rate"]',
    avgRounds: '[data-testid="avg-rounds"]',
    tpkRate: '[data-testid="tpk-rate"]',

    // Damage statistics
    damageStats: '[data-testid="damage-stats"]',
    statRow: '[data-testid="stat-row"]',

    // Timeline
    timeline: '[data-testid="timeline"]',
    timelineItem: '[data-testid="timeline-item"]',

    // Deciles
    decilesView: '[data-testid="deciles-view"]',
    descentGraph: '[data-testid="descent-graph"]',

    // Pacing
    pacingChart: '[data-testid="pacing-chart"]',

    // Details
    detailToggle: '[data-testid="detail-toggle"]',
    eventLog: '[data-testid="event-log"]',
    logEntry: '[data-testid="log-entry"]',

    // Export
    exportLogBtn: '[data-testid="export-log-btn"]',
    exportJsonBtn: '[data-testid="export-json-btn"]',
  };

  /**
   * Wait for results to be displayed
   */
  async waitForResults(timeout: number = 60000): Promise<void> {
    await this.waitForVisible(this.selectors.container, timeout);
    // Wait for loading to complete
    await this.page.waitForFunction(
      () => document.querySelector('[data-testid="simulation-loading"]') === null,
      { timeout }
    );
  }

  /**
   * Check if results are displayed
   */
  async isDisplayed(): Promise<boolean> {
    return await this.exists(this.selectors.container);
  }

  /**
   * Check if still loading
   */
  async isLoading(): Promise<boolean> {
    return await this.exists(this.selectors.loading);
  }

  /**
   * Check if results are empty (no simulation run)
   */
  async isEmpty(): Promise<boolean> {
    return await this.exists(this.selectors.empty);
  }

  /**
   * Get win rate percentage
   */
  async getWinRate(): Promise<number> {
    await this.waitForVisible(this.selectors.winRate);
    const text = await this.getText(this.selectors.winRate);
    const match = text.match(/(\d+)%/);
    return match ? parseInt(match[1], 10) : 0;
  }

  /**
   * Get average rounds
   */
  async getAvgRounds(): Promise<number> {
    await this.waitForVisible(this.selectors.avgRounds);
    const text = await this.getText(this.selectors.avgRounds);
    const match = text.match(/(\d+(?:\.\d+)?)/);
    return match ? parseFloat(match[1]) : 0;
  }

  /**
   * Get TPK (Total Party Kill) rate
   */
  async getTPKRate(): Promise<number> {
    const text = await this.getText(this.selectors.tpkRate);
    const match = text.match(/(\d+)%/);
    return match ? parseInt(match[1], 10) : 0;
  }

  /**
   * Get all damage statistics
   */
  async getDamageStats(): Promise<Array<{ name: string; avgDamage: number; maxDamage: number }>> {
    await this.waitForVisible(this.selectors.damageStats);
    return await this.evaluate(() => {
      const rows = Array.from(document.querySelectorAll('[data-testid="stat-row"]'));
      return rows.map(row => {
        const name = row.querySelector('[data-testid="creature-name"]')?.textContent || '';
        const avgDamageText = row.querySelector('[data-testid="avg-damage"]')?.textContent || '0';
        const maxDamageText = row.querySelector('[data-testid="max-damage"]')?.textContent || '0';
        const avgDamage = parseFloat(avgDamageText.replace(/[^\d.]/g, '')) || 0;
        const maxDamage = parseFloat(maxDamageText.replace(/[^\d.]/g, '')) || 0;
        return { name, avgDamage, maxDamage };
      });
    });
  }

  /**
   * Get number of timeline events
   */
  async getTimelineEventCount(): Promise<number> {
    if (!await this.exists(this.selectors.timeline)) return 0;
    return await this.evaluate(() => {
      return document.querySelectorAll('[data-testid="timeline-item"]').length;
    });
  }

  /**
   * Get timeline events as text
   */
  async getTimelineEvents(): Promise<string[]> {
    if (!await this.exists(this.selectors.timeline)) return [];
    return await this.getTexts(this.selectors.timelineItem);
  }

  /**
   * Check if deciles view is available
   */
  async hasDecilesView(): Promise<boolean> {
    return await this.exists(this.selectors.decilesView);
  }

  /**
   * Check if pacing chart is available
   */
  async hasPacingChart(): Promise<boolean> {
    return await this.exists(this.selectors.pacingChart);
  }

  /**
   * Toggle detailed view
   */
  async toggleDetails(): Promise<void> {
    await this.click(this.selectors.detailToggle);
    await this.page.waitForTimeout(500);
  }

  /**
   * Get event log entries
   */
  async getEventLog(): Promise<string[]> {
    if (!await this.exists(this.selectors.eventLog)) return [];
    return await this.getTexts(this.selectors.logEntry);
  }

  /**
   * Check if event log is visible
   */
  async isEventLogVisible(): Promise<boolean> {
    return await this.exists(this.selectors.eventLog);
  }

  /**
   * Export log as text
   */
  async exportLog(): Promise<void> {
    await this.click(this.selectors.exportLogBtn);
  }

  /**
   * Export results as JSON
   */
  async exportJson(): Promise<void> {
    await this.click(this.selectors.exportJsonBtn);
  }

  /**
   * Get creature-specific statistics
   */
  async getCreatureStats(creatureName: string): Promise<{
    damageDealt: number;
    damageTaken: number;
    deaths: number;
    kills: number;
  } | null> {
    await this.waitForVisible(this.selectors.damageStats);
    return await this.evaluate((name) => {
      const rows = Array.from(document.querySelectorAll('[data-testid="stat-row"]'));
      for (const row of rows) {
        const nameEl = row.querySelector('[data-testid="creature-name"]');
        if (nameEl?.textContent === name) {
          const damageDealtText = row.querySelector('[data-testid="damage-dealt"]')?.textContent || '0';
          const damageTakenText = row.querySelector('[data-testid="damage-taken"]')?.textContent || '0';
          const deathsText = row.querySelector('[data-testid="deaths"]')?.textContent || '0';
          const killsText = row.querySelector('[data-testid="kills"]')?.textContent || '0';
          return {
            damageDealt: parseFloat(damageDealtText.replace(/[^\d.]/g, '')) || 0,
            damageTaken: parseFloat(damageTakenText.replace(/[^\d.]/g, '')) || 0,
            deaths: parseInt(deathsText, 10) || 0,
            kills: parseInt(killsText, 10) || 0,
          };
        }
      }
      return null;
    }, creatureName);
  }

  /**
   * Screenshot the results panel
   */
  async screenshot(name: string): Promise<void> {
    const element = await this.page.$(this.selectors.container);
    if (element) {
      const path = `e2e/screenshots/results-${name}.png`;
      await element.screenshot({ path });
    }
  }

  /**
   * Wait for specific result text to appear
   */
  async waitForResultText(text: string, timeout: number = 30000): Promise<void> {
    await this.page.waitForFunction(
      (searchText) => {
        return document.body.textContent?.includes(searchText);
      },
      { timeout },
      text
    );
  }
}
