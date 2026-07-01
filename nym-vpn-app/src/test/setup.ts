import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';
import { clearMocks } from '@tauri-apps/api/mocks';

// Ensure the DOM and any Tauri IPC mocks are reset between tests so state never
// leaks across cases. `clearMocks` tears down mocks installed via `mockIPC`.
afterEach(() => {
  cleanup();
  clearMocks();
});
