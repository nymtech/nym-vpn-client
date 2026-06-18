import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { dispatch, useAppStore } from '../store';

function useServerFamilyReminders() {
  const enabled = useAppStore((s) => s.gatewayIndependenceNotifications);

  const toggle = useCallback(async () => {
    const next = !enabled;
    // optimistic update; daemon is the source of truth
    dispatch({ type: 'set-gateway-independence-notifications', enabled: next });
    try {
      await invoke('set_gateway_independence_notifications', { enabled: next });
    } catch (e) {
      console.error('failed to set gateway independence notifications', e);
      // revert on failure
      dispatch({
        type: 'set-gateway-independence-notifications',
        enabled,
      });
    }
  }, [enabled]);

  return { enabled, toggle };
}

export default useServerFamilyReminders;
