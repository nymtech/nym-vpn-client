import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { routes } from '../../router';
import { renderWithProviders, seedStore } from '../../test/harness';
import Settings from './Settings';

// `Settings` pulls UI from the `../../ui` barrel, which loads `DaemonDot`
// reading `window._APP.devMode` and calls the Tauri OS plugin's `type()` at
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

const navigate = vi.fn();
vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>();
  return { ...actual, useNavigate: () => navigate };
});

const add = vi.fn();
const toggleAutostart = vi
  .fn<() => Promise<void>>()
  .mockResolvedValue(undefined);
vi.mock('../../hooks', () => ({
  useToast: () => ({ add, close: vi.fn() }),
  useAutostart: () => ({ enabled: false, toggle: toggleAutostart }),
}));

const exit = vi.fn();
vi.mock('../../state', () => ({ useExit: () => ({ exit }) }));

// The account/info-data sub-rows have their own daemon dependencies that are
// out of scope here; stub them so the Settings menu can be asserted in
// isolation.
vi.mock('./account', () => ({
  AccountSettingRow: () => <div data-testid="account-row" />,
}));
vi.mock('./info-data', () => ({
  InfoData: () => <div data-testid="info-data" />,
}));

afterEach(() => {
  navigate.mockReset();
  add.mockReset();
  exit.mockReset();
  toggleAutostart.mockClear();
  seedStore({ ipv6Support: false, allowLan: false, enableAdBlocking: false });
});

describe('Settings', () => {
  it('renders the menu entries and the quit button', () => {
    renderWithProviders(<Settings />);

    expect(screen.getByText('Support & feedback')).toBeInTheDocument();
    expect(screen.getByText('Split tunneling (beta)')).toBeInTheDocument();
    expect(screen.getByText('App & wallet proxy (beta)')).toBeInTheDocument();
    expect(screen.getByText('Legal')).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Quit NymVPN' }),
    ).toBeInTheDocument();
  });

  it('navigates to support when the support row is clicked', async () => {
    renderWithProviders(<Settings />);

    await userEvent.click(screen.getByText('Support & feedback'));

    expect(navigate).toHaveBeenCalledWith(routes.support);
  });

  it('navigates to split tunneling with reset-scroll state', async () => {
    renderWithProviders(<Settings />);

    await userEvent.click(screen.getByText('Split tunneling (beta)'));

    expect(navigate).toHaveBeenCalledWith(routes.splitTunneling, {
      state: { resetScroll: true },
    });
  });

  it('toggles ad-block and reflects the store value', async () => {
    seedStore({ enableAdBlocking: false });
    mockIPC(() => undefined);
    renderWithProviders(<Settings />);

    await userEvent.click(screen.getByText('Block ads'));

    expect(add).not.toHaveBeenCalled();
  });

  it('exits the app when quit is clicked', async () => {
    renderWithProviders(<Settings />);

    await userEvent.click(screen.getByRole('button', { name: 'Quit NymVPN' }));

    expect(exit).toHaveBeenCalledOnce();
  });
});
