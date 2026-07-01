import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { useAppStore } from '../../../../store';
import { initialState } from '../../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../../test/harness';
import Display from './Display';

// The `../../../../ui` barrel loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time and calls the Tauri OS plugin.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// `useSystemTheme` reads the window theme at mount; stub the webview window so
// it resolves deterministically without a real Tauri runtime.
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: () => ({ theme: () => Promise.resolve('light') }),
}));

afterEach(() => {
  seedStore({ ...initialState });
});

describe('Display', () => {
  it('renders the theme options and zoom section', () => {
    renderWithProviders(<Display />);

    expect(screen.getByText('Theme')).toBeInTheDocument();
    expect(screen.getByText('Automatic')).toBeInTheDocument();
    expect(screen.getByText('Light theme')).toBeInTheDocument();
    expect(screen.getByText('Dark theme')).toBeInTheDocument();
    expect(screen.getByTestId('zoom-section-title')).toBeInTheDocument();
  });

  it('selects the option matching the stored theme mode (system)', () => {
    seedStore({ themeMode: 'system' });
    renderWithProviders(<Display />);

    expect(screen.getByRole('radio', { name: /Automatic/ })).toBeChecked();
  });

  it('selects the option matching the stored theme mode (dark)', () => {
    seedStore({ themeMode: 'dark' });
    renderWithProviders(<Display />);

    expect(screen.getByRole('radio', { name: /Dark theme/ })).toBeChecked();
  });

  it('applies a new theme mode to the store and invokes set_background_color', async () => {
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return undefined;
    });
    seedStore({ themeMode: 'system' });
    renderWithProviders(<Display />);

    await act(async () => {
      await userEvent.click(screen.getByRole('radio', { name: /Dark theme/ }));
    });

    await waitFor(() => {
      expect(useAppStore.getState().themeMode).toBe('dark');
    });
    expect(useAppStore.getState().uiTheme).toBe('dark');
    await waitFor(() => {
      expect(calls).toContain('set_background_color');
    });
  });
});
