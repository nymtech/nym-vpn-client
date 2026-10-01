import { useCallback, useMemo, useState } from 'react';
import { TopBarContext } from './context';

export type TopBarProviderProps = {
  children: React.ReactNode;
};

function TopBarProvider({ children }: TopBarProviderProps) {
  const [customLeftNavHandler, setCustomLeftNavHandler] = useState<
    (() => void) | null
  >(null);

  const updateCustomLeftNavHandler = useCallback(
    (handler: (() => void) | null) => {
      setCustomLeftNavHandler(() => handler);
    },
    [],
  );

  const ctx = useMemo(
    () => ({
      customLeftNavHandler,
      setCustomLeftNavHandler: updateCustomLeftNavHandler,
    }),
    [customLeftNavHandler, updateCustomLeftNavHandler],
  );

  return <TopBarContext value={ctx}>{children}</TopBarContext>;
}

export default TopBarProvider;
