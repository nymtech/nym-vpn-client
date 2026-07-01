import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import svgr from 'vite-plugin-svgr';

// Dedicated test config, kept separate from vite.config.ts so the tauri-tailored
// build path (fixed port, rolldown output groups) is untouched by test-only options.
export default defineConfig({
  plugins: [react(), svgr()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    // CSS is irrelevant to behavior tests; skip processing for speed.
    css: false,
    clearMocks: true,
    restoreMocks: true,
  },
});
