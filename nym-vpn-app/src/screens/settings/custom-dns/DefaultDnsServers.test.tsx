import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Toast } from '@base-ui/react';
import { renderWithProviders, seedStore } from '../../../test/harness';
import { DefaultDnsServers } from './DefaultDnsServers';

// `DefaultDnsServers` reaches the `../../../ui` barrel via `useCustomDns` /
// `ButtonText`, which loads `DaemonDot` reading `window._APP.devMode` and calls
// the Tauri OS plugin's `type()` at module-load time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

afterEach(() => {
  seedStore({ defaultDns: [] });
});

describe('DefaultDnsServers', () => {
  it('renders the collapsed toggle and hides the list initially', () => {
    seedStore({ defaultDns: ['8.8.8.8'] });
    renderWithProviders(
      <Toast.Provider>
        <DefaultDnsServers />
      </Toast.Provider>,
    );

    expect(
      screen.getByRole('button', { name: 'View default DNS' }),
    ).toBeInTheDocument();
    expect(screen.queryByText('- 8.8.8.8')).not.toBeInTheDocument();
  });

  it('reveals the default DNS list from the store when expanded', async () => {
    const user = userEvent.setup();
    seedStore({ defaultDns: ['8.8.8.8', '8.8.4.4'] });
    renderWithProviders(
      <Toast.Provider>
        <DefaultDnsServers />
      </Toast.Provider>,
    );

    await user.click(screen.getByRole('button', { name: 'View default DNS' }));

    expect(screen.getByText('- 8.8.8.8')).toBeInTheDocument();
    expect(screen.getByText('- 8.8.4.4')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Hide default DNS' }),
    ).toBeInTheDocument();
  });
});
