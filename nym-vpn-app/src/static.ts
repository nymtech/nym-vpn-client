// global state managed out of the React tree
import { VpnMode } from './types';

export type SState = {
  vpnModeInit: boolean;
  vpnModeAtStart: VpnMode;
  systemMessageInit: boolean;
  devMode: boolean;
};

export const S_STATE: SState = {
  // Either the vpn mode has been initialized or not
  vpnModeInit: false,
  vpnModeAtStart: 'wg',
  systemMessageInit: false,
  devMode: false,
};
