import dayjs from 'dayjs';
import {
  DefaultNode,
  DefaultRootFontSize,
  DefaultThemeMode,
} from '../../constants';
import {
  AccountLinks,
  AccountState,
  AppError,
  AppState,
  CodeDependency,
  ConnectingState,
  DaemonStatus,
  FeatureFlags,
  NetworkCompat,
  NetworkEnv,
  NodeHop,
  ProgressMsg,
  SelectedNode,
  ThemeMode,
  Tunnel,
  TunnelAction,
  TunnelError,
  UiTheme,
  VpnMode,
  VpndInfo,
} from '../../types';

export type StateAction =
  | { type: 'init-done' }
  | { type: 'set-tunnel'; tunnel: Tunnel }
  | { type: 'set-tunnel-error'; error: TunnelError | null }
  | { type: 'set-daemon-status'; status: DaemonStatus }
  | { type: 'set-daemon-info'; info: VpndInfo }
  | { type: 'set-vpn-mode'; mode: VpnMode }
  | { type: 'set-error'; error: AppError | null }
  | { type: 'reset-error' }
  | { type: 'new-progress-message'; message: ProgressMsg }
  | { type: 'connect' }
  | { type: 'disconnect' }
  | { type: 'set-version'; version: string }
  | { type: 'set-tunnel-connected'; tunnel: Tunnel }
  | { type: 'set-tunnel-disconnected' }
  | { type: 'set-tunnel-connecting'; state: ConnectingState }
  | { type: 'set-tunnel-disconnecting'; action: TunnelAction | null }
  | { type: 'set-tunnel-offline'; reconnect: boolean | null }
  | { type: 'set-tunnel-inerror'; error: TunnelError }
  | { type: 'set-auto-connect'; autoConnect: boolean }
  | { type: 'set-monitoring'; enabled: boolean }
  | { type: 'set-desktop-notifications'; enabled: boolean }
  | { type: 'reset' }
  | { type: 'set-ui-theme'; theme: UiTheme }
  | { type: 'set-theme-mode'; mode: ThemeMode }
  | { type: 'system-theme-changed'; theme: UiTheme }
  | {
      type: 'set-node';
      payload: { hop: NodeHop; node: SelectedNode };
    }
  | { type: 'set-root-font-size'; size: number }
  | { type: 'set-code-deps-js'; dependencies: CodeDependency[] }
  | { type: 'set-code-deps-rust'; dependencies: CodeDependency[] }
  | { type: 'set-autostart'; enabled: boolean }
  | { type: 'set-account'; stored: boolean }
  | { type: 'set-account-links'; links: AccountLinks | null }
  | { type: 'set-network-compat'; compat: NetworkCompat | null }
  | { type: 'set-ipv6-support'; enabled: boolean }
  | { type: 'set-allow-lan'; enabled: boolean }
  | { type: 'set-network-stats'; enabled: boolean }
  | { type: 'set-account-state'; state: AccountState }
  | { type: 'set-account-syncing'; syncing: boolean }
  | { type: 'set-welcome-checked'; checked: boolean }
  | { type: 'set-account-error'; error: AppError | null }
  | { type: 'set-backend-flags'; flags: FeatureFlags }
  | { type: 'set-quic'; enabled: boolean }
  | { type: 'set-domain-fronting'; enabled: boolean }
  | { type: 'set-streaming-optimized-label-seen'; seen: boolean };

export const initialState: AppState = {
  initialized: false,
  state: 'disconnected',
  tunnel: null,
  tunnelError: null,
  accountState: null,
  accountSyncing: false,
  accountError: null,
  daemonStatus: 'down',
  networkEnv: 'mainnet',
  version: null,
  vpnMode: 'wg',
  uiTheme: 'light',
  themeMode: DefaultThemeMode,
  progressMessages: [],
  autostart: false,
  autoConnect: false,
  monitoring: false,
  desktopNotifications: true,
  entryNode: DefaultNode,
  exitNode: DefaultNode,
  rootFontSize: DefaultRootFontSize,
  codeDepsRust: [],
  codeDepsJs: [],
  account: false,
  ipv6Support: true,
  networkStats: false,
  welcomeChecked: false,
  quic: false,
  allowLan: false,
  domainFronting: false,
  backendFlags: {
    quic: false,
    domainFronting: false,
    zknymCredential: false,
  },
  streamingOptimizedLabelSeen: false,
};

