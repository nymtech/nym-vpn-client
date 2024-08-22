import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { BkdErrorKey } from '../types';

/**
 * Hook to get the translation function for backend errors
 *
 * @returns The translation function
 */
function useI18nError() {
  const { t } = useTranslation('errors');

  const translateError = useCallback(
    (key: BkdErrorKey) => {
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
        case 'CredentialInvalid':
          return t('credential.invalid');
        case 'CredentialVpnRunning':
          return t('credential.vpn-running');
        case 'CredentialAlreadyImported':
          return t('credential.no-duplicate');
        case 'CredentialStorageError':
          return t('credential.storage');
        case 'CredentialDeserializationFailure':
          return t('credential.deserialize');
        case 'CredentialExpired':
          return t('credential.expired');
        case 'EntryGatewayNotRouting':
          return t('entry-node-routing');
        case 'ExitRouterPingIpv4':
          return t('exit-node.ping', { protocol: 'IPv4' });
        case 'ExitRouterNotRoutingIpv4':
          return t('exit-node.routing', { protocol: 'IPv4' });
        case 'UserNoBandwidth':
          return t('out-of-bandwidth');
        case 'GetEntryCountriesQuery':
          return t('countries-request.entry');
        case 'GetExitCountriesQuery':
          return t('countries-request.exit');
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
