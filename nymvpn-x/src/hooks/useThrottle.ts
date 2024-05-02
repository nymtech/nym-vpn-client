/* eslint-disable @typescript-eslint/no-explicit-any */
import { useCallback } from 'react';
import * as _ from 'lodash-es';

/**
 * Hook to throttle a function using `_.throttle` as a wrapper
 *
 * @param fn - The function to throttle
 * @param wait - The number of milliseconds to throttle invocations to
 * @param options - Throttle options
 * @returns The throttled function
 */
function useThrottle<Fn extends (...args: any[]) => Promise<any> | any>(
  fn: Fn,
  wait: number,
  options?: _.ThrottleSettings,
) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const t = useCallback(
    _.throttle(async () => fn(), wait, {
      trailing: false,
      ...options,
    }),
    [wait],
  );

  return t;
}

export default useThrottle;
