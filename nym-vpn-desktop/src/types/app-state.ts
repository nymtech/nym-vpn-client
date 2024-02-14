import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../state';
import { Country, NodeLocation, ThemeMode, UiTheme } from './common';

export type ConnectionState =
  | 'Connected'
  | 'Disconnected'
  | 'Connecting'
  | 'Disconnecting'
  | 'Unknown';

export type VpnMode = 'TwoHop' | 'Mixnet';

export interface TunnelConfig {
  id: string;
  name: string;
}

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: ConnectionState;
  version: string | null;
  loading: boolean;
  error?: string | null;
  progressMessages: ConnectProgressMsg[];
  sessionStartDate?: Dayjs | null;
  vpnMode: VpnMode;
  tunnel: TunnelConfig;
  // `UiTheme` is the current applied theme to the UI, that is either `Dark` or `Light`
  uiTheme: UiTheme;
  // `themeMode` is the current user selected mode, could be `System`, `Dark` or `Light`
  //  if `System` is selected, the app will follow the system theme
  themeMode: ThemeMode;
  entrySelector: boolean;
  autoConnect: boolean;
  monitoring: boolean;
  entryNodeLocation: NodeLocation;
  exitNodeLocation: NodeLocation;
  fastestNodeLocation: Country;
  countryList: Country[];
  rootFontSize: number;
};

export type ConnectionEventPayload = {
  state: ConnectionState;
  error?: string | null;
  start_time?: number | null; // unix timestamp in seconds
};

export type ConnectProgressMsg = 'Initializing' | 'InitDone';

export type ProgressEventPayload = {
  key: ConnectProgressMsg;
};

export type StateDispatch = Dispatch<StateAction>;
