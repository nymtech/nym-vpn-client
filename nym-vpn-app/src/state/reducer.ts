import dayjs from 'dayjs';
import * as _ from 'lodash-es';
import {
  DefaultCountry,
  DefaultRootFontSize,
  DefaultThemeMode,
} from '../constants';
import {
  AccountLinks,
  AppError,
  AppState,
  CodeDependency,
  ConnectProgressMsg,
  Country,
  DaemonInfo,
  DaemonStatus,
  Gateway,
  GatewayType,
  GatewaysByCountry,
  NodeHop,
  ThemeMode,
  Tunnel,
  TunnelAction,
  TunnelError,
  UiTheme,
  VpnMode,
} from '../types';
import { S_STATE } from '../static';

export type StateAction =
  | { type: 'init-done' }
  | { type: 'set-tunnel'; tunnel: Tunnel }
  | { type: 'set-tunnel-error'; error: TunnelError | null }
  | { type: 'set-daemon-status'; status: DaemonStatus }
  | { type: 'set-daemon-info'; info: DaemonInfo }
  | { type: 'set-vpn-mode'; mode: VpnMode }
  | { type: 'set-error'; error: AppError }
  | { type: 'reset-error' }
  | { type: 'new-progress-message'; message: ConnectProgressMsg }
  | { type: 'connect' }
  | { type: 'disconnect' }
  | { type: 'set-version'; version: string }
  | { type: 'set-tunnel-connected'; tunnel: Tunnel }
  | { type: 'set-tunnel-disconnected' }
  | { type: 'set-tunnel-connecting'; tunnel: Tunnel | null }
  | { type: 'set-tunnel-disconnecting'; action: TunnelAction | null }
  | { type: 'set-tunnel-offline'; reconnect: boolean | null }
  | { type: 'set-tunnel-inerror'; error: TunnelError }
  | { type: 'set-auto-connect'; autoConnect: boolean }
  | { type: 'set-monitoring'; monitoring: boolean }
  | { type: 'set-desktop-notifications'; enabled: boolean }
  | { type: 'reset' }
  | { type: 'set-ui-theme'; theme: UiTheme }
  | { type: 'set-theme-mode'; mode: ThemeMode }
  | { type: 'system-theme-changed'; theme: UiTheme }
  | {
      type: 'set-gateways';
      payload: { type: GatewayType; gateways: GatewaysByCountry[] };
    }
  | {
      type: 'set-gateways-loading';
      payload: { type: GatewayType; loading: boolean };
    }
  | {
      type: 'set-gateways-error';
      payload: { type: GatewayType; error: AppError | null };
    }
  | {
      type: 'set-node';
      payload: { hop: NodeHop; node: Country | Gateway };
    }
  | { type: 'set-root-font-size'; size: number }
  | { type: 'set-code-deps-js'; dependencies: CodeDependency[] }
  | { type: 'set-code-deps-rust'; dependencies: CodeDependency[] }
  | { type: 'set-autostart'; enabled: boolean }
  | { type: 'set-account'; stored: boolean }
  | { type: 'set-account-links'; links: AccountLinks | null };

export const initialState: AppState = {
  initialized: false,
  state: 'Disconnected',
  tunnel: null,
  tunnelError: null,
  daemonStatus: 'down',
  networkEnv: 'mainnet',
  version: null,
  vpnMode: S_STATE.vpnModeAtStart,
  uiTheme: 'Light',
  themeMode: DefaultThemeMode,
  progressMessages: [],
  autostart: false,
  autoConnect: false,
  monitoring: false,
  desktopNotifications: true,
  entryNode: DefaultCountry,
  exitNode: DefaultCountry,
  mxEntryGateways: [],
  mxExitGateways: [],
  wgGateways: [],
  mxEntryGatewaysLoading: true,
  mxExitGatewaysLoading: true,
  wgGatewaysLoading: true,
  rootFontSize: DefaultRootFontSize,
  codeDepsRust: [],
  codeDepsJs: [],
  account: false,
  fetchGateways: async () => {
    /*  SCARECROW */
  },
};

export function reducer(state: AppState, action: StateAction): AppState {
  switch (action.type) {
    case 'init-done':
      return {
        ...state,
        initialized: true,
      };
    case 'set-daemon-status':
      return {
        ...state,
        daemonStatus: action.status,
      };
    case 'set-daemon-info':
      return {
        ...state,
        daemonVersion: action.info.version,
        networkEnv: action.info.network,
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
        monitoring: action.monitoring,
      };
    case 'set-desktop-notifications':
      return {
        ...state,
        desktopNotifications: action.enabled,
      };
    case 'set-gateways': {
      let prop: keyof AppState = 'wgGateways';
      if (action.payload.type === 'mx-entry') {
        prop = 'mxEntryGateways';
      }
      if (action.payload.type === 'mx-exit') {
        prop = 'mxExitGateways';
      }
      if (!_.isEqual(action.payload.gateways, state[prop])) {
        return {
          ...state,
          [prop]: action.payload.gateways,
        };
      }
      return state;
    }
    case 'set-gateways-loading':
      if (action.payload.type === 'mx-entry') {
        return {
          ...state,
          mxEntryGatewaysLoading: action.payload.loading,
        };
      }
      if (action.payload.type === 'mx-exit') {
        return {
          ...state,
          mxExitGatewaysLoading: action.payload.loading,
        };
      }
      return {
        ...state,
        wgGatewaysLoading: action.payload.loading,
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
      return { ...state, state: 'Connecting' };
    case 'disconnect':
      return { ...state, state: 'Disconnecting' };
    case 'set-version':
      return {
        ...state,
        version: action.version,
      };
    case 'set-tunnel-connected':
      return {
        ...state,
        state: 'Connected',
        tunnel: action.tunnel,
        progressMessages: [],
        tunnelConnectedAt: action.tunnel.connectedAt
          ? dayjs.unix(action.tunnel.connectedAt)
          : dayjs(),
        tunnelError: null,
        error: null,
      };
    case 'set-tunnel-disconnected':
      return {
        ...state,
        state: 'Disconnected',
        tunnel: null,
        progressMessages: [],
        tunnelConnectedAt: null,
        tunnelError: null,
      };
    case 'set-tunnel-connecting':
      return {
        ...state,
        state: 'Connecting',
        tunnel: action.tunnel,
        tunnelError: null,
      };
    case 'set-tunnel-disconnecting':
      return {
        ...state,
        state: 'Disconnecting',
        tunnel: null,
        tunnelError: null,
      };
    case 'set-tunnel-offline':
      return {
        ...state,
        state: action.reconnect ? 'OfflineAutoReconnect' : 'Offline',
        tunnel: null,
        tunnelError: null,
      };
    case 'set-tunnel-inerror':
      return {
        ...state,
        state: 'Error',
        tunnelError: action.error,
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
      if (state.themeMode === 'System' && state.uiTheme !== action.theme) {
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
    case 'set-gateways-error':
      if (action.payload.type === 'mx-entry') {
        return {
          ...state,
          mxEntryGatewaysError: action.payload.error,
        };
      }
      if (action.payload.type === 'mx-exit') {
        return {
          ...state,
          mxExitGatewaysError: action.payload.error,
        };
      }
      return {
        ...state,
        wgGatewaysError: action.payload.error,
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

    case 'reset':
      return initialState;
  }
}
