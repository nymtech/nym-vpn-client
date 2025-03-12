import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { Notification } from './type';
import { InAppNotificationContext } from './context';

type Timeout = ReturnType<typeof setTimeout>;

export type NotificationProviderProps = {
  children: React.ReactNode;
};

// ⚠ This duration must be greater than the duration of the
// toast animation (defined in Toast.tsx)
const transitionDuration = 300; // ms

function InAppNotificationProvider({ children }: NotificationProviderProps) {
  const [stack, setStack] = useState<Notification[]>([]);
  // the current notification being displayed
  const [current, setCurrent] = useState<Notification | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);

  const throttled = useRef<Record<string, number>>({});
  const transitionRef = useRef<Timeout | null>(null);

  const push = useCallback((notification: Notification) => {
    // using an updater function to add notifications into the stack
    // in a serial fashion
    // without this, it will not work!
    setStack((prev) => {
      // check for duplicates
      if (prev.some((n) => n.message === notification.message)) {
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
  }, []);

  const clear = useCallback(() => {
    setStack([]);
    throttled.current = {};
    setIsTransitioning(false);
    setCurrent(null);
    clearTimeout(transitionRef.current as Timeout | undefined);
  }, []);

  useEffect(() => {
    if (current || isTransitioning) {
      return;
    }
    const notification = stack[0];
    if (notification) {
      setCurrent(notification);
      // set the stack state with the previous stack but first element removed
      setStack([...stack.slice(1)]);
    }
  }, [current, stack, isTransitioning]);

  // ⚠ keep this function un-memoized to prevent transition glitch
  const onClose = () => {
    setIsTransitioning(true);
    setCurrent(null);
    transitionRef.current = setTimeout(() => {
      setIsTransitioning(false);
    }, transitionDuration);
  };

  const ctx = useMemo(
    () => ({ clear, current, push, onClose }),
    [clear, current, push],
  );

  return (
    <InAppNotificationContext.Provider value={ctx}>
      {children}
    </InAppNotificationContext.Provider>
  );
}

export default InAppNotificationProvider;
