import type { Page } from 'puppeteer';

/**
 * Base Page Object Model
 * Provides common functionality for all page objects
 */
export class BasePage {
  constructor(protected page: Page) {}

  /**
   * Navigate to a path
   */
  async goto(path: string = '/'): Promise<void> {
    await this.page.goto(`${process.env.E2E_BASE_URL || 'http://localhost:3000'}${path}`, {
      waitUntil: 'networkidle0',
    });
  }

  /**
   * Wait for an element to be visible
   */
  async waitForVisible(selector: string, timeout: number = 5000): Promise<void> {
    await this.page.waitForSelector(selector, {
      visible: true,
      timeout,
    });
  }

  /**
   * Wait for an element to exist
   */
  async waitForExists(selector: string, timeout: number = 5000): Promise<void> {
    await this.page.waitForSelector(selector, {
      timeout,
    });
  }

  /**
   * Click an element
   */
  async click(selector: string): Promise<void> {
    await this.page.waitForSelector(selector, { visible: true });
    await this.page.click(selector);
  }

  /**
   * Type text into an input
   */
  async type(selector: string, text: string): Promise<void> {
    await this.page.waitForSelector(selector, { visible: true });
    await this.page.type(selector, text);
  }

  /**
   * Get text content of an element
   */
  async getText(selector: string): Promise<string> {
    await this.page.waitForSelector(selector, { visible: true });
    const element = await this.page.$(selector);
    if (!element) throw new Error(`Element not found: ${selector}`);
    return await (await element.getProperty('textContent')).jsonValue() as string;
  }

  /**
   * Get multiple text contents
   */
  async getTexts(selector: string): Promise<string[]> {
    await this.page.waitForSelector(selector, { visible: true });
    return await this.page.evaluate((sel) => {
      return Array.from(document.querySelectorAll(sel))
        .map(el => el.textContent || '');
    }, selector);
  }

  /**
   * Check if element exists
   */
  async exists(selector: string): Promise<boolean> {
    return await this.page.$(selector) !== null;
  }

  /**
   * Wait for network idle
   */
  async waitForNetworkIdle(timeout: number = 5000): Promise<void> {
    await this.page.waitForNetworkIdle({ idleTime: 500, timeout });
  }

  /**
   * Screenshot helper
   */
  async screenshot(name: string): Promise<void> {
    const path = `e2e/screenshots/${name}.png`;
    await this.page.screenshot({ path, fullPage: true });
  }

  /**
   * Evaluate JavaScript in the browser context
   */
  async evaluate<T>(fn: () => T): Promise<T> {
    return await this.page.evaluate(fn);
  }
}
