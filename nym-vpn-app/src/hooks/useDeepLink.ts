import { useCallback, useEffect, useRef } from "react";
import { onOpenUrl } from "@tauri-apps/plugin-deep-link";

const PRIVY_DEEPLINK_URL = 'nymvpn://signin/';

export const useDeepLink = () => {
  const unlistenRef = useRef<(() => void) | null>(null);
  const isCleanedUpRef = useRef(false);

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
        if (!urls || urls.length === 0 || urls[0] !== PRIVY_DEEPLINK_URL) return;
        const url = urls[0];

        cleanup();
        resolve(url);
      }).then((unlistenFn) => {
        if (isCleanedUpRef.current) {
          unlistenFn();
          return;
        }
        unlistenRef.current = unlistenFn;
      }).catch((error: unknown) => {
        if (!isCleanedUpRef.current) {
          cleanup();
          reject(error instanceof Error ? error : new Error(String(error)));
        }
      });
    });
  }, [cleanup]);

  return { startListening };
}
