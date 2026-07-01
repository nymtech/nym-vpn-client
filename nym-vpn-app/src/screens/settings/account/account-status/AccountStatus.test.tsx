import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { initialState } from '../../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../../test/harness';
import type { TAccountSummary } from '../../../../types';
import { AccountStatus } from './AccountStatus';

// The `../../../../ui` barrel loads `DaemonDot`, which reads
// `window._APP.devMode` at module-load time and calls the Tauri OS plugin.
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

// `AccountStatus` (and the nested `RenewButton`) pull the hooks barrel; stub it
// so no toast/deeplink providers are required and toasts can be asserted.
vi.mock('../../../../hooks', () => ({
  useToast: () => ({ add, close: vi.fn() }),
  useDeepLink: () => ({ startListening: vi.fn() }),
}));

const validUntil = Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 200;

function activeSummary(): TAccountSummary {
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
  };
}

afterEach(() => {
  add.mockReset();
  seedStore({ ...initialState });
});

describe('AccountStatus', () => {
  it('renders nothing when the account state is unset', () => {
    seedStore({ accountState: null });
    const { container } = renderWithProviders(
      <AccountStatus refresh={vi.fn()} refreshing={false} />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  it('renders nothing when the account state is an error', () => {
    seedStore({ accountState: 'error' });
    const { container } = renderWithProviders(
      <AccountStatus refresh={vi.fn()} refreshing={false} />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  it('shows the no-plan state when a subscription is needed', () => {
    seedStore({
      accountState: 'no-subscription',
      accountSummary: null,
    });
    renderWithProviders(<AccountStatus refresh={vi.fn()} refreshing={false} />);

    expect(screen.getByText('Account status')).toBeInTheDocument();
    expect(screen.getByText('No active plan')).toBeInTheDocument();
  });

  it('shows the active plan when a subscription is active', () => {
    seedStore({
      accountState: 'ready',
      accountSummary: activeSummary(),
    });
    renderWithProviders(<AccountStatus refresh={vi.fn()} refreshing={false} />);

    expect(screen.getByText('Daily allowance used')).toBeInTheDocument();
  });

  it('invokes the refresh callback when the refresh button is clicked', async () => {
    const refresh = vi.fn().mockResolvedValue(undefined);
    seedStore({
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<AccountStatus refresh={refresh} refreshing={false} />);

    await act(async () => {
      await userEvent.click(screen.getByTestId('refresh-account-summary'));
    });

    expect(refresh).toHaveBeenCalledOnce();
    expect(add).not.toHaveBeenCalled();
  });

  it('shows an error toast when the refresh callback rejects', async () => {
    const refresh = vi.fn().mockRejectedValue(new Error('nope'));
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(vi.fn());
    seedStore({
      accountState: 'ready',
      accountSummary: activeSummary(),
      daemonStatus: 'ok',
    });
    renderWithProviders(<AccountStatus refresh={refresh} refreshing={false} />);

    await act(async () => {
      await userEvent.click(screen.getByTestId('refresh-account-summary'));
    });

    expect(add).toHaveBeenCalledExactlyOnceWith({
      id: 'refresh-account-state-error',
      title: 'Failed to refresh account',
      type: 'error',
    });
    errorSpy.mockRestore();
  });
});
