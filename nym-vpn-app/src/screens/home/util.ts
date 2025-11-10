import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useInAppNotify, useMainState } from '../../contexts';
import { StateDispatch } from '../../types';
import { kvSet } from '../../kvStore';

export function useActionToast(action: 'node-select' | 'mode-select') {
  const { state } = useMainState();
  const { t } = useTranslation('home');
  const { push } = useInAppNotify();

  const toast = useCallback(
    (throttle = 2) => {
      let text = null;
      switch (state) {
        case 'connected':
          text = t('snackbar-disabled-message.connected');
          break;
        case 'connecting':
          text = t('snackbar-disabled-message.connecting');
          break;
        case 'disconnecting':
          text = t('snackbar-disabled-message.disconnecting');
          break;
        case 'offline':
        case 'offline-auto-reconnect':
          text = t('snackbar-disabled-message.offline');
          break;
        case 'error':
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

export const setStreamOptimizedLabelSeen = (dispatch: StateDispatch) => {
  dispatch({ type: 'set-streaming-optimized-label-seen', seen: true });
  kvSet('streaming-optimized-label-seen', true);
};
