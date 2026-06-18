import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../store';
import { useGwIndependenceWarning } from '../contexts/gatewayIndependence';

// While connected, the daemon may surface
// Error(NeedsRelaxedIndependenceCriteria) after a settings change.
// Notifications ON  -> show the same warning modal; accept relaxes, decline stays in error.
// Notifications OFF -> relax silently; the library auto-reconnects.
function useGatewayIndependenceWatcher() {
  const tunnelError = useAppStore((s) => s.tunnelError);
  const notificationsEnabled = useAppStore(
    (s) => s.gatewayIndependenceNotifications,
  );
  const { requestConfirmation } = useGwIndependenceWarning();
  // guard against re-triggering while a single error episode is handled
  const handlingRef = useRef(false);

  useEffect(() => {
    if (tunnelError !== 'needs-relaxed-independence-criteria') {
      handlingRef.current = false;
      return;
    }
    if (handlingRef.current) {
      return;
    }
    handlingRef.current = true;

    const handle = async () => {
      if (notificationsEnabled) {
        const confirmed = await requestConfirmation();
        if (!confirmed) {
          return; // remain in error state
        }
      }
      // library triggers the reconnection itself; no further calls needed
      await invoke('set_gateway_independence', { enabled: false });
    };

    handle().catch((e: unknown) => {
      console.error('gateway independence watcher failed', e);
      // Allow a retry on the next state tick; otherwise a failed relax would
      // wedge us in the error state with no way out.
      handlingRef.current = false;
    });
  }, [tunnelError, notificationsEnabled, requestConfirmation]);
}

export default useGatewayIndependenceWatcher;
