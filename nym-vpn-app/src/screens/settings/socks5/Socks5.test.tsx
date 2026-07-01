import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { Socks5Status } from '../../../types';
import { renderWithProviders, seedStore } from '../../../test/harness';
import Socks5 from './Socks5';

// `Socks5` pulls UI from the `../../../ui` barrel, which loads modules reading
// `window._APP.devMode` and calling the Tauri OS plugin's `type()` at
// module-load time. `vi.hoisted`/`vi.mock` run before the imports.
vi.hoisted(() => {
  (globalThis as unknown as { _APP: { devMode: boolean } })._APP = {
    devMode: false,
  };
});

vi.mock('@tauri-apps/plugin-os', () => ({
  type: () => 'linux',
  platform: () => 'linux',
}));

const add = vi.fn();

// `Socks5` pulls `useToast` from the hooks barrel (and `ProxyUrl` pulls
// `useClipboard` from the same module); stub both so the toast manager provider
// isn't required and toast calls can be asserted.
vi.mock('../../../hooks/index', () => ({
  useToast: () => ({ add, close: vi.fn() }),
  useClipboard: () => ({ copy: vi.fn() }),
}));

const enable = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
const disable = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);

function seedSocks5(status: Socks5Status | null) {
  seedStore({
    status,
    isLoading: false,
    enable,
    disable,
    refresh: vi.fn().mockResolvedValue(undefined),
  });
}

const disabledStatus: Socks5Status = {
  state: 'disabled',
  socks5Settings: null,
  httpRpcSettings: null,
  errorMessage: null,
  activeConnections: 0,
};

const connectedStatus: Socks5Status = {
  state: 'connected',
  socks5Settings: { listenAddress: '127.0.0.1:1080' },
  httpRpcSettings: { listenAddress: '127.0.0.1:8545' },
  errorMessage: null,
  activeConnections: 3,
};

afterEach(() => {
  add.mockReset();
  enable.mockClear();
  disable.mockClear();
});

describe('Socks5', () => {
  it('renders the intro, enable switch and proxy configuration cards', () => {
    seedSocks5(disabledStatus);
    renderWithProviders(<Socks5 />);

    expect(screen.getByText(/Route app & wallet traffic/)).toBeInTheDocument();
    expect(screen.getByText('Enable proxy')).toBeInTheDocument();
    expect(
      screen.getByText('SOCKS5 configuration (for apps)'),
    ).toBeInTheDocument();
    expect(
      screen.getByText('HTTP RPC configuration (for wallets)'),
    ).toBeInTheDocument();
  });

  it('shows the disabled status and no active connections when disabled', () => {
    seedSocks5(disabledStatus);
    renderWithProviders(<Socks5 />);

    expect(screen.getByText('Disabled')).toBeInTheDocument();
    expect(screen.getByText('0')).toBeInTheDocument();
    expect(screen.getByTestId('switch')).toHaveAttribute(
      'data-test-checked',
      'false',
    );
  });

  it('shows the connected status, active connections and URLs when connected', () => {
    seedSocks5(connectedStatus);
    renderWithProviders(<Socks5 />);

    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByText('socks5h://127.0.0.1:1080')).toBeInTheDocument();
    expect(screen.getByTestId('switch')).toHaveAttribute(
      'data-test-checked',
      'true',
    );
  });

  it('enables the proxy and shows a toast when the switch is clicked while disabled', async () => {
    seedSocks5(disabledStatus);
    renderWithProviders(<Socks5 />);

    await act(async () => {
      await userEvent.click(screen.getByText('Enable proxy'));
    });

    expect(enable).toHaveBeenCalledOnce();
    await waitFor(() =>
      expect(add).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'socks5-enabled' }),
      ),
    );
  });

  it('disables the proxy and shows a toast when the switch is clicked while enabled', async () => {
    seedSocks5(connectedStatus);
    renderWithProviders(<Socks5 />);

    await act(async () => {
      await userEvent.click(screen.getByText('Enable proxy'));
    });

    expect(disable).toHaveBeenCalledOnce();
    await waitFor(() =>
      expect(add).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'socks5-disabled' }),
      ),
    );
  });

  it('surfaces an error toast when the state reports an error', () => {
    seedSocks5({
      ...disabledStatus,
      state: 'error',
      errorMessage: 'boom',
    });
    renderWithProviders(<Socks5 />);

    expect(add).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'socks5-error', title: 'boom' }),
    );
  });
});
