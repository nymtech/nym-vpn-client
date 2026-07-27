import { TFunction } from 'i18next';
import dayjs from 'dayjs';
import { AccountState, TAccountSummary } from '../../../types';

export const getAccountDescriptionColor = (
  accountSyncing: boolean,
  state?: AccountState | null,
  accountSummary?: TAccountSummary | null,
) => {
  if (accountSyncing) {
    return 'text-text-secondary';
  }
  if (
    state === 'no-subscription' ||
    state === 'bandwidth-exceeded' ||
    state === 'max-device-reached' ||
    state === 'error' ||
    state === 'pending-subscription'
  ) {
    return 'text-status-error';
  }
  if (state === 'offline' || state === 'status-not-active') {
    return 'text-status-warning';
  }
  if (!accountSummary?.isSubscriptionActive) {
    return 'text-status-error';
  }
  return 'text-text-secondary';
};

export const getAccountStateDescription = (
  t: TFunction<'settings', undefined>,
  accountSyncing: boolean,
  state?: AccountState | null,
  accountSummary?: TAccountSummary | null,
) => {
  if (accountSyncing) {
    return t('account.syncing');
  }

  if (!state) {
    return null;
  }

  switch (state) {
    case 'no-subscription':
      return t('account.no-plan');
    case 'max-device-reached':
      return t('account.max-device-reached');
    case 'status-not-active':
      return t('account.status-inactive');
    case 'bandwidth-exceeded':
      return t('account.bandwidth-exceeded');
    case 'pending-subscription':
      return t('account.pending-subscription', { ns: 'errors' });
    case 'offline':
      return t('account.offline', { ns: 'errors' });
    case 'error':
      return t('account.internal', { ns: 'errors' });
  }

  if (!accountSummary?.isSubscriptionActive) {
    return t('account.no-plan');
  }

  // Global default
  return null;
};

export const getAccountStatus = (accountSummary?: TAccountSummary | null) => {
  const subscription = accountSummary?.subscription?.subscription;
  if (
    !accountSummary ||
    subscription?.isRecurring ||
    accountSummary.isSubscriptionStacked
  ) {
    return 'green';
  }

  const diff = dayjs
    .unix(Number(subscription?.validUntilUtc))
    .diff(dayjs(), 'day');

  if (subscription?.kind === 'freepass' || subscription?.kind === 'one-month') {
    if (diff < 3) return 'amber'; // 2 days left
    if (diff < 8) return 'yellow'; // 7 days left
    return 'green';
  }

  // 1 & 2 years subscriptions
  if (diff < 31) return 'amber'; // 30 days left
  if (diff < 61) return 'yellow'; // 60 days left
  return 'green';
};