export function reducer(state: AppState, action: StateAction): AppState {
  switch (action.type) {
    case 'init-done':
      return {
        ...state,
        initialized: true,
      };
    case 'set-daemon-status':
      if (action.status === 'down') {
        return {
          ...state,
          daemonStatus: action.status,
          state: 'unknown',
          tunnel: null,
          progressMessages: [],
          tunnelConnectedAt: null,
          tunnelError: null,
          connectingState: null,
          error: {
            key: 'not-connected-to-daemon',
            message: 'Not connected to the daemon',
          },
        };
      }
      return {
        ...state,
        daemonStatus: action.status,
        error: null,
      };
    case 'set-daemon-info':
      return {
        ...state,
        daemonVersion: action.info.version,
        networkEnv: action.info.network as NetworkEnv,
      };
    case 'set-node':
      if (action.payload.hop === 'entry') {
        return {
          ...state,
          entryNode: action.payload.node,
        };
      }
      return {
        ...state,
        exitNode: action.payload.node,
      };
    case 'set-vpn-mode':
      return {
        ...state,
        vpnMode: action.mode,
      };
    case 'set-auto-connect':
      return {
        ...state,
        autoConnect: action.autoConnect,
      };
    case 'set-monitoring':
      return {
        ...state,
        monitoring: action.enabled,
      };
    case 'set-ipv6-support':
      return {
        ...state,
        ipv6Support: action.enabled,
      };
    case 'set-allow-lan':
      return {
        ...state,
        allowLan: action.enabled,
      };
    case 'set-desktop-notifications':
      return {
        ...state,
        desktopNotifications: action.enabled,
      };
    case 'set-tunnel':
      return {
        ...state,
        tunnel: action.tunnel,
      };
    case 'set-tunnel-error':
      return {
        ...state,
        tunnelError: action.error,
      };
    case 'connect':
      return { ...state, state: 'connecting' };
    case 'disconnect':
      return { ...state, state: 'disconnecting' };
    case 'set-version':
      return {
        ...state,
        version: action.version,
      };
    case 'set-tunnel-connected':
      return {
        ...state,
        state: 'connected',
        tunnel: action.tunnel,
        progressMessages: [],
        tunnelConnectedAt: action.tunnel.connectedAt
          ? dayjs.unix(action.tunnel.connectedAt as unknown as number)
          : dayjs(),
        tunnelError: null,
        error: null,
        connectingState: null,
      };
    case 'set-tunnel-disconnected':
      return {
        ...state,
        state: 'disconnected',
        tunnel: null,
        progressMessages: [],
        tunnelConnectedAt: null,
        tunnelError: null,
        connectingState: null,
      };
    case 'set-tunnel-connecting':
      return {
        ...state,
        state: 'connecting',
        connectingState: action.state,
        tunnelError: null,
      };
    case 'set-tunnel-disconnecting':
      return {
        ...state,
        state: 'disconnecting',
        tunnel: null,
        tunnelError: null,
        connectingState: null,
      };
    case 'set-tunnel-offline':
      return {
        ...state,
        state: action.reconnect ? 'offline-auto-reconnect' : 'offline',
        tunnel: null,
        tunnelError: null,
        connectingState: null,
      };
    case 'set-tunnel-inerror':
      return {
        ...state,
        state: 'error',
        tunnelError: action.error,
        connectingState: null,
      };
    case 'set-account':
      return { ...state, account: action.stored };
    case 'set-error':
      return { ...state, error: action.error };
    case 'reset-error':
      return { ...state, error: null, tunnelError: null };
    case 'new-progress-message':
      return {
        ...state,
        progressMessages: [...state.progressMessages, action.message],
      };
    case 'set-ui-theme':
      return {
        ...state,
        uiTheme: action.theme,
      };
    case 'set-theme-mode':
      return {
        ...state,
        themeMode: action.mode,
      };
    case 'system-theme-changed':
      if (state.themeMode === 'system' && state.uiTheme !== action.theme) {
        return {
          ...state,
          uiTheme: action.theme,
        };
      }
      return state;
    case 'set-root-font-size':
      return {
        ...state,
        rootFontSize: action.size,
      };
    case 'set-code-deps-js':
      return {
        ...state,
        codeDepsJs: action.dependencies,
      };
    case 'set-code-deps-rust':
      return {
        ...state,
        codeDepsRust: action.dependencies,
      };
    case 'set-account-links':
      return {
        ...state,
        accountLinks: action.links,
      };
    case 'set-autostart':
      return {
        ...state,
        autostart: action.enabled,
      };
    case 'set-network-compat':
      return {
        ...state,
        networkCompat: action.compat,
      };
    case 'set-network-stats':
      return {
        ...state,
        networkStats: action.enabled,
      };
    case 'set-account-state':
      return {
        ...state,
        accountState: action.state,
      };
    case 'set-account-syncing':
      return {
        ...state,
        accountSyncing: action.syncing,
      };
    case 'set-account-error':
      return {
        ...state,
        accountError: action.error,
      };
    case 'set-welcome-checked':
      return {
        ...state,
        welcomeChecked: action.checked,
      };
    case 'set-streaming-optimized-label-seen':
      return {
        ...state,
        streamingOptimizedLabelSeen: action.seen,
      };
    case 'set-backend-flags':
      return {
        ...state,
        backendFlags: action.flags,
      };
    case 'set-quic':
      return {
        ...state,
        quic: action.enabled,
      };
    case 'set-domain-fronting':
      return {
        ...state,
        domainFronting: action.enabled,
      };

    case 'reset':
      return initialState;
  }
}
