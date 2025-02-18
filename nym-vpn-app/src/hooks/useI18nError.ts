import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorKey, TunnelError } from '../types';

// enforce that all errors are handled
type Terror = (error: ErrorKey | TunnelError) => string;

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
            return t('tunnel.internal');
          case 'firewall':
            return t('tunnel.firewall');
          case 'routing':
            return t('tunnel.routing');
          case 'dns':
            return t('tunnel.dns');
          case 'tun-device':
            return t('tunnel.tun-device');
          case 'tunnel-provider':
            return t('tunnel.provider');
          case 'same-entry-and-exit-gw':
            return t('tunnel.same-entry-exit-gw');
          case 'invalid-entry-gw-country':
            return t('tunnel.invalid-entry-gw-country');
          case 'invalid-exit-gw-country':
            return t('tunnel.invalid-exit-gw-country');
          case 'bad-bandwidth-increase':
            return t('tunnel.bad-bandwidth-increase');
          case 'duplicate-tun-fd':
            return t('tunnel.duplicate-tun-fd');
          case 'sync-account-no-account-stored':
            return t('sync-account', { reason: t('account.not-stored') });
          case 'sync-account-unexpected-response':
            return t('sync-account', {
              reason: t('account.unexpected-response'),
            });
          case 'sync-account-internal':
            return t('sync-account', { reason: t('account.internal') });
          case 'sync-account-vpn-api':
            return t('sync-account', { reason: t('account.vpn-api') });
          case 'sync-device-no-account-stored':
            return t('sync-device', { reason: t('account.not-stored') });
          case 'sync-device-no-device-stored':
            return t('sync-device', { reason: t('account.no-device-stored') });
          case 'sync-device-unexpected-response':
            return t('sync-device', {
              reason: t('account.unexpected-response'),
            });
          case 'sync-device-internal':
            return t('sync-device', { reason: t('account.internal') });
          case 'sync-device-vpn-api':
            return t('sync-device', { reason: t('account.vpn-api') });
          case 'register-device-no-account-stored':
            return t('register-device', { reason: t('account.not-stored') });
          case 'register-device-no-device-stored':
            return t('register-device', {
              reason: t('account.no-device-stored'),
            });
          case 'register-device-unexpected-response':
            return t('register-device', {
              reason: t('account.unexpected-response'),
            });
          case 'register-device-internal':
            return t('register-device', { reason: t('account.internal') });
          case 'register-device-vpn-api':
            return t('register-device', { reason: t('account.vpn-api') });
          case 'req-zknym-no-account-stored':
            return t('request-zknym', { reason: t('account.not-stored') });
          case 'req-zknym-no-device-stored':
            return t('request-zknym', {
              reason: t('account.no-device-stored'),
            });
          case 'req-zknym-unexpected-response':
            return t('request-zknym', {
              reason: t('account.unexpected-response'),
            });
          case 'req-zknym-storage':
            return t('request-zknym', { reason: t('account.storage') });
          case 'req-zknym-internal':
            return t('request-zknym', { reason: t('account.internal') });
          case 'req-zknym-vpn-api':
            return t('request-zknym', { reason: t('account.vpn-api') });
        }

        console.warn('unhandled tunnel error', error);
        return t('unknown');
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
