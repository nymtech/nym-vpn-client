import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useInAppNotify, useMainState } from '../../contexts';

export function useActionToast(action: 'node-select' | 'mode-select') {
  const { state } = useMainState();
  const { t } = useTranslation('home');
  const { push } = useInAppNotify();

  const toast = useCallback(
    (throttle = 2) => {
      let text = null;
      switch (state) {
        case 'Connected':
          text = t('snackbar-disabled-message.connected');
          break;
        case 'Connecting':
          text = t('snackbar-disabled-message.connecting');
          break;
        case 'Disconnecting':
          text = t('snackbar-disabled-message.disconnecting');
          break;
        case 'Offline':
        case 'OfflineAutoReconnect':
          text = t('snackbar-disabled-message.offline');
          break;
        case 'Error':
          text = t('snackbar-disabled-message.error');
          break;
      }
      if (text) {
        push({
          id: `disabled-${action}-${state}`,
          message: text,
          throttle,
          clickAway: true,
        });
      }
    },
    [action, push, state, t],
  );

  return toast;
}
