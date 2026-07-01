import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { renderWithProviders } from '../../../../test/harness';
import type { TAccountSummary } from '../../../../types';
import { RenewButton } from './RenewButton';

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
const startListening = vi.fn<() => Promise<string>>();

// Stub the hooks barrel so no toast provider is needed and the deeplink wait
// can be resolved synchronously in tests.
vi.mock('../../../../hooks', () => ({
  useToast: () => ({ add, close: vi.fn() }),
  useDeepLink: () => ({ startListening }),
}));

// A subscription that is neither recurring nor stacked and expires soon, so
// `getAccountStatus` resolves to "amber" and the renew button renders.
const expiresSoon = Math.floor(Date.now() / 1000) + 60 * 60 * 24; // 1 day

function makeSummary(
  overrides: Partial<TAccountSummary> = {},
): TAccountSummary {
  return {
    trafficUsedGb: 0n,
    trafficLimitGb: 5n,
    trafficResetTime: null,
    fairUsageDataUnavailable: false,
    accountAddr: 'addr',
    canonicalAccountAddr: 'addr',
    authMethods: [],
    isLinked: false,
    fairUsageLeft: true,
    isSubscriptionActive: true,
    isSubscriptionStacked: false,
    subscription: {
      status: 'active',
      subscription: {
        createdOnUtc: '',
        lastUpdatedUtc: '',
        id: 'sub-1',
        validUntilUtc: BigInt(expiresSoon),
        validFromUtc: 0n,
        status: 'active',
        kind: 'one-month',
        isRecurring: false,
      },
    },
    ...overrides,
  };
}

afterEach(() => {
  add.mockReset();
  startListening.mockReset();
});

describe('RenewButton', () => {
  it('renders the renew call-to-action for an expiring plan', () => {
    renderWithProviders(<RenewButton accountSummary={makeSummary()} />);

    expect(screen.getByText('Renew now to stay protected')).toBeInTheDocument();
  });

  it('renders nothing for a healthy (recurring) subscription', () => {
    const summary: TAccountSummary = {
      ...makeSummary(),
      subscription: {
        status: 'active',
        subscription: {
          createdOnUtc: '',
          lastUpdatedUtc: '',
          id: 'sub-1',
          validUntilUtc: BigInt(expiresSoon),
          validFromUtc: 0n,
          status: 'active',
          kind: 'one-month',
          isRecurring: true,
        },
      },
    };
    const { container } = renderWithProviders(
      <RenewButton accountSummary={summary} />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  it('handles the subscription payment after a successful renew flow', async () => {
    startListening.mockResolvedValue('nym://deeplink');
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      return undefined;
    });
    renderWithProviders(<RenewButton accountSummary={makeSummary()} />);

    await act(async () => {
      await userEvent.click(screen.getByText('Renew now to stay protected'));
    });

    await waitFor(() => {
      expect(startListening).toHaveBeenCalledOnce();
    });
    await waitFor(() => {
      expect(calls).toContain('handle_subscription_payment');
    });
    expect(add).not.toHaveBeenCalled();
  });
});
