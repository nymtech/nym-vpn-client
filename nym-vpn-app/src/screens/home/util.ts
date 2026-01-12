import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { allCountries } from 'country-region-data';
import { useInAppNotify, useMainState } from '../../contexts';
import { TunnelData, isWireguardData } from '../../types';

// find country code (lowercase) for a given region name
export function regionToCountryCode(region: string): string | null {
  for (const data of allCountries) {
    const res = data[2].some(
      (r) => r[0].toLowerCase() === region.toLowerCase(),
    );
    if (res) {
      return data[1].toLowerCase();
    }
  }
  console.warn(`country not found for region [${region}]`);
  return null;
}

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

export function isBridgeMode(data?: TunnelData | null) {
  if (!data) {
    return false;
  }
  return isWireguardData(data) && data.entryBridgeAddr;
}
