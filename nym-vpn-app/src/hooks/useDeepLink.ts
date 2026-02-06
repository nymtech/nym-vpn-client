import { useCallback, useEffect, useRef } from 'react';
import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
import { invoke } from '@tauri-apps/api/core';
import { StateDispatch } from '../types/app-state';
import { useMainDispatch } from '../contexts';
import { TAccountMode } from '../types/tauri';

const PRIVY_DEEPLINK_URL = 'nymvpn://auth/privy/privateKey';

const useDeepLink = () => {
  const dispatch = useMainDispatch() as StateDispatch;

  const unlistenRef = useRef<(() => void) | null>(null);
  const isCleanedUpRef = useRef(false);

  const refreshAccountMode = useCallback(async () => {
    const mode = await invoke<TAccountMode>('get_account_mode');
    dispatch({ type: 'set-account-mode', mode });
  }, [dispatch]);

  const cleanup = useCallback(() => {
    if (unlistenRef.current && !isCleanedUpRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
      isCleanedUpRef.current = true;
    }
  }, []);

  useEffect(() => {
    return cleanup;
  }, [cleanup]);

  const startListening = useCallback((): Promise<string> => {
    isCleanedUpRef.current = false;
    unlistenRef.current = null;

    return new Promise<string>((resolve, reject) => {
      onOpenUrl((urls) => {
        if (isCleanedUpRef.current) return;
        if (
          !urls ||
          urls.length === 0 ||
          !urls[0].startsWith(PRIVY_DEEPLINK_URL)
        )
          return;
        const url = urls[0];

        cleanup();
        refreshAccountMode();
        resolve(url);
      })
        .then((unlistenFn) => {
          if (isCleanedUpRef.current) {
            unlistenFn();
            return;
          }
          unlistenRef.current = unlistenFn;
        })
        .catch((error: unknown) => {
          if (!isCleanedUpRef.current) {
            cleanup();
            reject(error instanceof Error ? error : new Error(String(error)));
          }
        });
    });
  }, [cleanup, refreshAccountMode]);

  return { startListening };
};

export default useDeepLink;
