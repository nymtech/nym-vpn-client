import { useCallback, useMemo, useState } from 'react';
import { DialogContext, DialogKey, DialogState } from './context';

export type DialogProviderProps = {
  children: React.ReactNode;
};

function DialogProvider({ children }: DialogProviderProps) {
  const [dialogs, setDialogs] = useState<DialogState>({
    'location-info': {
      open: false,
    },
  });

  const isOpen = useCallback(
    (key: DialogKey) => {
      return dialogs[key].open;
    },
    [dialogs],
  );

  const show = useCallback(
    (key: DialogKey) => {
      setDialogs({ ...dialogs, [key]: { open: true } });
    },
    [dialogs],
  );

  const close = useCallback(
    (key: DialogKey) => {
      setDialogs({ ...dialogs, [key]: { open: false } });
    },
    [dialogs],
  );

  const ctx = useMemo(
    () => ({
      dialogs,
      isOpen,
      show,
      close,
    }),
    [close, dialogs, isOpen, show],
  );

  return (
    <DialogContext.Provider value={ctx}>{children}</DialogContext.Provider>
  );
}

export default DialogProvider;
