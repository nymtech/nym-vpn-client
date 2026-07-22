import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { ConnectingProgress, ProgressMsg } from '../types';

/**
 * Hook to translate connecting progress state messages
 *
 * @returns The translation function
 */
function useI18nProgressMsg() {
  const { t } = useTranslation('backend-messages');

  const translate = useCallback(
    (state: ConnectingProgress | ProgressMsg) => {
      switch (state) {
        case 'canceling':
          return t('connection-progress.canceling');
        case 'resolving-api-addresses':
          return t('connection-progress.resolving-api-addresses');
        case 'awaiting-account-readiness':
          return t('connection-progress.awaiting-account-readiness');
        case 'awaiting-credentials-availability':
          return t('connection-progress.awaiting-credentials-availability');
        case 'refreshing-gateways':
          return t('connection-progress.refreshing-gateways');
        case 'selecting-gateways':
          return t('connection-progress.selecting-gateways');
        case 'registering-with-gateways':
          return t('connection-progress.registering-with-gateways');
        case 'connecting-tunnel':
          return t('connection-progress.connecting-tunnel');
      }
    },
    [t],
  );

  return { t: translate };
}

export default useI18nProgressMsg;
