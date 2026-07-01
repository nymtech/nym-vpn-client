import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockTauriCommands, renderWithProviders } from '../../../test/harness';
import Logs from './Logs';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const openPath = vi.fn();
vi.mock('@tauri-apps/plugin-opener', () => ({
  openPath: (path: string) => {
    openPath(path);
  },
}));

describe('Logs', () => {
  it('renders the app and daemon log entries', () => {
    renderWithProviders(<Logs />);

    expect(screen.getByTestId('logs-page')).toBeInTheDocument();
    expect(screen.getByText('App logs')).toBeInTheDocument();
    expect(screen.getByText('Daemon logs')).toBeInTheDocument();
  });

  it('opens the app log directory returned by the backend', async () => {
    mockTauriCommands((cmd) => {
      if (cmd === 'log_dir') return '/tmp/app-logs';
      return undefined;
    });
    renderWithProviders(<Logs />);

    await userEvent.click(screen.getByText('App logs'));

    await waitFor(() => expect(openPath).toHaveBeenCalledWith('/tmp/app-logs'));
  });

  it('does not open a path when the backend returns no directory', async () => {
    mockTauriCommands(() => undefined);
    renderWithProviders(<Logs />);

    await userEvent.click(screen.getByText('Daemon logs'));

    // give the resolved promise a chance to run before asserting nothing opened
    await Promise.resolve();
    expect(openPath).not.toHaveBeenCalled();
  });
});
