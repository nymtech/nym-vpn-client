import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../state';
import { Country, NodeLocation, ThemeMode, UiTheme } from './common';
import { BackendError, BkdErrorKey } from './tauri-ipc';

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

export type CodeDependency = {
  name: string;
  version?: string;
  licenses: string[];
  licenseTexts: string[];
  repository?: string;
  authors: string[];
  copyright?: string;
};

export type WindowSize = {
  type: 'Physical' | 'Logical';
  width: number;
  height: number;
};

export type DaemonStatus = 'Ok' | 'NotOk';

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: ConnectionState;
  daemonStatus: DaemonStatus;
  version: string | null;
  loading: boolean;
  error?: AppError | null;
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
  entryCountryList: Country[];
  exitCountryList: Country[];
  entryCountriesLoading: boolean;
  exitCountriesLoading: boolean;
  rootFontSize: number;
  codeDepsJs: CodeDependency[];
  codeDepsRust: CodeDependency[];
  windowSize?: WindowSize | null;
  fetchEntryCountries: FetchCountriesFn;
  fetchExitCountries: FetchCountriesFn;
};

export type ConnectionEventPayload = {
  state: ConnectionState;
  error?: BackendError | null;
  start_time?: bigint | null; // unix timestamp in seconds
};

export type ConnectProgressMsg = 'Initializing' | 'InitDone';

export type ProgressEventPayload = {
  key: ConnectProgressMsg;
};

export type StateDispatch = Dispatch<StateAction>;

export type FetchCountriesFn = () => Promise<void> | undefined;

export type AppError = {
  message: string;
  key?: BkdErrorKey | null;
  data?: Record<string, string> | null;
};
