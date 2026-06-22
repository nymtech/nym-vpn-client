import { invoke } from '@tauri-apps/api/core';
import { useCallback, useRef, useState } from 'react';

/**
 * Triggers a forced refresh of account state/summary on the daemon.
 *
 * The daemon responds by emitting AccountStateEvent(s) (`syncing` -> final
 * state). `useAccountSummaryOnAccountState` already re-fetches the account
 * summary on those transitions, so this hook only needs to *trigger* the
 * refresh — it does not fetch the summary itself.
 */
export default function useRefreshAccountSummary() {
  const [refreshing, setRefreshing] = useState(false);
  // Guard against overlapping invocations (e.g. mount-refresh + fast button clicks).
  const inFlight = useRef(false);

  const refresh = useCallback(async (force = true) => {
    if (inFlight.current) return;
    inFlight.current = true;
    setRefreshing(true);
    try {
      await invoke<void>('refresh_account_state', { force });
    } finally {
      inFlight.current = false;
      setRefreshing(false);
    }
  }, []);

  return { refresh, refreshing };
}
