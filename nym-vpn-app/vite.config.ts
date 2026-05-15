import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import svgr from 'vite-plugin-svgr';

// https://vitejs.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwindcss(), svgr()],
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  // 3. to make use of `TAURI_DEBUG` and other env variables
  envPrefix: ['VITE_', 'TAURI_', 'APP_'],
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: 'tauri',
              test: /@tauri-apps/,
              priority: 70,
            },
            {
              name: 'ui',
              test: /@(headlessui|radix-ui|base-ui)/,
              priority: 60,
            },
            {
              name: 'lodash',
              test: /lodash/,
              priority: 50,
            },
            {
              name: 'lottie',
              test: /lottie/,
              priority: 40,
            },
            {
              name: 'motion',
              test: /[\\/]motion[\\/]/,
              priority: 30,
            },
            {
              name: 'i18next',
              test: /i18next/,
              priority: 20,
            },
            {
              name: 'react',
              test: /node_modules[\\/](react|react-dom|react-router|react-window|react-is|scheduler|use-sync-external-store)([\\/]|$)/,
              priority: 10,
            },
          ],
        },
      },
    },
    chunkSizeWarningLimit: 800,
  },
}));
