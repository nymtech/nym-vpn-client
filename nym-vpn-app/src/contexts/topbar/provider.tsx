import { useCallback, useMemo, useState } from 'react';
import { TopBarContext } from './context';

export type TopBarProviderProps = {
  children: React.ReactNode;
};

function TopBarProvider({ children }: TopBarProviderProps) {
  const [customLeftNavHandler, setCustomLeftNavHandlerState] = useState<
    (() => void) | null
  >(null);

  const setCustomLeftNavHandler = useCallback(
    (handler: (() => void) | null) => {
      setCustomLeftNavHandlerState(() => handler);
    },
    [],
  );

  const ctx = useMemo(
    () => ({
      customLeftNavHandler,
      setCustomLeftNavHandler,
    }),
    [customLeftNavHandler, setCustomLeftNavHandler],
  );

  return (
    <TopBarContext.Provider value={ctx}>{children}</TopBarContext.Provider>
  );
}

export default TopBarProvider;
