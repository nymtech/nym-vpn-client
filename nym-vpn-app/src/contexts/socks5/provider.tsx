import { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  SelectedNode,
  Socks5Status,
  Socks5Settings,
  HttpRpcSettings,
} from '../../types';
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
    } catch (error) {
      console.error('Failed to get SOCKS5 status:', error);
    }
  }, []);

  // enable socks5
  const enable = useCallback(
    async (
      socks5Settings: Socks5Settings,
      httpRpcSettings: HttpRpcSettings,
      exit: SelectedNode,
    ) => {
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
        throw error;
      } finally {
        setIsLoading(false);
      }
    },
    [refresh],
  );

  // disable socks5
  const disable = useCallback(async () => {
    setIsLoading(true);
    try {
      await invoke('disable_socks5');
      await refresh();
    } catch (error) {
      console.error('Failed to disable SOCKS5 proxy:', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [refresh]);

  // initial load and periodic polling
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    refresh();
    const interval = setInterval(refresh, POLL_INTERVAL);
    return () => clearInterval(interval);
  }, [refresh]);

  const ctx = useMemo(
    () => ({ status, isLoading, enable, disable, refresh }),
    [status, isLoading, enable, disable, refresh],
  );

  return (
    <Socks5Context.Provider value={ctx}>{children}</Socks5Context.Provider>
  );
}
