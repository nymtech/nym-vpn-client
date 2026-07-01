import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders } from '../../../../test/harness';
import type { TAccountSummary } from '../../../../types';
import { ActivePlan } from './ActivePlan';

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

// `ActivePlan` renders `RenewButton`, which pulls the hooks barrel; stub it so
// no toast/deeplink providers are needed.
vi.mock('../../../../hooks', () => ({
  useToast: () => ({ add: vi.fn(), close: vi.fn() }),
  useDeepLink: () => ({ startListening: vi.fn() }),
}));

const validUntil = Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 200;

function makeSummary(
  overrides: Partial<TAccountSummary> = {},
): TAccountSummary {
  return {
    trafficUsedGb: 2n,
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

describe('ActivePlan', () => {
  it('renders the bandwidth usage figures', () => {
    renderWithProviders(<ActivePlan accountSummary={makeSummary()} />);

    expect(screen.getByText('Daily allowance used')).toBeInTheDocument();
    expect(screen.getByText('2 GB')).toBeInTheDocument();
    expect(screen.getByText('5 GB')).toBeInTheDocument();
  });

  it('shows the fallback message when fair-usage data is unavailable', () => {
    renderWithProviders(
      <ActivePlan
        accountSummary={makeSummary({ fairUsageDataUnavailable: true })}
      />,
    );

    expect(
      screen.getByText('Usage data temporarily unavailable'),
    ).toBeInTheDocument();
    expect(screen.queryByText('Daily allowance used')).not.toBeInTheDocument();
  });

  it('renders the daily reset row', () => {
    renderWithProviders(<ActivePlan accountSummary={makeSummary()} />);

    expect(screen.getByText('Resets daily')).toBeInTheDocument();
  });

  it('shows the reset-unknown label when no reset time is set', () => {
    renderWithProviders(
      <ActivePlan accountSummary={makeSummary({ trafficResetTime: null })} />,
    );

    expect(screen.getByText('Unknown')).toBeInTheDocument();
  });

  it('hides the renew button for a recurring, stacked subscription', () => {
    renderWithProviders(<ActivePlan accountSummary={makeSummary()} />);

    expect(
      screen.queryByText('Renew now to stay protected'),
    ).not.toBeInTheDocument();
  });

  it('shows the renew button when the subscription is not recurring', () => {
    const summary = makeSummary({
      isSubscriptionStacked: false,
      subscription: {
        status: 'active',
        subscription: {
          createdOnUtc: '',
          lastUpdatedUtc: '',
          id: 'sub-1',
          validUntilUtc: BigInt(Math.floor(Date.now() / 1000) + 60 * 60 * 24),
          validFromUtc: 0n,
          status: 'active',
          kind: 'one-month',
          isRecurring: false,
        },
      },
    });
    renderWithProviders(<ActivePlan accountSummary={summary} />);

    expect(screen.getByText('Renew now to stay protected')).toBeInTheDocument();
  });
});
