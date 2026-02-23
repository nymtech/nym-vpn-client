import { defineConfig } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(__dirname, '..');

export default defineConfig({
  testDir: './tests',
  outputDir: path.join(appRoot, 'test-results'),
  timeout: 30_000,
  expect: {
    timeout: 10_000,
  },
  fullyParallel: true,
  retries: 1,
  reporter: [
    ['list'],
    [
      'html',
      {
        outputFolder: path.join(appRoot, 'playwright-report'),
        open: process.env.CI ? 'never' : 'on-failure',
      },
    ],
  ],
  use: {
    baseURL: 'http://localhost:1420',
    headless: true,
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'npm run dev:browser',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    cwd: appRoot,
    timeout: 30_000,
  },
});
