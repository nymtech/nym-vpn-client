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
    // 1. reset to constrained on every Connect press
    await invoke('set_gateway_independence', { enabled: true });

    // 2. tentative selection
    const tentative = await invoke<TentativeGateways>('get_tentative_gateways');

    // 3. handle the "needs relaxing" case
    if (tentative === 'needs-relaxed-independence-criteria') {
      if (notificationsEnabled) {
        const confirmed = await requestConfirmation();
        if (!confirmed) {
          return; // user declined — stay disconnected
        }
      }
      // Relaxed only for this attempt. If `connect` below throws, we
      // intentionally do NOT roll this back — step 1 re-asserts `true` on the
      // next Connect press, so the constrained default always returns.
      await invoke('set_gateway_independence', { enabled: false });
    }

    // 'selected' and 'no-gateways-available' fall through to connect as before
    dispatch({ type: 'reset-error' });
    dispatch({ type: 'connect' });
    await invoke('connect');
  }, [notificationsEnabled, requestConfirmation]);
}

export default useConnect;
