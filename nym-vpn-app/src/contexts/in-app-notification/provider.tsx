import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { Notification } from './type';
import { InAppNotificationContext } from './context';

type Timeout = ReturnType<typeof setTimeout>;

export type NotificationProviderProps = {
  children: React.ReactNode;
};

// ⚠ This duration must be greater than the duration of the
// snackbar animation (defined in Snackbar.tsx)
const transitionDuration = 300; // ms

function InAppNotificationProvider({ children }: NotificationProviderProps) {
  const [stack, setStack] = useState<Notification[]>([]);
  const [current, setCurrent] = useState<Notification | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);

  const throttled = useRef<Record<string, number>>({});
  const transitionRef = useRef<Timeout | null>(null);

  const checkDuplicate = useCallback(
    (stack: Notification[], toBeChecked: Notification) => {
      return stack.some((n) => n.text === toBeChecked.text);
    },
    [],
  );

  const push = useCallback(
    (notification: Notification) => {
      setStack((prev) => {
        if (checkDuplicate(prev, notification)) {
          return prev;
        }
        const { id, throttle } = notification;
        if (id && throttle && throttle > 0) {
          const expiry = throttled.current[id];
          if (expiry && Date.now() < expiry) {
            return prev;
          }
          throttled.current[id] = Date.now() + throttle * 1000;
        }
        return [...prev, notification];
      });
    },
    [checkDuplicate],
  );

  const shift = useCallback(() => {
    if (stack.length === 0) {
      return null;
    }
    const first = stack[0];
    setStack([...stack.slice(1)]);
    return first;
  }, [stack]);

  const clear = useCallback(() => {
    setStack([]);
    setIsTransitioning(false);
    setCurrent(null);
    throttled.current = {};
    clearTimeout(transitionRef.current as Timeout | undefined);
  }, []);

  useEffect(() => {
    if (current || isTransitioning) {
      return;
    }
    const notification = shift();
    if (notification) {
      setCurrent(notification);
    }
  }, [shift, current, stack.length, isTransitioning]);

  const next = useCallback(() => {
    setIsTransitioning(true);
    setCurrent(null);
    transitionRef.current = setTimeout(() => {
      setIsTransitioning(false);
    }, transitionDuration);
  }, []);

  const ctx = useMemo(
    () => ({ current, next, push, clear }),
    [clear, current, next, push],
  );

  return (
    <InAppNotificationContext.Provider value={ctx}>
      {children}
    </InAppNotificationContext.Provider>
  );
}

export default InAppNotificationProvider;
