// global state managed out of the React tree
import { VpnMode } from './types';

export type SState = {
  vpnModeInit: boolean;
  vpnModeAtStart: VpnMode;
  networkEnvInit: boolean;
  systemMessageInit: boolean;
  devMode: boolean;
};

export const S_STATE: SState = {
  // Either the vpn mode has been initialized or not
  vpnModeInit: false,
  vpnModeAtStart: 'TwoHop',
  networkEnvInit: false,
  systemMessageInit: false,
  devMode: false,
};
