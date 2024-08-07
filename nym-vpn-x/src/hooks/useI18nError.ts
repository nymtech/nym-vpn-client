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
        case 'ConnectionTimeout':
          return t('connection.timeout');
        case 'ConnectionGatewayLookup':
          return t('connection.gateway-lookup');
        case 'ConnectionNoValidCredential':
          return t('connection.no-valid-credential');
        case 'ConnectionSameEntryAndExitGw':
          return t('connection.bad-country-combination');
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
        case 'GetEntryCountriesRequest':
          return t('countries-request.entry');
        case 'GetExitCountriesRequest':
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
