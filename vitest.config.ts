import { defineConfig } from 'vitest/config';
// @ts-ignore
import react from '@vitejs/plugin-react';
// @ts-ignore
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    css: {
      modules: {
        classNameStrategy: 'non-scoped',
      },
    },
  },
});