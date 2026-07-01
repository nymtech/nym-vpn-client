import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { renderWithProviders, seedStore } from '../../../test/harness';
import InfoData from './InfoData';

// `InfoData` reads `window._APP.devMode` at module-load time, and pulls
// `ButtonText` from the `../../../ui` barrel, which loads `DaemonDot` (also
// reading `window._APP.devMode`) and calls the Tauri OS plugin's `type()`.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: true,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

afterEach(() => {
  navigate.mockReset();
  seedStore({
    version: '1.30.0',
    daemonStatus: 'down',
    daemonVersion: undefined,
    networkEnv: 'mainnet',
  });
});

describe('InfoData', () => {
  it('renders the client version from the store', () => {
    seedStore({ version: '1.30.0', daemonStatus: 'down' });
    renderWithProviders(
      <Toast.Provider>
        <InfoData />
      </Toast.Provider>,
    );

    expect(screen.getByText('App version')).toBeInTheDocument();
    expect(screen.getByTestId('client-version-value')).toHaveTextContent(
      '1.30.0',
    );
  });

  it('hides daemon details while the daemon is down', () => {
    seedStore({ daemonStatus: 'down', daemonVersion: '1.30.0' });
    renderWithProviders(
      <Toast.Provider>
        <InfoData />
      </Toast.Provider>,
    );

    expect(
      screen.queryByTestId('daemon-version-container'),
    ).not.toBeInTheDocument();
  });

  it('shows daemon version and network name once the daemon is up', () => {
    seedStore({
      daemonStatus: 'ok',
      daemonVersion: '1.30.1',
      networkEnv: 'mainnet',
    });
    renderWithProviders(
      <Toast.Provider>
        <InfoData />
      </Toast.Provider>,
    );

    expect(screen.getByTestId('daemon-version-value')).toHaveTextContent(
      '1.30.1',
    );
    expect(screen.getByTestId('network-name-value')).toHaveTextContent(
      'mainnet',
    );
  });

  it('navigates to the dev screen on double-clicking the version in dev mode', async () => {
    const user = userEvent.setup();
    seedStore({ version: '1.30.0', daemonStatus: 'down' });
    renderWithProviders(
      <Toast.Provider>
        <InfoData />
      </Toast.Provider>,
    );

    await user.dblClick(screen.getByTestId('client-version-value'));

    expect(navigate).toHaveBeenCalledOnce();
  });
});
