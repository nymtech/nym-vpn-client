import { afterEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { initialState } from '../../../store/slices/createMainSlice';
import { renderWithProviders, seedStore } from '../../../test/harness';
import type { TAccountSummary } from '../../../types';
import { AccountDescription } from './AccountDescription';

// A minimal summary with a fixed future expiry; individual tests override the
// subscription block to exercise the different rendering branches.
const validUntil = Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 200;

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

afterEach(() => {
  seedStore({ ...initialState });
});

describe('AccountDescription', () => {
  it('shows the syncing label while the account is syncing', () => {
    seedStore({ accountSyncing: true, accountState: null });
    renderWithProviders(<AccountDescription />);

    expect(screen.getByText('Syncing…')).toBeInTheDocument();
  });

  it('shows the no-plan label for a no-subscription account', () => {
    seedStore({ accountSyncing: false, accountState: 'no-subscription' });
    renderWithProviders(<AccountDescription />);

    expect(screen.getByText('No active plan')).toBeInTheDocument();
  });

  it('renders the plan expiry date for an active subscription', () => {
    seedStore({
      accountSyncing: false,
      accountState: 'ready',
      accountSummary: makeSummary(),
    });
    renderWithProviders(<AccountDescription />);

    expect(screen.getByText(/Plan valid until/)).toBeInTheDocument();
  });

  it('renders the auto-renew note for a recurring subscription', () => {
    seedStore({
      accountSyncing: false,
      accountState: 'ready',
      accountSummary: makeSummary(),
    });
    renderWithProviders(<AccountDescription />);

    expect(screen.getByText(/Auto-renews/)).toBeInTheDocument();
  });

  it('renders nothing when there is neither a description nor a valid subscription', () => {
    seedStore({
      accountSyncing: false,
      accountState: 'ready',
      accountSummary: makeSummary({
        isSubscriptionActive: true,
        subscription: null,
      }),
    });
    const { container } = renderWithProviders(<AccountDescription />);

    expect(container).toBeEmptyDOMElement();
  });
});
