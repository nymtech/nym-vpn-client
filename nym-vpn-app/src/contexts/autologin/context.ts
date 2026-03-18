import { createContext, useContext } from 'react';
import { DeeplinkKind } from '../../types/tauri';

export type AutologinKind = Extract<
  DeeplinkKind,
  'autologinRenew' | 'autologinView'
>;

export type AutologinContextType = {
  autologin: (kind: AutologinKind) => Promise<void>;
};

const initialState: AutologinContextType = {
  autologin: async () => Promise.resolve(),
};

export const AutologinContext =
  createContext<AutologinContextType>(initialState);

export function useAutologin() {
  return useContext(AutologinContext);
}
