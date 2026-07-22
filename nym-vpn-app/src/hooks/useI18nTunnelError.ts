import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { TunnelError, isTunnelInternalError } from '../types';

/**
 * Hook to get the translation function for tunnel errors
 *
 * @returns The translation function
 */
function useI18nTunnelError() {
  const { t } = useTranslation('errors');

  const translate = useCallback(
    (error: TunnelError) => {
      if (isTunnelInternalError(error)) {
        return `${t('tunnel.internal')} - ${error.internal}`;
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
        case 'needs-relaxed-independence-criteria':
          return t('tunnel.needs-relaxed-independence-criteria');
        case 'credential-fetching-failed':
          return t('tunnel.credential-fetching-failed');
        case 'no-credential-available':
          return t('tunnel.no-credential-available');
      }

      console.warn('unhandled tunnel error', error);
      return t('unknown');
    },
    [t],
  );

  return { tTE: translate };
}

export default useI18nTunnelError;
