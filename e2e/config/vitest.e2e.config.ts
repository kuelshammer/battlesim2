import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./e2e/config/setup.ts'],
    include: ['e2e/specs/**/*.e2e.ts'],
    exclude: ['node_modules', 'dist', '.next'],
    testTimeout: 30000,
    hookTimeout: 60000,
    teardownTimeout: 30000,
    reporter: ['default'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['e2e/', 'node_modules/', 'src/pages/api/'],
    },
    // E2E specific settings
    fileParallelism: false, // Run E2E tests sequentially
    // Integration with Puppeteer via custom matchers
    benchmark: {
      includeSamples: false,
    },
  },
  resolve: {
    alias: {
      '@': '/src',
      '@/components': '/src/components',
      '@/model': '/src/model',
      '@/styles': '/styles',
      '@/data': '/src/data',
      '@/pages': '/src/pages',
      '@/utils': '/src/components/utils',
    },
  },
});
