import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
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
    rollupOptions: {
      output: {
        manualChunks: {
          // put the following packages in their own chunk
          // to reduce main chunk size
          tauri: [
            '@tauri-apps/api',
            '@tauri-apps/plugin-autostart',
            '@tauri-apps/plugin-clipboard-manager',
            '@tauri-apps/plugin-dialog',
            '@tauri-apps/plugin-notification',
            '@tauri-apps/plugin-opener',
            '@tauri-apps/plugin-os',
            '@tauri-apps/plugin-process',
            '@tauri-apps/plugin-updater',
            '@tauri-apps/plugin-window-state',
          ],
          motion: ['motion'],
          sentry: ['@sentry/react'],
          i18next: ['i18next', 'i18next-browser-languagedetector'],
          ui: [
            '@headlessui/react',
            '@radix-ui/react-accordion',
            '@radix-ui/react-slider',
            '@radix-ui/react-toast',
          ],
          lodash: ['lodash-es'],
        },
      },
    },
    chunkSizeWarningLimit: 800,
  },
}));
