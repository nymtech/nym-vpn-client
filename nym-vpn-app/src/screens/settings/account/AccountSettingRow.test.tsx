import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { initialState } from '../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../test/harness';
import AccountSettingRow from './AccountSettingRow';

// The `../../../ui` barrel loads `DaemonDot`, which reads `window._APP.devMode`
// at module-load time and calls the Tauri OS plugin.
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

vi.mock('react-router', async () => {
  const actual =
    await vi.importActual<typeof import('react-router')>('react-router');
  return { ...actual, useNavigate: () => navigate };
});

afterEach(() => {
  navigate.mockReset();
  seedStore({ ...initialState });
});

describe('AccountSettingRow', () => {
  it('renders the get-started button when logged out', () => {
    // Keep the daemon down so the mount effect does not overwrite `account`.
    seedStore({ account: false, daemonStatus: 'down' });
    renderWithProviders(<AccountSettingRow />);

    expect(
      screen.getByRole('button', { name: 'Get started' }),
    ).toBeInTheDocument();
  });

  it('navigates to onboarding when get-started is clicked', async () => {
    // With the daemon up the button is enabled; the mount effect reports no
    // stored account so the logged-out branch stays rendered.
    mockIPC((cmd) => (cmd === 'is_account_stored' ? false : undefined));
    seedStore({ account: false, daemonStatus: 'ok' });
    renderWithProviders(<AccountSettingRow />);

    await userEvent.click(screen.getByRole('button', { name: 'Get started' }));

    expect(navigate).toHaveBeenCalledWith('/hideout/onboarding');
  });

  it('renders the account row when logged in', async () => {
    mockIPC((cmd) => (cmd === 'is_account_stored' ? true : undefined));
    seedStore({ account: true, accountState: 'ready', daemonStatus: 'ok' });
    renderWithProviders(<AccountSettingRow />);

    expect(
      await screen.findByRole('button', { name: /Account/ }),
    ).toBeInTheDocument();
  });

  it('shows the choose-plan button for an account without a subscription', async () => {
    mockIPC((cmd) => (cmd === 'is_account_stored' ? true : undefined));
    seedStore({
      account: true,
      accountState: 'no-subscription',
      daemonStatus: 'ok',
    });
    renderWithProviders(<AccountSettingRow />);

    expect(
      await screen.findByRole('button', { name: 'Choose plan' }),
    ).toBeInTheDocument();
  });

  it('navigates to the account screen when the account row is clicked', async () => {
    mockIPC((cmd) => (cmd === 'is_account_stored' ? true : undefined));
    seedStore({ account: true, accountState: 'ready', daemonStatus: 'ok' });
    renderWithProviders(<AccountSettingRow />);

    await userEvent.click(
      await screen.findByRole('button', { name: /Account/ }),
    );

    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith('/settings/account');
    });
  });
});
