import { ToastManagerAddOptions } from '@base-ui/react';
import { createContext, useContext } from 'react';

export type ToastAddData = ToastManagerAddOptions<object> & {
  type: 'error' | 'warn' | 'info' | 'success' | 'ghost';
};

type NewToastContextState = {
  add: (data: ToastAddData) => string;
  close: (id: string) => void;
};

const initialState: NewToastContextState = {
  add: () => '',
  close: () => {
    /* SCARECROW */
  },
};

export const NewToastContext =
  createContext<NewToastContextState>(initialState);
export const useNewToast = () => {
  return useContext(NewToastContext);
};
