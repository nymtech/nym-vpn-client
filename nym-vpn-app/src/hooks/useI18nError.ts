import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorKey, TunnelError } from '../types';

/**
 * Hook to get the translation function for backend errors
 *
 * @returns The translation function
 */
function useI18nError() {
  const { t } = useTranslation('errors');

  const translateError = useCallback(
    (key: ErrorKey | TunnelError) => {
      switch (key) {
        // tunnel state errors
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
        // mixnet event errors
        case 'EntryGwDown':
          return t('entry-gateway-down');
        case 'ExitGwDownIpv4':
          return t('exit-gateway-down.ipv4');
        case 'ExitGwDownIpv6':
          return t('exit-gateway-down.ipv6');
        case 'ExitGwRoutingErrorIpv4':
          return t('exit-gateway-routing.ipv4');
        case 'ExitGwRoutingErrorIpv6':
          return t('exit-gateway-routing.ipv6');
        case 'NoBandwidth':
          return t('no-bandwidth');
        // general errors
        case 'InternalError':
          return t('internal');
        case 'NotConnectedToDaemon':
          return t('daemon.not-connected');
        case 'GrpcError':
          return t('grpc');
        case 'AccountInvalidMnemonic':
          return t('account.invalid-recovery-phrase');
        case 'AccountStorage':
          return t('account.storage');
        case 'AccountIsConnected':
          return t('account.is-connected');
        case 'ConnectGeneral':
          return t('connection-general');
        case 'ConnectNoAccountStored':
          return t('account.not-stored');
        case 'ConnectNoDeviceStored':
          return t('account.no-device-stored');
        case 'ConnectUpdateAccount':
          return t('account.update');
        case 'ConnectUpdateDevice':
          return t('account.update-device');
        case 'ConnectRegisterDevice':
          return t('account.register-device');
        case 'ConnectRequestZkNym':
          return t('zknym.request-failed');
        case 'GetMixnetEntryCountriesQuery':
          return t('countries-request.entry');
        case 'GetMixnetExitCountriesQuery':
          return t('countries-request.exit');
        case 'GetWgCountriesQuery':
          return t('countries-request.fast-mode');
        case 'InvalidNetworkName':
          return t('daemon.invalid-network');
        case 'UnknownError':
          return t('unknown');
        case 'MaxRegisteredDevices':
          return t('account.maximum-registered-devices');

        default:
          console.warn(`Unknown error key: ${key}`);
          return t('unknown');
      }
    },
    [t],
  );

  return { tE: translateError };
}

export default useI18nError;
