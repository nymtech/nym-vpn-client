import { Dayjs } from 'dayjs';
import {
  AccountLinks,
  ConnectingState,
  DiagnosticsSuggestedReason,
  FeatureFlags,
  FrontingMode,
  GatewaySelectionAlgorithmConfig,
  GeoExclusionSettings,
  MixnetTrafficConfig,
  MixnetTrafficDefaults,
  NetworkCompat,
  NetworkEnv,
  SelectedNode,
  SplitTunnelSettings,
  TAccountMode,
  TAccountSummary,
  ThemeMode,
  Tunnel,
  TunnelError,
  UiTheme,
  VpnMode,
  VpndStatus,
} from './tauri';
import {
  AccountState,
  AppError,
  CodeDependency,
  DaemonStatus,
  ProgressMsg,
  TunnelState,
} from './util';

// early stage state used to initialize the main app-state
export type InitState = {
  uiTheme: UiTheme;
  technicalOptinSeen: boolean;
  vpnMode: VpnMode;
  vpnd: VpndStatus;
  entryNode: SelectedNode;
  exitNode: SelectedNode;
  quic: boolean;
  noIpv6: boolean;
  allowLan: boolean;
  enableAdBlocking: boolean;
  customDnsEnabled: boolean;
  customDns: string[];
  mixnetTrafficConfig: MixnetTrafficConfig;
  mixnetTrafficDefaults: MixnetTrafficDefaults;
  splitTunnel: SplitTunnelSettings;
  geoExclusion: GeoExclusionSettings;
  gatewaySelectionAlgorithmConfig: GatewaySelectionAlgorithmConfig;
  frontingMode: FrontingMode;
  gatewayIndependenceNotifications: boolean;
};

export type AppState = {
  // initial loading phase when the app is starting and fetching data from the backend
  initialized: boolean;
  state: TunnelState;
  tunnel?: Tunnel | null;
  connectingState?: ConnectingState | null;
  tunnelError?: TunnelError | null;
  accountState?: AccountState | null;
  accountMode?: TAccountMode | null;
  accountSummary?: TAccountSummary | null;
  accountError?: AppError | null;
  accountSyncing: boolean;
  daemonStatus: DaemonStatus;
  daemonVersion?: string;
  // feature flags from backend and APIs (via daemon)
  backendFlags: FeatureFlags;
  networkEnv: NetworkEnv;
  version: string | null;
  linuxAppUpdated?: boolean;
  diagnosticsSuggestedReason?: DiagnosticsSuggestedReason | null;
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
  // app debug logging to a file
  debugLogging: boolean;
  desktopNotifications: boolean;
  entryNode: SelectedNode;
  exitNode: SelectedNode;
  rootFontSize: number;
  codeDepsJs: CodeDependency[];
  codeDepsRust: CodeDependency[];
  // just a boolean for now to indicate if the user has added an account
  account: boolean;
  accountLinks?: AccountLinks | null;
  networkCompat?: NetworkCompat | null;
  ipv6Support: boolean;
  allowLan: boolean;
  enableAdBlocking: boolean;
  networkStats: boolean;
  // whether the user has completed once the welcome screen
  technicalOptinSeen: boolean;
  // aka bridges mode
  quic: boolean;
  // current user setting
  customDnsEnabled: boolean;
  customDns: string[];
  defaultDns: string[];
  mixnetTrafficConfig: MixnetTrafficConfig;
  mixnetTrafficDefaults: MixnetTrafficDefaults;
  splitTunnel: SplitTunnelSettings;
  geoExclusion: GeoExclusionSettings;
  gatewaySelectionAlgorithmConfig: GatewaySelectionAlgorithmConfig;
  frontingMode: FrontingMode;
  // gateway-independence reminder toggle ("Server family reminders"); daemon-backed
  gatewayIndependenceNotifications: boolean;
};
