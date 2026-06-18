import { useCallback, useMemo, useRef, useState } from 'react';
import { GwIndependenceWarningContext } from './context';

export type GwIndependenceWarningProviderProps = {
  children: React.ReactNode;
};

function GwIndependenceWarningProvider({
  children,
}: GwIndependenceWarningProviderProps) {
  const [isOpen, setIsOpen] = useState(false);
  const resolverRef = useRef<((value: boolean) => void) | null>(null);

  const settle = useCallback((value: boolean) => {
    setIsOpen(false);
    resolverRef.current?.(value);
    resolverRef.current = null;
  }, []);

  const requestConfirmation = useCallback(
    () =>
      new Promise<boolean>((resolve) => {
        // If a confirmation is already pending, settle it false before
        // replacing its resolver so the prior promise never hangs forever.
        resolverRef.current?.(false);
        resolverRef.current = resolve;
        setIsOpen(true);
      }),
    [],
  );

  const accept = useCallback(() => settle(true), [settle]);
  const cancel = useCallback(() => settle(false), [settle]);

  const ctx = useMemo(
    () => ({ isOpen, requestConfirmation, accept, cancel }),
    [isOpen, requestConfirmation, accept, cancel],
  );

  return (
    <GwIndependenceWarningContext.Provider value={ctx}>
      {children}
    </GwIndependenceWarningContext.Provider>
  );
}

export default GwIndependenceWarningProvider;
