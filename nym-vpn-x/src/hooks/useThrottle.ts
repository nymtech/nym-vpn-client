import { DependencyList, useCallback } from 'react';
import * as _ from 'lodash-es';

/**
 * Hook to throttle a function using `_.throttle` as a wrapper
 *
 * @param fn - The function to throttle
 * @param wait - The number of milliseconds to throttle invocations to
 * @param deps - The dependencies to watch for callback reset (passed to `useCallback`)
 * @param options - Throttle options
 * @returns The throttled function
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function useThrottle<Fn extends (...args: any[]) => Promise<void> | void>(
  fn: Fn,
  wait: number,
  deps: DependencyList = [],
  options?: _.ThrottleSettings,
) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const t = useCallback(
    _.throttle(async () => fn(), wait, {
      trailing: false,
      ...options,
    }),
    [wait, ...deps],
  );

  return t;
}

export default useThrottle;
