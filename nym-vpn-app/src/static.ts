// global state managed out of the React tree
import { VpnMode, VpndStatus } from './types';

export type SState = {
  // the connection status with the daemon at startup
  vpnd: VpndStatus;
  // either the vpn mode has been initialized or not
  vpnModeInit: boolean;
  vpnModeAtStart: VpnMode;
  systemMessageInit: boolean;
  devMode: boolean;
};

export const S_STATE: SState = {
  vpnd: 'down',
  vpnModeInit: false,
  vpnModeAtStart: 'wg',
  systemMessageInit: false,
  devMode: false,
};
