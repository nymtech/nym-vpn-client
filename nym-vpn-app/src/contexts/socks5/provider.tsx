import { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { HttpRpcSettings, SelectedNode, Socks5Settings, Socks5Status } from '../../types';
import { Socks5Context } from './context';

export type Socks5ProviderProps = {
  children: React.ReactNode;
};

// get socks5 status every 5 seconds
const POLL_INTERVAL = 5000; // 5 seconds
// prevent multiple initialization of the interval due to react strict mode
let initialized = false;

export function Socks5Provider({ children }: Socks5ProviderProps) {
  const [status, setStatus] = useState<Socks5Status | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // update status state
  const refresh = useCallback(async () => {
    try {
      const result = await invoke<Socks5Status>('get_socks5_status');
      setStatus(result);
    } catch {
      // silently ignore - status polling may fail intermittently
    }
  }, []);

  // enable socks5
  const enable = useCallback(
    async (
      socks5Settings: Socks5Settings,
      httpRpcSettings: HttpRpcSettings,
      exit: SelectedNode,
    ) => {
      // Prevent concurrent enable calls
      if (isLoading) {
        console.warn(
          'SOCKS5 enable already in progress, ignoring duplicate call',
        );
        return;
      }

      setIsLoading(true);
      try {
        await invoke('enable_socks5', {
          socks5Settings,
          httpRpcSettings,
          exit,
        });
        await refresh();
      } catch (error) {
        console.error('Failed to enable SOCKS5 proxy:', error);
        // TODO: remove this throw and use proper UI notification
        throw error;
      } finally {
        setIsLoading(false);
      }
    },
    [refresh, isLoading],
  );

  // disable socks5
  const disable = useCallback(async () => {
    // Prevent concurrent disable calls
    if (isLoading) {
      console.warn(
        'SOCKS5 disable already in progress, ignoring duplicate call',
      );
      return;
    }

    setIsLoading(true);
    try {
      await invoke('disable_socks5');
      await refresh();
    } catch (error) {
      console.error('Failed to disable SOCKS5 proxy:', error);
      // TODO: remove this throw and use proper UI notification
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [refresh, isLoading]);

  // initial load and periodic polling
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    refresh();
    const interval = setInterval(refresh, POLL_INTERVAL);
    return () => {
      clearInterval(interval);
      initialized = false;
    };
  }, [refresh]);

  const ctx = useMemo(
    () => ({ status, isLoading, enable, disable, refresh }),
    [status, isLoading, enable, disable, refresh],
  );

  return (
    <Socks5Context.Provider value={ctx}>{children}</Socks5Context.Provider>
  );
}
