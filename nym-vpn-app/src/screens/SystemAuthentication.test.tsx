import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { initialState } from '../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../test/harness';
import { SystemAuthentication } from './SystemAuthentication';

// The `../ui` barrel loads `DaemonDot` (reads `window._APP.devMode`) and the
// Tauri OS plugin at module-load time; the hoisted global + OS mock satisfy
// both before the static import above runs.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const invoke = vi.fn<(cmd: string) => Promise<void>>().mockResolvedValue();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string) => invoke(cmd),
}));

afterEach(() => {
  invoke.mockClear();
  seedStore({ ...initialState });
});

describe('SystemAuthentication', () => {
  it('keeps the dialog closed while the daemon is ok', () => {
    seedStore({ daemonStatus: 'ok' });

    renderWithProviders(<SystemAuthentication />);

    // A closed headlessui `Dialog` renders nothing, so its content is absent.
    expect(
      screen.queryByText('Authentication required'),
    ).not.toBeInTheDocument();
  });

  it('opens the dialog when authentication is denied', async () => {
    seedStore({ daemonStatus: 'auth-denied' });

    renderWithProviders(<SystemAuthentication />);

    expect(
      await screen.findByText('Authentication required'),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Authenticate' }),
    ).toBeInTheDocument();
  });

  it('invokes retry_authentication when the authenticate button is clicked', async () => {
    seedStore({ daemonStatus: 'auth-denied' });

    renderWithProviders(<SystemAuthentication />);

    await userEvent.click(
      await screen.findByRole('button', { name: 'Authenticate' }),
    );

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('retry_authentication'),
    );
  });
});
