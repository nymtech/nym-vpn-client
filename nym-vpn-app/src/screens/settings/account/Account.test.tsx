import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { initialState } from '../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../test/harness';
import type { TAccountSummary } from '../../../types';
import Account from './Account';

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

vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn() }));

const navigate = vi.fn();

vi.mock('react-router', async () => {
  const actual =
    await vi.importActual<typeof import('react-router')>('react-router');
  return { ...actual, useNavigate: () => navigate };
});

const logout = vi.fn();
const refresh = vi.fn().mockResolvedValue(undefined);
const add = vi.fn();

// Stub the provider-backed hooks the component (and its children) use, keeping
// the rest of the barrel intact so utilities like `useClipboard` still work.
vi.mock('../../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../hooks')>();
  return {
    ...actual,
    useLogout: () => ({ logout, loading: false }),
    useRefreshAccountSummary: () => ({ refresh, refreshing: false }),
    useDeepLink: () => ({ startListening: vi.fn() }),
    useToast: () => ({ add, close: vi.fn() }),
    // `useClipboard` (via `CardNewCopyableRow`) pulls the real `useToast`, which
    // needs a provider; stub it to keep the copyable rows rendering.
    useClipboard: () => ({ copy: vi.fn(), copied: false }),
  };
});

// Avoid the KV-backed cache reads; ids are resolved via `invoke` in the mock.
vi.mock('../../../cache', () => ({
  CCache: {
    get: vi.fn().mockResolvedValue(null),
    set: vi.fn(),
    del: vi.fn(),
  },
}));

const validUntil = Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 200;

function activeSummary(
  overrides: Partial<TAccountSummary> = {},
): TAccountSummary {
  return {
    trafficUsedGb: 1n,
    trafficLimitGb: 5n,
    trafficResetTime: BigInt(Math.floor(Date.now() / 1000) + 3600),
    fairUsageDataUnavailable: false,
    accountAddr: 'addr',
    canonicalAccountAddr: 'addr',
    authMethods: [],
    isLinked: false,
    fairUsageLeft: true,
    isSubscriptionActive: true,
    isSubscriptionStacked: true,
    subscription: {
      status: 'active',
      subscription: {
        createdOnUtc: '',
        lastUpdatedUtc: '',
        id: 'sub-1',
        validUntilUtc: BigInt(validUntil),
        validFromUtc: 0n,
        status: 'active',
        kind: 'one-year',
        isRecurring: true,
      },
    },
    ...overrides,
  };
}

function mockIds() {
  mockIPC((cmd) => {
    if (cmd === 'get_device_id') return 'device-123';
    if (cmd === 'get_canonical_account_id') return 'account-456';
    return undefined;
  });
}

afterEach(() => {
  logout.mockReset();
  refresh.mockClear();
  add.mockReset();
  navigate.mockReset();
  seedStore({ ...initialState });
});

describe('Account', () => {
  it('renders the logout button when logged in', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    expect(
      await screen.findByRole('button', { name: 'Log out' }),
    ).toBeInTheDocument();
  });

  it('resolves and displays the device and account ids', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    expect(await screen.findByText('device-123')).toBeInTheDocument();
    expect(screen.getByText('account-456')).toBeInTheDocument();
  });

  it('shows the choose-plan button for an account without a subscription', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'no-subscription',
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    expect(
      await screen.findByRole('button', { name: 'Choose plan' }),
    ).toBeInTheDocument();
  });

  it('shows the linked note when the account is linked', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'ready',
      accountSummary: activeSummary({ isLinked: true }),
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    expect(
      await screen.findByText('Backup login method linked via Privy.'),
    ).toBeInTheDocument();
  });

  it('refreshes the account summary on mount', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    await waitFor(() => {
      expect(refresh).toHaveBeenCalled();
    });
  });

  it('logs out when the logout button is clicked', async () => {
    mockIds();
    seedStore({
      account: true,
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<Account />);

    await userEvent.click(
      await screen.findByRole('button', { name: 'Log out' }),
    );

    expect(logout).toHaveBeenCalledOnce();
  });

  it('redirects to settings when there is no account', async () => {
    mockIds();
    seedStore({ account: false, daemonStatus: 'ok' });
    renderWithProviders(<Account />);

    await waitFor(() => {
      expect(navigate).toHaveBeenCalledWith('/settings', { replace: true });
    });
  });
});
