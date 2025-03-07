import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../contexts';
import { Country, ThemeMode, UiTheme } from './common';
import {
  AccountLinks,
  ErrorKey,
  Gateway,
  NetworkCompat,
  NetworkEnv,
} from './tauri';
import { Tunnel, TunnelError } from './tunnel';

export type TunnelState =
  | 'Connected'
  | 'Disconnected'
  | 'Connecting'
  | 'Disconnecting'
  | 'Error'
  | 'Offline'
  | 'OfflineAutoReconnect';

export type VpnMode = 'wg' | 'mixnet';

export type CodeDependency = {
  name: string;
  version?: string;
  licenses: string[];
  repository?: string;
  authors: string[];
  copyright?: string;
};

export type DaemonStatus = 'ok' | 'non-compat' | 'down';

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: TunnelState;
  tunnel?: Tunnel | null;
  tunnelError?: TunnelError | null;
  daemonStatus: DaemonStatus;
  daemonVersion?: string;
  networkEnv: NetworkEnv;
  version: string | null;
  error?: AppError | null;
  progressMessages: ConnectProgressMsg[];
  tunnelConnectedAt?: Dayjs | null;
  vpnMode: VpnMode;
  // `UiTheme` is the current applied theme to the UI, that is either `Dark` or `Light`
  uiTheme: UiTheme;
  // `themeMode` is the current user selected mode, could be `System`, `Dark` or `Light`
  //  if `System` is selected, the app follows the system theme
  themeMode: ThemeMode;
  autostart: boolean;
  autoConnect: boolean;
  monitoring: boolean;
  desktopNotifications: boolean;
  entryNode: Country | Gateway;
  exitNode: Country | Gateway;
  rootFontSize: number;
  codeDepsJs: CodeDependency[];
  codeDepsRust: CodeDependency[];
  // TODO just a boolean for now to indicate if the user has added an account
  account: boolean;
  accountLinks?: AccountLinks | null;
  networkCompat?: NetworkCompat | null;
};

export type ConnectProgressMsg = 'Initializing' | 'InitDone' | 'Canceling';

export type ProgressEventPayload = {
  key: ConnectProgressMsg;
};

export type StateDispatch = Dispatch<StateAction>;

export type AppError = {
  message: string;
  key: ErrorKey;
  data?: Record<string, string> | null;
};
