import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, seedStore } from '../../../test/harness';
import type { Tunnel } from '../../../types';
import Dev from './Dev';

// `Dev` renders `NetworkEnvSelect` (which pulls `MsIcon` from the
// `../../../ui` barrel) and `PageAnim`; the barrel loads `DaemonDot` reading
// `window._APP.devMode` and calls the Tauri OS plugin's `type()` at load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

afterEach(() => {
  seedStore({
    daemonStatus: 'down',
    networkEnv: 'mainnet',
    tunnel: null,
    state: 'disconnected',
  });
});

describe('Dev', () => {
  it('renders the state value from the store', () => {
    seedStore({ state: 'connected' });
    renderWithProviders(<Dev />);

    expect(screen.getByTestId('dev-state-title')).toBeInTheDocument();
    expect(screen.getByTestId('dev-state-value')).toHaveTextContent(
      'connected',
    );
  });

  it('shows the network env selector when the daemon is up', () => {
    seedStore({ daemonStatus: 'ok', networkEnv: 'mainnet' });
    renderWithProviders(<Dev />);

    expect(
      screen.getByTestId('network-env-select-container'),
    ).toBeInTheDocument();
  });

  it('hides the network env selector when the daemon is down', () => {
    seedStore({ daemonStatus: 'down', networkEnv: 'mainnet' });
    renderWithProviders(<Dev />);

    expect(
      screen.queryByTestId('network-env-select-container'),
    ).not.toBeInTheDocument();
  });

  it('renders tunnel gateway details when a tunnel is present', () => {
    seedStore({
      tunnel: {
        entryGwId: 'entry-gw',
        exitGwId: 'exit-gw',
        data: {},
      } as unknown as Tunnel,
    });
    renderWithProviders(<Dev />);

    expect(screen.getByTestId('dev-tunnel-entry-gw')).toHaveTextContent(
      'entry-gw',
    );
    expect(screen.getByTestId('dev-tunnel-exit-gw')).toHaveTextContent(
      'exit-gw',
    );
  });

  it('omits the tunnel section when there is no tunnel', () => {
    seedStore({ tunnel: null });
    renderWithProviders(<Dev />);

    expect(
      screen.queryByTestId('dev-tunnel-container'),
    ).not.toBeInTheDocument();
  });
});
