import { createContext, useContext } from 'react';

export type GwIndependenceWarningContext = {
  readonly isOpen: boolean;
  readonly requestConfirmation: () => Promise<boolean>;
  readonly accept: () => void;
  readonly cancel: () => void;
};

const init: GwIndependenceWarningContext = {
  isOpen: false,
  requestConfirmation: () => Promise.resolve(false),
  accept: () => {
    /* SCARECROW */
  },
  cancel: () => {
    /* SCARECROW */
  },
};

export const GwIndependenceWarningContext =
  createContext<GwIndependenceWarningContext>(init);

export const useGwIndependenceWarning = () =>
  useContext(GwIndependenceWarningContext);
