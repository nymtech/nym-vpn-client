import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../state';
import { Country, NodeHop, NodeLocation, ThemeMode, UiTheme } from './common';
import { AccountLinks, BackendError, ErrorKey, NetworkEnv } from './tauri-ipc';

export type ConnectionState =
  | 'Connected'
  | 'Disconnected'
  | 'Connecting'
  | 'Disconnecting'
  | 'Unknown';

export type VpnMode = 'TwoHop' | 'Mixnet';

export type CodeDependency = {
  name: string;
  version?: string;
  licenses: string[];
  repository?: string;
  authors: string[];
  copyright?: string;
};

export type DaemonStatus = 'Ok' | 'NonCompat' | 'NotOk';

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: ConnectionState;
  daemonStatus: DaemonStatus;
  daemonVersion?: string;
  networkEnv?: NetworkEnv;
  version: string | null;
  error?: AppError | null;
  progressMessages: ConnectProgressMsg[];
  sessionStartDate?: Dayjs | null;
  vpnMode: VpnMode;
  // `UiTheme` is the current applied theme to the UI, that is either `Dark` or `Light`
  uiTheme: UiTheme;
  // `themeMode` is the current user selected mode, could be `System`, `Dark` or `Light`
  //  if `System` is selected, the app will follow the system theme
  themeMode: ThemeMode;
  autoConnect: boolean;
  monitoring: boolean;
  desktopNotifications: boolean;
  entryNodeLocation: NodeLocation;
  exitNodeLocation: NodeLocation;
  fastestNodeLocation: Country;
  entryCountryList: Country[];
  exitCountryList: Country[];
  entryCountriesLoading: boolean;
  exitCountriesLoading: boolean;
  entryCountriesError?: AppError | null;
  exitCountriesError?: AppError | null;
  rootFontSize: number;
  codeDepsJs: CodeDependency[];
  codeDepsRust: CodeDependency[];
  // TODO just a boolean for now to indicate if the user has added an account
  account: boolean;
  accountLinks?: AccountLinks | null;
  fetchMnCountries: FetchMnCountriesFn;
  fetchWgCountries: FetchWgCountriesFn;
};

export type ConnectionEvent =
  | ({ type: 'Update' } & ConnectionEventPayload)
  | ({ type: 'Failed' } & (BackendError | null));

export type ConnectionEventPayload = {
  state: ConnectionState;
  error?: BackendError | null;
  start_time?: bigint | null; // unix timestamp in seconds
};

export type ConnectProgressMsg = 'Initializing' | 'InitDone' | 'Cancelling';

export type ProgressEventPayload = {
  key: ConnectProgressMsg;
};

export type StateDispatch = Dispatch<StateAction>;

export type FetchMnCountriesFn = (node: NodeHop) => Promise<void> | undefined;
export type FetchWgCountriesFn = () => Promise<void> | undefined;

export type AppError = {
  message: string;
  key: ErrorKey;
  data?: Record<string, string> | null;
};

export type StatusUpdate =
  | 'Unknown'
  | 'EntryGatewayConnectionEstablished'
  | 'ExitRouterConnectionEstablished'
  | 'TunnelEndToEndConnectionEstablished'
  | 'EntryGatewayNotRoutingMixnetMessages'
  | 'ExitRouterNotRespondingToIpv4Ping'
  | 'ExitRouterNotRespondingToIpv6Ping'
  | 'ExitRouterNotRoutingIpv4Traffic'
  | 'ExitRouterNotRoutingIpv6Traffic'
  | 'ConnectionOkIpv4'
  | 'ConnectionOkIpv6'
  | 'RemainingBandwidth'
  | 'NoBandwidth';

export type StatusUpdatePayload = {
  status: StatusUpdate;
  message: string;
  data?: Record<string, string> | null;
  error?: BackendError | null;
};
