import { beforeAll, afterAll } from 'vitest';
import puppeteer from 'puppeteer';
import type { Browser, Page } from 'puppeteer';

declare global {
  var browser: Browser;
  var page: Page;
  var E2E_BASE_URL: string;
}

// Base URL for E2E tests - can be overridden via environment variable
const BASE_URL = process.env.E2E_BASE_URL || 'http://localhost:3000';
const HEADLESS = process.env.E2E_HEADLESS !== 'false'; // Default to headless, set to 'false' for debugging

beforeAll(async () => {
  console.log(`🎭 Launching Puppeteer (headless: ${HEADLESS})...`);
  console.log(`🌐 E2E Base URL: ${BASE_URL}`);

  global.browser = await puppeteer.launch({
    headless: HEADLESS,
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-dev-shm-usage',
      '--disable-gpu',
      ...(HEADLESS ? ['--headless=new'] : []),
    ],
    defaultViewport: {
      width: 1920,
      height: 1080,
    },
  });

  global.page = await global.browser.newPage();
  
  // Log browser console messages
  global.page.on('console', msg => {
    console.log(`[BROWSER] ${msg.type().toUpperCase()}: ${msg.text()}`);
  });

  global.E2E_BASE_URL = BASE_URL;

  // Set default timeout
  global.page.setDefaultTimeout(10000);

  // Ignore HTTPS errors for local development
  await global.page.setBypassCSP(true);

  console.log('✅ Puppeteer ready');
});

afterAll(async () => {
  if (global.page) {
    await global.page.close();
  }
  if (global.browser) {
    await global.browser.close();
  }
  console.log('🧹 Puppeteer cleaned up');
});

// Global test utilities
export const e2e = {
  navigate: async (path: string = '/') => {
    await global.page.goto(`${global.E2E_BASE_URL}${path}`, {
      waitUntil: 'networkidle0',
    });
  },

  screenshot: async (name: string) => {
    const screenshotPath = `e2e/screenshots/${name}.png`;
    await global.page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`📸 Screenshot saved: ${screenshotPath}`);
  },

  // Helper to wait for WASM initialization
  waitForWasmReady: async () => {
    await global.page.waitForFunction(
      () => {
        return (window as any).simulationWasm !== undefined;
      },
      { timeout: 15000 }
    );
  },

  // Helper to get LocalStorage state
  getLocalStorage: async () => {
    return await global.page.evaluate(() => {
      return { ...localStorage };
    });
  },

  // Helper to set LocalStorage state
  setLocalStorage: async (data: Record<string, string>) => {
    await global.page.evaluate((items) => {
      Object.entries(items).forEach(([key, value]) => {
        localStorage.setItem(key, value);
      });
    }, data);
  },
};
