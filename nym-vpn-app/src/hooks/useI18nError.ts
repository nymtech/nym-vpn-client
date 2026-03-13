import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorKey } from '../types';

/**
 * Hook to get the translation function for backend errors
 *
 * @returns The translation function
 */
function useI18nError() {
  const { t } = useTranslation('errors');

  const translate = useCallback(
    (error: ErrorKey) => {
      switch (error) {
        // mixnet event errors
        case 'entry-gw-down':
          return t('mixnet.entry-gateway-down');
        case 'exit-gw-down-ipv4':
          return t('mixnet.exit-gateway-down.ipv4');
        case 'exit-gw-down-ipv6':
          return t('mixnet.exit-gateway-down.ipv6');
        case 'exit-gw-routing-error-ipv4':
          return t('mixnet.exit-gateway-routing.ipv4');
        case 'exit-gw-routing-error-ipv6':
          return t('mixnet.exit-gateway-routing.ipv6');
        case 'mixnet-no-bandwidth':
          return t('mixnet.no-bandwidth');
        // general errors
        case 'internal':
          return t('internal');
        case 'vpnd-client':
          return t('vpnd-client');
        case 'not-connected-to-daemon':
          return t('daemon.not-connected');
        case 'auth-denied':
          return t('daemon.auth-denied');
        case 'account-invalid-mnemonic':
        case 'account-invalid-secret':
          return t('account.invalid-recovery-phrase');
        case 'get-mixnet-entry-countries-query':
          return t('countries-request.entry');
        case 'get-mixnet-exit-countries-query':
          return t('countries-request.exit');
        case 'get-wg-countries-query':
          return t('countries-request.fast-mode');
        // account related
        case 'no-account-stored':
          return t('account.no-account-stored');
        case 'no-device-stored':
          return t('account.no-device-stored');
        case 'existing-account':
          return t('account.existing-account');
        case 'account-status-not-active':
          return t('account.status-not-active');
        case 'no-subscription':
          return t('account.no-subscription');
        case 'max-device-reached':
          return t('account.max-devices-reached');
        case 'device-time-desync':
          return t('account.device-time-out-of-sync');
        case 'bandwidth-exceeded':
          return t('account.bandwidth-exceeded');
      }

      console.warn('unhandled backend error', error);
      return t('unknown');
    },
    [t],
  );

  return { tE: translate };
}

export default useI18nError;
