import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { dispatch, useAppStore } from '../store';
import { useGwIndependenceWarning } from '../contexts/gatewayIndependence';
import type { TentativeGateways } from '../types/tauri';

// Orchestrates the family/co-location-aware pre-connect flow.
// 1. reset gateway independence to ON (every Connect press)
// 2. ask the daemon for the tentative pair
// 3. on NeedsRelaxedIndependenceCriteria: warn (notifications ON) or
//    silently relax (notifications OFF) before connecting.
function useConnect() {
  const notificationsEnabled = useAppStore(
    (s) => s.gatewayIndependenceNotifications,
  );
  const { requestConfirmation } = useGwIndependenceWarning();

  return useCallback(async () => {
    await invoke('set_gateway_independence', { enabled: true });

    const tentative = await invoke<TentativeGateways>('get_tentative_gateways');

    if (tentative === 'needs-relaxed-independence-criteria') {
      if (notificationsEnabled && !(await requestConfirmation())) return;

      await invoke('set_gateway_independence', { enabled: false });
    }

    dispatch({ type: 'reset-error' });
    dispatch({ type: 'connect' });
    await invoke('connect');
  }, [notificationsEnabled, requestConfirmation]);
}

export default useConnect;
