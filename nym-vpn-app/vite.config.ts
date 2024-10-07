import { resolve } from 'path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import svgr from 'vite-plugin-svgr';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), svgr()],
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
  // https://tauri.app/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ['VITE_', 'TAURI_', 'APP_'],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // put the following packages in their own chunk
          // to reduce main chunk size
          'framer-motion': ['framer-motion'],
          'mui-base': ['@mui/base'],
          sentry: ['@sentry/react'],
        },
      },
    },
  },
}));
