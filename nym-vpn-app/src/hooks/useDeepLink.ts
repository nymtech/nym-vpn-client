import { useCallback, useEffect, useRef } from 'react';
import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
import { DeeplinkTimeout } from '../errors';

const useDeepLink = () => {
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

  const startListening = useCallback(
    (timeoutMs?: number): Promise<string> => {
      isCleanedUpRef.current = false;
      unlistenRef.current = null;

      const basePromise = new Promise<string>((resolve, reject) => {
        onOpenUrl((urls) => {
          if (isCleanedUpRef.current) return;
          if (!urls || urls.length === 0) return;
          const url = urls[0];

          cleanup();
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

      if (timeoutMs === undefined) {
        return basePromise;
      }

      let timeoutId: ReturnType<typeof setTimeout> | null = null;
      const timeoutPromise = new Promise<never>((_, reject) => {
        timeoutId = setTimeout(() => {
          cleanup();
          if (!isCleanedUpRef.current) {
            isCleanedUpRef.current = true;
          }
          reject(new DeeplinkTimeout());
        }, timeoutMs);
      });

      return Promise.race([basePromise, timeoutPromise]).finally(() => {
        if (timeoutId !== null) {
          clearTimeout(timeoutId);
          timeoutId = null;
        }
      });
    },
    [cleanup],
  );

  return { startListening };
};

export default useDeepLink;
