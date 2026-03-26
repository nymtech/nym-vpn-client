import { TFunction } from 'i18next';
import dayjs from 'dayjs';
import { AccountState, TAccountSummary } from '../../../types';

export const getAccountDescriptionColor = (
  accountSyncing: boolean,
  state?: AccountState | null,
) => {
  if (accountSyncing) {
    return 'text-iron dark:text-bombay';
  }
  if (
    state === 'no-subscription' ||
    state === 'bandwidth-exceeded' ||
    state === 'max-device-reached' ||
    state === 'error'
  ) {
    return 'text-aphrodisiac';
  }
  if (state === 'offline' || state === 'status-not-active') {
    return 'text-cheddar dark:text-king-nacho ';
  }
  return 'text-iron dark:text-bombay';
};

export const getAccountStateDescription = (
  t: TFunction<'settings', undefined>,
  accountSyncing: boolean,
  state?: AccountState | null,
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
    case 'requesting-zk-nyms':
      return t('account.requesting-zknyms');
    case 'offline':
    case 'error':
      return t('account.error');
    default:
      return null;
  }
};

export const getAccountStatus = (accountSummary?: TAccountSummary | null) => {
  if (!accountSummary || accountSummary?.['is-recurring']) {
    return 'green';
  }

  const diff = dayjs
    .unix(Number(accountSummary?.['subscription-valid-until']))
    .diff(dayjs(), 'day');

  if (
    accountSummary?.['subscription-kind'] === 'freepass' ||
    accountSummary?.['subscription-kind'] === 'one-month'
  ) {
    if (diff < 3) return 'amber'; // 2 days left
    if (diff < 8) return 'yellow'; // 7 days left
    return 'green';
  }

  // 1 & 2 years subscriptions
  if (diff < 31) return 'amber'; // 30 days left
  if (diff < 61) return 'yellow'; // 60 days left
  return 'green';
};
