import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { AccountState } from '../types';

type TAccountState = (error: AccountState) => string;

/**
 * Hook to translate some account state
 *
 * @returns The translation function
 */
function useI18nAccountState() {
  const { t } = useTranslation('errors');

  const translateAccountState: TAccountState = useCallback(
    (state: AccountState) => {
      switch (state) {
        case 'bandwidth-exceeded':
          return t('account.bandwidth-exceeded');
        case 'status-not-active':
          return t('account.status-not-active');
        case 'no-subscription':
          return t('account.no-subscription');
        case 'max-device-reached':
          return t('account.max-devices-reached');
        default:
          return t('account.internal');
      }
    },
    [t],
  );

  return { tA: translateAccountState };
}

export default useI18nAccountState;
