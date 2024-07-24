import { useCallback, useEffect, useRef, useState } from 'react';
import type { Notification } from './type';
import { NotificationContext } from './context';

type Timeout = ReturnType<typeof setTimeout>;

export type NotificationProviderProps = {
  children: React.ReactNode;
};

// ⚠ This duration must be greater than the duration of the
// snackbar animation (defined in Snackbar.tsx)
const transitionDuration = 300; // ms

function NotificationProvider({ children }: NotificationProviderProps) {
  const [stack, setStack] = useState<Notification[]>([]);
  const [current, setCurrent] = useState<Notification | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);

  const transitionRef = useRef<Timeout | null>(null);

  const checkDuplicate = useCallback(
    (stack: Notification[], toBeChecked: Notification) => {
      return stack.some((n) => n.text === toBeChecked.text);
    },
    [],
  );

  const push = useCallback(
    (notification: Notification | Notification[]) => {
      if (Array.isArray(notification)) {
        setStack((prev) => {
          const isDuplicate = notification.some((n) => checkDuplicate(prev, n));
          if (isDuplicate) {
            return prev;
          }
          return [...prev, ...notification];
        });
      } else {
        setStack((prev) => {
          if (checkDuplicate(prev, notification)) {
            return prev;
          }
          return [...prev, notification];
        });
      }
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

  const clear = () => {
    setStack([]);
    setIsTransitioning(false);
    setCurrent(null);
    clearTimeout(transitionRef.current as Timeout | undefined);
  };

  useEffect(() => {
    if (current || isTransitioning) {
      return;
    }
    const notification = shift();
    if (notification) {
      setCurrent(notification);
    }
  }, [shift, current, stack.length, isTransitioning]);

  const next = () => {
    setIsTransitioning(true);
    setCurrent(null);
    transitionRef.current = setTimeout(() => {
      setIsTransitioning(false);
    }, transitionDuration);
  };

  return (
    <NotificationContext.Provider
      value={{
        stack,
        current,
        next,
        push,
        clear,
      }}
    >
      {children}
    </NotificationContext.Provider>
  );
}

export default NotificationProvider;
