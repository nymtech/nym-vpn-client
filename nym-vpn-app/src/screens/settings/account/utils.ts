import { TFunction } from 'i18next';
import { AccountState } from '../../../types';

export const getAccountColor = (
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

export const getAccountDescription = (
  t: TFunction<'settings', undefined>,
  accountSyncing: boolean,
  state?: AccountState | null,
) => {
  if (!state) {
    return null;
  }
  if (accountSyncing) {
    return t('account.syncing');
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
