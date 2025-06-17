// global state managed out of the React tree
import { UiTheme, VpnMode, VpndStatus } from './types';

export type SState = {
  // the connection status with the daemon at startup
  vpnd: VpndStatus;
  // either the vpn mode has been initialized or not
  vpnModeInit: boolean;
  uiTheme: UiTheme;
  vpnModeAtStart: VpnMode;
  systemMessageInit: boolean;
  welcomeScreenSeen: boolean;
};

export const S_STATE: SState = {
  vpnd: 'down',
  vpnModeInit: false,
  vpnModeAtStart: 'wg',
  uiTheme: 'light',
  systemMessageInit: false,
  welcomeScreenSeen: false,
};
