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

  const getErrorTranslation = useCallback(
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
        case 'UnknownError':
          return t('unknown');

        default:
          console.warn(`Unknown error key: ${key}`);
          return t('unknown');
      }
    },
    [t],
  );

  return { eT: getErrorTranslation };
}

export default useI18nError;
