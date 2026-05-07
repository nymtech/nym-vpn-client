import { createContext, useCallback, useContext, useRef } from 'react';

type ExitFn = () => Promise<void>;

const CardAnimationContext = createContext<{
  registerExit: (fn: ExitFn | null) => void;
  triggerExit: () => Promise<void>;
}>({
  registerExit: () => {
    /* SCARECROW */
  },
  triggerExit: () => Promise.resolve(),
});

export function CardAnimationProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const exitFnRef = useRef<ExitFn | null>(null);

  const registerExit = useCallback((fn: ExitFn | null) => {
    exitFnRef.current = fn;
  }, []);

  const triggerExit = useCallback(async () => {
    await exitFnRef.current?.();
  }, []);

  return (
    <CardAnimationContext.Provider value={{ registerExit, triggerExit }}>
      {children}
    </CardAnimationContext.Provider>
  );
}

export function useCardAnimation() {
  return useContext(CardAnimationContext);
}
