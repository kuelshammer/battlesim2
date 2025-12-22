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
      '@/utils': path.resolve(__dirname, './src/components/utils'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/model': path.resolve(__dirname, './src/model'),
      '@/styles': path.resolve(__dirname, './styles'),
      '@/data': path.resolve(__dirname, './src/data'),
      '@/pages': path.resolve(__dirname, './src/pages'),
      '@': path.resolve(__dirname, './src'),
    },
    css: {
      modules: {
        classNameStrategy: 'non-scoped',
      },
    },
  },
});