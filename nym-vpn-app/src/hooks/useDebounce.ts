import { useCallback, useEffect, useRef } from 'react';

type DebouncedFn<T extends unknown[]> = ((...args: T) => void) & {
  cancel: () => void;
};

/**
 * Returns a debounced version of `callback` that delays invocation until
 * `delay` ms have elapsed since the last call. The returned function exposes
 * a `cancel` method to drop any pending invocation, and pending timers are
 * cleared automatically on unmount.
 */
function useDebounce<T extends unknown[]>(
  callback: (...args: T) => void,
  delay = 250,
): DebouncedFn<T> {
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const callbackRef = useRef(callback);

  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  const cancel = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
  }, []);

  useEffect(() => cancel, [cancel]);

  const debounced = useCallback(
    (...args: T) => {
      cancel();
      timer.current = setTimeout(() => {
        callbackRef.current(...args);
      }, delay);
    },
    [cancel, delay],
  );

  return Object.assign(debounced, { cancel });
}

export default useDebounce;
