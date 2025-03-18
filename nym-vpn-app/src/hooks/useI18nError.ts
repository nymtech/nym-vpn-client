import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorKey, TunnelError } from '../types';

// enforce that all errors are handled
type Terror = (error: ErrorKey | TunnelError) => string;

function fmtErr(msg: string, data?: string | null) {
  if (data && data.length > 0) {
    return `${msg} - ${data}`;
  }
  return msg;
}

/**
 * Hook to get the translation function for backend errors
 *
 * @returns The translation function
 */
function useI18nError() {
  const { t } = useTranslation('errors');

  const translateError: Terror = useCallback(
    (error: ErrorKey | TunnelError) => {
      if (typeof error === 'object') {
        // tunnel state errors
        switch (error.key) {
          case 'internal':
            return fmtErr(t('tunnel.internal'), error.data);
          case 'api':
            return fmtErr(t('tunnel.api'), error.data);
          case 'firewall':
            return fmtErr(t('tunnel.firewall'), error.data);
          case 'routing':
            return fmtErr(t('tunnel.routing'), error.data);
          case 'dns':
            return fmtErr(t('tunnel.dns'), error.data);
          case 'same-entry-and-exit-gw':
            return fmtErr(t('tunnel.same-entry-exit-gw'), error.data);
          case 'invalid-entry-gw-country':
            return fmtErr(t('tunnel.invalid-entry-gw-country'), error.data);
          case 'invalid-exit-gw-country':
            return fmtErr(t('tunnel.invalid-exit-gw-country'), error.data);
          case 'max-devices-reached':
            return fmtErr(t('tunnel.max-devices-reached'), error.data);
          case 'bandwidth-exceeded':
            return fmtErr(t('tunnel.bandwidth-exceeded'), error.data);
          case 'subscription-expired':
            return fmtErr(t('tunnel.subscription-expired'), error.data);
        }
      }
      // no tunnel errors
      switch (error) {
        // mixnet event errors
        case 'entry-gw-down':
          return t('entry-gateway-down');
        case 'exit-gw-down-ipv4':
          return t('exit-gateway-down.ipv4');
        case 'exit-gw-down-ipv6':
          return t('exit-gateway-down.ipv6');
        case 'exit-gw-routing-error-ipv4':
          return t('exit-gateway-routing.ipv4');
        case 'exit-gw-routing-error-ipv6':
          return t('exit-gateway-routing.ipv6');
        case 'no-bandwidth':
          return t('no-bandwidth');
        // general errors
        case 'internal-error':
          return t('internal');
        case 'not-connected-to-daemon':
          return t('daemon.not-connected');
        case 'grpc-error':
          return t('grpc');
        case 'account-invalid-mnemonic':
          return t('account.invalid-recovery-phrase');
        case 'account-storage':
          return t('account.storage');
        case 'account-is-connected':
          return t('account.is-connected');
        case 'get-mixnet-entry-countries-query':
          return t('countries-request.entry');
        case 'get-mixnet-exit-countries-query':
          return t('countries-request.exit');
        case 'get-wg-countries-query':
          return t('countries-request.fast-mode');
        case 'invalid-network-name':
          return t('daemon.invalid-network');
        case 'unknown-error':
          return t('unknown');
      }

      console.warn('unhandled backend error', error);
      return t('unknown');
    },
    [t],
  );

  return { tE: translateError };
}

export default useI18nError;
