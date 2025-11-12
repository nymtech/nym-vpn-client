import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorKey, TunnelError, isInternalError } from '../types';

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
// TODO split this function to avoid key conflicts between BackendError and TunnelError
function useI18nError() {
  const { t } = useTranslation('errors');

  const translateError: Terror = useCallback(
    (error: ErrorKey | TunnelError) => {
      if (isInternalError(error)) {
        return fmtErr(t('tunnel.internal'), error.internal);
      }
      // tunnel state errors
      switch (error) {
        case 'tun-device':
          return t('tunnel.tun-device');
        case 'tunnel-provider':
          return t('tunnel.tunnel-provider');
        case 'inactive-account':
          return t('tunnel.inactive-account');
        case 'device-logged-out':
          return t('tunnel.device-logged-out');
        case 'set-firewall-policy':
          return t('tunnel.firewall');
        case 'set-routing':
          return t('tunnel.routing');
        case 'set-dns':
          return t('tunnel.dns');
        case 'same-entry-and-exit-gw':
          return t('tunnel.same-entry-exit-gw');
        case 'invalid-entry-gw-country':
          return t('tunnel.invalid-entry-gw-country');
        case 'invalid-exit-gw-country':
          return t('tunnel.invalid-exit-gw-country');
        case 'invalid-entry-gw-id':
          return t('tunnel.invalid-entry-gw-id');
        case 'invalid-exit-gw-id':
          return t('tunnel.invalid-exit-gw-id');
        case 'max-devices-reached':
          return t('tunnel.max-devices-reached');
        case 'bandwidth-exceeded':
          return t('tunnel.bandwidth-exceeded');
        case 'inactive-subscription':
          return t('tunnel.subscription-expired');
        case 'device-time-out-of-sync':
          return t('tunnel.device-time-out-of-sync');
        case 'ipv6-unavailable':
          return t('tunnel.ipv6-unavailable');
        case 'credential-wasted-on-entry-gateway':
          return t('tunnel.credential-wasted-entry-gw');
        case 'credential-wasted-on-exit-gateway':
          return t('tunnel.credential-wasted-exit-gw');
        case 'performant-entry-gw-unavailable':
          return t('tunnel.performant-entry-gw-unavailable');
        case 'performant-exit-gw-unavailable':
          return t('tunnel.performant-exit-gw-unavailable');
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
