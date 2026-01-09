import { BasePage } from './BasePage';

/**
 * Party Overview Page Object Model
 */
export class PartyOverviewPage extends BasePage {
  private currentContainer: string = '[data-testid="party-overview"]';

  /**
   * Set the container to target (e.g. overall summary vs encounter result)
   */
  setContainer(selector: string): void {
    this.currentContainer = selector;
  }

  private getSelector(subSelector: string): string {
    return `${this.currentContainer} ${subSelector}`;
  }

  /**
   * Check if Party Overview is visible
   */
  async isVisible(): Promise<boolean> {
    return await this.exists(this.currentContainer);
  }

  /**
   * Get canvas dimensions
   */
  async getCanvasDimensions(): Promise<{ width: number; height: number }> {
    const canvasSelector = this.getSelector('[data-testid="party-overview-canvas"]');
    await this.waitForVisible(canvasSelector);
    return await this.evaluate((sel) => {
      const canvas = document.querySelector(sel) as HTMLCanvasElement;
      return {
        width: canvas.clientWidth,
        height: canvas.clientHeight,
      };
    }, canvasSelector);
  }

  /**
   * Get the number of player tags in the legend
   */
  async getPlayerTagCount(): Promise<number> {
    const legendSelector = this.getSelector('[data-testid="party-overview-legend"]');
    await this.waitForVisible(legendSelector);
    const tagSelector = this.getSelector('[data-testid^="player-tag-"]');
    return await this.evaluate((sel) => {
      return document.querySelectorAll(sel).length;
    }, tagSelector);
  }

  /**
   * Get all player names from the legend
   */
  async getPlayerNames(): Promise<string[]> {
    const tagSelector = this.getSelector('[data-testid^="player-tag-"]');
    return await this.getTexts(tagSelector);
  }

  /**
   * Get color key items count
   */
  async getColorKeyCount(): Promise<number> {
    const swatchSelector = this.getSelector('[data-testid^="swatch-"]');
    return await this.evaluate((sel) => {
      return document.querySelectorAll(sel).length;
    }, swatchSelector);
  }

  /**
   * Hover over the canvas at a specific percentile
   * @param percentile 0-99
   */
  async hoverPercentile(percentile: number): Promise<void> {
    const dims = await this.getCanvasDimensions();
    const canvasSelector = this.getSelector('[data-testid="party-overview-canvas"]');
    const canvas = await this.page.$(canvasSelector);
    if (!canvas) throw new Error('Canvas not found');

    const rect = await canvas.boundingBox();
    if (!rect) throw new Error('Canvas bounding box not found');

    const x = rect.x + (rect.width * (percentile + 0.5) / 100);
    const y = rect.y + (rect.height / 2);

    await this.page.mouse.move(x, y);
    // Wait for tooltip/crosshair to potentially react
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  /**
   * Get the data from the canvas at a specific point
   * (Uses getImageData to verify colors)
   */
  async getPixelColor(xPercent: number, yPercent: number): Promise<{ r: number; g: number; b: number; a: number }> {
    return await this.evaluate((sel, xP, yP) => {
      const canvas = document.querySelector(sel) as HTMLCanvasElement;
      const ctx = canvas.getContext('2d');
      if (!ctx) throw new Error('Could not get canvas context');
      
      const x = Math.floor(canvas.width * xP / 100);
      const y = Math.floor(canvas.height * yP / 100);
      const pixel = ctx.getImageData(x, y, 1, 1).data;
      
      return {
        r: pixel[0],
        g: pixel[1],
        b: pixel[2],
        a: pixel[3],
      };
    }, this.selectors.canvas, xPercent, yPercent);
  }
}
