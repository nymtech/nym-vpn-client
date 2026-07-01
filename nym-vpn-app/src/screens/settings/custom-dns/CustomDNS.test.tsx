import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { TopBarProvider } from '../../../contexts/topbar';
import { useAppStore } from '../../../store';
import { renderWithProviders, seedStore } from '../../../test/harness';
import CustomDNS from './CustomDNS';

// `CustomDNS` pulls UI from the `../../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
// module-load time. `vi.hoisted`/`vi.mock` run before the static import below
// so the global exists and the plugin is stubbed in time.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

// The toast hook is stubbed so no toast provider is needed and the "applied"
// toast can be asserted.
const addToast = vi.fn();
vi.mock('../../../hooks', () => ({
  useToast: () => ({ add: addToast, close: vi.fn() }),
}));

function renderCustomDns() {
  return renderWithProviders(
    <TopBarProvider>
      <CustomDNS />
    </TopBarProvider>,
  );
}

afterEach(() => {
  addToast.mockReset();
  seedStore({ customDnsEnabled: false, customDns: [], defaultDns: [] });
});

describe('CustomDNS', () => {
  it('renders the top description and default DNS toggle', () => {
    renderCustomDns();

    expect(
      screen.getByText(/using Nym's recommended DNS servers/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'View default DNS' }),
    ).toBeInTheDocument();
  });

  it('disables the custom DNS switch when there are no servers', () => {
    seedStore({ customDnsEnabled: false, customDns: [] });
    renderCustomDns();

    expect(screen.getByRole('switch')).toBeDisabled();
  });

  it('seeds the custom DNS list from the store', () => {
    seedStore({ customDnsEnabled: true, customDns: ['1.1.1.1', '9.9.9.9'] });
    renderCustomDns();

    expect(screen.getByText('1.1.1.1')).toBeInTheDocument();
    expect(screen.getByText('9.9.9.9')).toBeInTheDocument();
    expect(screen.getByText(/Custom DNS servers \(2\/5\)/)).toBeInTheDocument();
  });

  it('adds a valid DNS entry and applies it via set_custom_dns', async () => {
    const user = userEvent.setup();
    const calls: { cmd: string; payload?: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return undefined;
    });
    renderCustomDns();

    await user.type(screen.getByPlaceholderText(/address/i), '8.8.8.8');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByText('8.8.8.8')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Save changes' }));

    expect(calls).toContainEqual({
      cmd: 'set_custom_dns',
      payload: { dns: ['8.8.8.8'] },
    });
    expect(useAppStore.getState().customDns).toEqual(['8.8.8.8']);
    expect(addToast).toHaveBeenCalledWith({
      title: 'Custom DNS saved.',
      type: 'info',
    });
  });

  it('shows a validation error for an invalid DNS address', async () => {
    const user = userEvent.setup();
    renderCustomDns();

    await user.type(screen.getByPlaceholderText(/address/i), 'not-an-ip');
    await user.click(screen.getByRole('button', { name: 'Add' }));

    expect(screen.getByText('Invalid DNS address format')).toBeInTheDocument();
    expect(screen.queryByText('not-an-ip')).not.toBeInTheDocument();
  });
});
