import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../state';
import { Country, NodeLocation, UiTheme } from './common';

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
  state: ConnectionState;
  version: string | null;
  loading: boolean;
  error?: string | null;
  progressMessages: ConnectProgressMsg[];
  sessionStartDate?: Dayjs | null;
  vpnMode: VpnMode;
  tunnel: TunnelConfig;
  uiTheme: UiTheme;
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
