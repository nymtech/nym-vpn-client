import { Dispatch } from 'react';
import { Dayjs } from 'dayjs';
import { StateAction } from '../contexts';
import { Country, ThemeMode, UiTheme } from './common';
import {
  AccountLinks,
  AccountState,
  ErrorKey,
  FeatureFlags,
  Gateway,
  NetworkCompat,
  NetworkEnv,
  VpndStatus,
} from './tauri';
import { ConnectingState, Tunnel, TunnelError } from './tunnel';

export type TunnelState =
  | 'connected'
  | 'disconnected'
  | 'connecting'
  | 'disconnecting'
  | 'error'
  | 'offline'
  | 'offline-auto-reconnect'
  // when not connected to the daemon, the state is unknown
  | 'unknown';

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

// early stage state used to initialize the main app-state
export type InitState = {
  uiTheme: UiTheme;
  welcomeChecked: boolean;
  vpnMode: VpnMode;
  vpnd: VpndStatus;
};

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: TunnelState;
  tunnel?: Tunnel | null;
  connectingState?: ConnectingState | null;
  tunnelError?: TunnelError | null;
  accountState?: AccountState | null;
  accountError?: AppError | null;
  accountSyncing: boolean;
  daemonStatus: DaemonStatus;
  daemonVersion?: string;
  // feature flags from backend and APIs (via daemon)
  backendFlags?: FeatureFlags | null;
  networkEnv: NetworkEnv;
  version: string | null;
  error?: AppError | null;
  // general progress messages to show in the main badge
  progressMessages: ProgressMsg[];
  tunnelConnectedAt?: Dayjs | null;
  vpnMode: VpnMode;
  // `UiTheme` is the current applied theme to the UI, that is either `dark` or `light`
  uiTheme: UiTheme;
  // `themeMode` is the current user selected mode, could be `system`, `dark` or `light`
  //  if `system` is selected, the app follows the system theme
  themeMode: ThemeMode;
  autostart: boolean;
  autoConnect: boolean;
  // error monitoring
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
  ipv6Support: boolean;
  networkStats: boolean;
  // whether the user has completed once the welcome screen
  welcomeChecked: boolean;
};

export type ProgressMsg = 'canceling';

export type StateDispatch = Dispatch<StateAction>;

export type AppError = {
  message: string;
  key: ErrorKey;
  data?: Record<string, string> | null;
};
