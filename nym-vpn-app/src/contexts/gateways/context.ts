import { createContext, useContext } from 'react';
import { GatewaysState } from './types';

export const initialState: GatewaysState = {
  mxEntry: [],
  mxExit: [],
  wg: [],
  mxEntryLoading: false,
  mxExitLoading: false,
  wgLoading: false,
  mxEntryError: null,
  mxExitError: null,
  wgError: null,
  fetch: async () => {
    /*  SCARECROW */
  },
};

export const GatewaysContext = createContext<GatewaysState>(initialState);
export const useGateways = () => {
  return useContext(GatewaysContext);
};
