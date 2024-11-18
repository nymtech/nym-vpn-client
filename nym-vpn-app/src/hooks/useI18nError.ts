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

  const translateError = useCallback(
    (key: ErrorKey) => {
      switch (key) {
        case 'InternalError':
          return t('internal');
        case 'NotConnectedToDaemon':
          return t('daemon.not-connected');
        case 'GrpcError':
          return t('grpc');
        case 'CStateNoValidCredential':
          return t('connection.no-valid-credential');
        case 'CStateTimeout':
          return t('connection.timeout');
        case 'CStateMixnetTimeout':
          return t('connection.mixnet.timeout');
        case 'CStateMixnetStoragePaths':
          return t('connection.mixnet.storage-path');
        case 'CStateMixnetDefaultStorage':
          return t('connection.mixnet.default-storage');
        case 'CStateMixnetBuildClient':
          return t('connection.mixnet.build-client');
        case 'CStateMixnetConnect':
          return t('connection.mixnet.connect');
        case 'CStateMixnetEntryGateway':
          return t('connection.gateway-lookup.entry');
        case 'CStateIprFailedToConnect':
          return t('connection.ipr-connect');
        case 'CStateGwDir':
          return t('connection.gateway-lookup.generic');
        case 'CStateGwDirLookupGateways':
          return t('connection.gateway-lookup.generic');
        case 'CStateGwDirLookupGatewayId':
          return t('connection.gateway-lookup.id');
        case 'CStateGwDirLookupRouterAddr':
          return t('connection.gateway-lookup.ipr');
        case 'CStateGwDirLookupIp':
          return t('connection.gateway-lookup.ip');
        case 'CStateGwDirEntry':
          return t('connection.gateway-lookup.entry');
        case 'CStateGwDirEntryId':
          return t('connection.gateway-lookup.entry-id');
        case 'CStateGwDirEntryLocation':
          return t('connection.gateway-lookup.entry-location');
        case 'CStateGwDirExit':
          return t('connection.gateway-lookup.exit');
        case 'CStateGwDirExitLocation':
          return t('connection.gateway-lookup.exit-location');
        case 'CStateGwDirSameEntryAndExitGw':
          return t('connection.bad-country-combination');
        case 'CStateOutOfBandwidth':
          return t('out-of-bandwidth');
        case 'CStateOutOfBandwidthSettingUpTunnel':
          return t('connection.bandwidth.tunnel-up');
        case 'CStateFindDefaultInterface':
          return t('connection.interface.find-default');
        case 'CStateBringInterfaceUp':
          return t('connection.interface.wg-bring-up');
        case 'CStateFirewallInit':
          return t('connection.firewall.init');
        case 'CStateFirewallResetPolicy':
          return t('connection.firewall.reset-policy');
        case 'CStateDnsInit':
          return t('connection.dns.init');
        case 'CStateDnsSet':
          return t('connection.dns.set');
        case 'CSDaemonInternal':
          return t('daemon.internal');
        case 'CSUnhandledExit':
          return t('connection.unhandled-exit');
        case 'CSAuthenticatorFailedToConnect':
          return t('connection.authenticator.connect');
        case 'CSAuthenticatorConnectTimeout':
          return t('connection.authenticator.timeout');
        case 'CSAuthenticatorInvalidResponse':
          return t('connection.authenticator.invalid-response');
        case 'CSAuthenticatorRegistrationDataVerification':
          return t('connection.authenticator.registration-data');
        case 'CSAuthenticatorEntryGatewaySocketAddr':
          return t('connection.authenticator.entry-gw-socket-addr');
        case 'CSAuthenticatorEntryGatewayIpv4':
          return t('connection.authenticator.entry-gw-ipv4');
        case 'CSAuthenticatorWrongVersion':
          return t('connection.authenticator.wrong-version');
        case 'CSAuthenticatorMalformedReply':
          return t('connection.authenticator.malformed-reply');
        case 'CSAuthenticatorAddressNotFound':
          return t('connection.authenticator.address-not-found');
        case 'CSAuthenticatorAuthenticationNotPossible':
          return t('connection.authenticator.auth-not-possible');
        case 'CSAddIpv6Route':
          return t('connection.add-ipv6-route');
        case 'CSTun':
          return t('connection.tun-device');
        case 'CSRouting':
          return t('connection.routing');
        case 'CSWireguardConfig':
          return t('connection.wireguard.config');
        case 'CSMixnetConnectionMonitor':
          return t('connection.mixnet.monitor');
        case 'AccountInvalidMnemonic':
          return t('account.invalid-recovery-phrase');
        case 'AccountStorage':
          return t('account.storage');
        case 'ConnectGeneral':
          return t('connection.general');
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
        case 'EntryGatewayNotRouting':
          return t('entry-node-routing');
        case 'ExitRouterPingIpv4':
          return t('exit-node.ping', { protocol: 'IPv4' });
        case 'ExitRouterNotRoutingIpv4':
          return t('exit-node.routing', { protocol: 'IPv4' });
        case 'ExitRouterPingIpv6':
          return t('exit-node.ping', { protocol: 'IPv6' });
        case 'ExitRouterNotRoutingIpv6':
          return t('exit-node.routing', { protocol: 'IPv6' });
        case 'UserNoBandwidth':
          return t('out-of-bandwidth');
        case 'WgTunnelError':
          return t('connection.wireguard.tunnel');
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
