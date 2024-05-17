import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { CmdErrorI18nKey } from '../types';

/**
 * Hook to get the translation function for command errors
 *
 * @returns The translation function
 */
function useCmdErrorI18n() {
  const { t } = useTranslation('errors');

  const getErrorTranslation = useCallback(
    (key: CmdErrorI18nKey) => {
      switch (key) {
        case 'UnknownError':
          return t('unknown');
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
      }
    },
    [t],
  );

  return { eT: getErrorTranslation };
}

export default useCmdErrorI18n;
