import { useState } from 'react';
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

  const isOpen = (key: DialogKey) => {
    return dialogs[key].open;
  };

  const show = (key: DialogKey) => {
    setDialogs({ ...dialogs, [key]: { open: true } });
  };

  const close = (key: DialogKey) => {
    setDialogs({ ...dialogs, [key]: { open: false } });
  };

  return (
    <DialogContext.Provider
      value={{
        dialogs,
        isOpen,
        show,
        close,
      }}
    >
      {children}
    </DialogContext.Provider>
  );
}

export default DialogProvider;
