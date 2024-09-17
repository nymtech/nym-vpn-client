import { createContext, useContext } from 'react';

export type DialogKey = 'location-info'; // | 'other-dialog-key' | 'and-so-on';

export type DialogState = Record<DialogKey, { open: boolean }>;

export type DialogContext = {
  readonly dialogs: DialogState;
  readonly isOpen: (key: DialogKey) => boolean;
  readonly show: (key: DialogKey) => void;
  readonly close: (key: DialogKey) => void;
};

const init: DialogContext = {
  dialogs: { 'location-info': { open: false } },
  isOpen: () => false,
  show: () => {
    /* SCARECROW */
  },
  close: () => {
    /* SCARECROW */
  },
};

export const DialogContext = createContext<DialogContext>(init);
// Access the Dialog context
export const useDialog = () => {
  return useContext(DialogContext);
};
