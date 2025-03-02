import { useEffect, useRef } from 'react';

type ClickAwayProps = {
  on: () => void;
  disabled?: boolean;
};

/* Hook to register click-away listener */
function useClickAway<T extends HTMLElement>({ on, disabled }: ClickAwayProps) {
  const ref = useRef<T>(null);

  useEffect(() => {
    if (disabled || !ref.current) {
      return;
    }
    const handleClickOutside = (event: MouseEvent) => {
      if (!ref.current?.contains(event.target as Node)) {
        on();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [on, disabled]);

  return ref;
}

export default useClickAway;
