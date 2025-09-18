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
        const { message } = error;
        // tunnel state errors
        switch (error.key) {
          case 'tun-device':
            return fmtErr(t('tunnel.tun-device'), message);
          case 'tunnel-provider':
            return fmtErr(t('tunnel.tunnel-provider'), message);
          case 'inactive-account':
            return fmtErr(t('tunnel.inactive-account'), message);
          case 'device-logged-out':
            return fmtErr(t('tunnel.device-logged-out'), message);
          case 'internal':
            return fmtErr(t('tunnel.internal'), message);
          case 'set-firewall-policy':
            return fmtErr(t('tunnel.firewall'), message);
          case 'set-routing':
            return fmtErr(t('tunnel.routing'), message);
          case 'set-dns':
            return fmtErr(t('tunnel.dns'), message);
          case 'same-entry-and-exit-gw':
            return fmtErr(t('tunnel.same-entry-exit-gw'), message);
          case 'invalid-entry-gw-country':
            return fmtErr(t('tunnel.invalid-entry-gw-country'), message);
          case 'invalid-exit-gw-country':
            return fmtErr(t('tunnel.invalid-exit-gw-country'), message);
          case 'max-devices-reached':
            return fmtErr(t('tunnel.max-devices-reached'), message);
          case 'bandwidth-exceeded':
            return fmtErr(t('tunnel.bandwidth-exceeded'), message);
          case 'inactive-subscription':
            return fmtErr(t('tunnel.subscription-expired'), message);
          case 'device-time-out-of-sync':
            return fmtErr(t('tunnel.device-time-out-of-sync'), message);
          case 'ipv6-unavailable':
            return fmtErr(t('tunnel.ipv6-unavailable'), message);
          case 'credential-wasted-on-entry-gateway':
            return fmtErr(
              t('tunnel.entry-gateway-bandwidth-increase'),
              message,
            );
          case 'credential-wasted-on-exit-gateway':
            return fmtErr(t('tunnel.exit-gateway-bandwidth-increase'), message);
        }
      }
      // not a tunnel error
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
        case 'grpc':
          return t('grpc');
        case 'not-connected-to-daemon':
          return t('daemon.not-connected');
        case 'account-invalid-mnemonic':
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
        case 'bandwidth-exceeded':
          return t('account.bandwidth-exceeded');
        case 'account-status-not-active':
          return t('account.status-not-active');
        case 'no-subscription':
          return t('account.no-subscription');
        case 'max-device-reached':
          return t('account.max-devices-reached');
        case 'device-time-desync':
          return t('account.device-time-out-of-sync');
      }

      console.warn('unhandled backend error', error);
      return t('unknown');
    },
    [t],
  );

  return { tE: translateError };
}

export default useI18nError;
