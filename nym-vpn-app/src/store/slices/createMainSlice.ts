import dayjs from 'dayjs';
import { StateCreator } from 'zustand';
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
  DiagnosticsSuggestedReason,
  FeatureFlags,
  FrontingMode,
  MixnetTrafficConfig,
  NetworkCompat,
  NetworkEnv,
  NodeHop,
  ProgressMsg,
  SelectedNode,
  SplitApp,
  TAccountMode,
  TAccountSummary,
  ThemeMode,
  Tunnel,
  TunnelAction,
  TunnelError,
  UiTheme,
  VpnMode,
  VpndConfig,
  VpndInfo,
} from '../../types';
import type { BoundStore } from '../types';

export type StateAction =
  | { type: 'init-done' }
  | { type: 'set-tunnel'; tunnel: Tunnel }
  | { type: 'set-tunnel-error'; error: TunnelError | null }
  | { type: 'set-daemon-status'; status: DaemonStatus }
  | { type: 'set-daemon-info'; info: VpndInfo }
  | { type: 'update-tunnel-config'; config: VpndConfig }
  | { type: 'set-vpn-mode'; mode: VpnMode }
  | { type: 'set-error'; error: AppError | null }
  | { type: 'reset-error' }
  | { type: 'new-progress-message'; message: ProgressMsg }
  | { type: 'connect' }
  | { type: 'disconnect' }
  | { type: 'set-version'; version: string }
  | { type: 'set-linux-app-updated'; updated: boolean }
  | {
      type: 'set-diagnostics-suggested-reason';
      reason: DiagnosticsSuggestedReason | null;
    }
  | { type: 'set-tunnel-connected'; tunnel: Tunnel }
  | { type: 'set-tunnel-disconnected' }
  | { type: 'set-tunnel-connecting'; state: ConnectingState }
  | { type: 'set-tunnel-disconnecting'; action: TunnelAction | null }
  | { type: 'set-tunnel-offline'; reconnect: boolean | null }
  | { type: 'set-tunnel-inerror'; error: TunnelError }
  | { type: 'set-auto-connect'; autoConnect: boolean }
  | { type: 'set-monitoring'; enabled: boolean }
  | { type: 'set-debug-logging'; enabled: boolean }
  | { type: 'set-desktop-notifications'; enabled: boolean }
  | { type: 'set-gateway-independence-notifications'; enabled: boolean }
  | { type: 'reset' }
  | { type: 'set-ui-theme'; theme: UiTheme }
  | { type: 'set-theme-mode'; mode: ThemeMode }
  | { type: 'system-theme-changed'; theme: UiTheme }
  | { type: 'set-node'; payload: { hop: NodeHop; node: SelectedNode } }
  | { type: 'set-root-font-size'; size: number }
  | { type: 'set-code-deps-js'; dependencies: CodeDependency[] }
  | { type: 'set-code-deps-rust'; dependencies: CodeDependency[] }
  | { type: 'set-autostart'; enabled: boolean }
  | { type: 'set-account'; stored: boolean }
  | { type: 'set-account-links'; links: AccountLinks | null }
  | { type: 'set-network-compat'; compat: NetworkCompat | null }
  | { type: 'set-ipv6-support'; enabled: boolean }
  | { type: 'set-allow-lan'; enabled: boolean }
  | { type: 'set-enable-ad-blocking'; enabled: boolean }
  | { type: 'set-network-stats'; enabled: boolean }
  | { type: 'set-account-state'; state: AccountState }
  | { type: 'set-account-mode'; mode: TAccountMode }
  | { type: 'set-account-syncing'; syncing: boolean }
  | { type: 'set-technical-optin-seen'; seen: boolean }
  | { type: 'set-account-error'; error: AppError | null }
  | { type: 'set-backend-flags'; flags: FeatureFlags }
  | { type: 'set-quic'; enabled: boolean }
  | { type: 'set-fronting-mode'; mode: FrontingMode }
  | { type: 'set-custom-dns-enabled'; enabled: boolean }
  | { type: 'set-custom-dns'; dns: string[] }
  | { type: 'set-default-dns'; dns: string[] }
  | { type: 'set-mixnet-traffic-config'; config: MixnetTrafficConfig }
  | { type: 'set-account-summary'; summary: TAccountSummary | null }
  | { type: 'set-enable-split-tunnel'; enabled: boolean }
  | { type: 'set-split-tunnel-apps'; apps: SplitApp[] }
  | { type: 'set-geo-exclusion-enabled'; enabled: boolean }
  | { type: 'set-geo-exclusion-listen-port'; port: number }
  | { type: 'set-geo-exclusion-excluded-countries'; countries: string[] };

export const initialState: AppState = {
  initialized: false,
  state: 'disconnected',
  tunnel: null,
  tunnelError: null,
  accountState: null,
  accountMode: null,
  accountSummary: null,
  accountSyncing: false,
  accountError: null,
  daemonStatus: 'down',
  networkEnv: 'mainnet',
  version: null,
  linuxAppUpdated: false,
  diagnosticsSuggestedReason: null,
  vpnMode: 'wg',
  uiTheme: 'light',
  themeMode: DefaultThemeMode,
  progressMessages: [],
  autostart: false,
  autoConnect: false,
  monitoring: false,
  debugLogging: true,
  desktopNotifications: true,
  entryNode: DefaultNode,
  exitNode: DefaultNode,
  rootFontSize: DefaultRootFontSize,
  codeDepsRust: [],
  codeDepsJs: [],
  account: false,
  ipv6Support: true,
  networkStats: false,
  technicalOptinSeen: false,
  quic: false,
  allowLan: false,
  frontingMode: 'onRetry',
  enableAdBlocking: false,
  backendFlags: {
    quic: false,
    domainFronting: false,
    zknymCredential: false,
  },
  customDnsEnabled: false,
  customDns: [],
  defaultDns: [],
  mixnetTrafficConfig: {
    poissonParameterForLoopCoverStream: null,
    averagePacketDelay: null,
    messageSendingAverageDelay: null,
    disablePoissonRate: false,
    disableBackgroundCoverTraffic: false,
    minMixnodePerformance: null,
    minGatewayMixnetPerformance: null,
  },
  mixnetTrafficDefaults: {
    mixingDelay: { minValue: 0, maxValue: 0, defaultValue: 0 },
    disablePoissonRate: false,
    defaultBackgroundTraffic: { value: 0, multiplier: '' },
    defaultContinuousTraffic: { value: 0, throughput: '' },
    allBackgroundTraffic: [],
    allContinuousTraffic: [],
  },
  splitTunnel: { enabled: false, apps: [] },
  geoExclusion: { enabled: false, listenPort: 1080, excludedCountries: ['CN'] },
  gatewaySelectionAlgorithmConfig: { enableGeoLocation: true },
  gatewayIndependenceNotifications: true,
};

export type MainSlice = AppState & {
  _dispatch: (action: StateAction) => void;
};

export const createMainSlice: StateCreator<BoundStore, [], [], MainSlice> = (
  set,
  get,
) => ({
  ...initialState,

  _dispatch(action) {
    switch (action.type) {
      case 'init-done':
        set({ initialized: true });
        break;

      case 'set-daemon-status':
        if (action.status === 'down') {
          set({
            daemonStatus: action.status,
            state: 'unknown',
            tunnel: null,
            progressMessages: [],
            tunnelConnectedAt: null,
            tunnelError: null,
            connectingState: null,
            accountSummary: null,
            account: false,
            accountMode: null,
            error: {
              key: 'not-connected-to-daemon',
              message: 'Not connected to the daemon',
            },
          });
        } else if (action.status === 'auth-denied') {
          set({
            daemonStatus: action.status,
            state: 'unknown',
            tunnel: null,
            progressMessages: [],
            tunnelConnectedAt: null,
            tunnelError: null,
            connectingState: null,
            accountSummary: null,
            account: false,
            accountMode: null,
            error: { key: 'auth-denied', message: 'Authentication required' },
          });
        } else {
          set({ daemonStatus: action.status, error: null });
        }
        break;

      case 'update-tunnel-config':
        set({
          entryNode: action.config.entryNode,
          exitNode: action.config.exitNode,
          vpnMode: action.config.vpnMode,
          quic: action.config.bridges,
          frontingMode: action.config.frontingMode,
          ipv6Support: !action.config.disableIpv6,
          allowLan: action.config.allowLan,
          customDnsEnabled: action.config.enableCustomDns,
          customDns: action.config.customDns ?? [],
          mixnetTrafficConfig: action.config.mixnetTraffic,
          mixnetTrafficDefaults: action.config.mixnetTrafficDefaults,
          gatewayIndependenceNotifications:
            action.config.gatewayIndependenceNotifications,
          enableAdBlocking: action.config.enableAdBlocking,
          splitTunnel: action.config.splitTunnel,
          geoExclusion: action.config.geoExclusion,
          gatewaySelectionAlgorithmConfig:
            action.config.gatewaySelectionAlgorithmConfig,
        });
        break;

      case 'set-daemon-info':
        set({
          daemonVersion: action.info.version,
          networkEnv: action.info.network as NetworkEnv,
        });
        break;

      case 'set-node':
        if (action.payload.hop === 'entry') {
          set({ entryNode: action.payload.node });
        } else {
          set({ exitNode: action.payload.node });
        }
        break;

      case 'set-vpn-mode':
        set({ vpnMode: action.mode });
        break;

      case 'set-auto-connect':
        set({ autoConnect: action.autoConnect });
        break;

      case 'set-monitoring':
        set({ monitoring: action.enabled });
        break;

      case 'set-debug-logging':
        set({ debugLogging: action.enabled });
        break;

      case 'set-ipv6-support':
        set({ ipv6Support: action.enabled });
        break;

      case 'set-allow-lan':
        set({ allowLan: action.enabled });
        break;

      case 'set-enable-ad-blocking':
        set({ enableAdBlocking: action.enabled });
        break;

      case 'set-desktop-notifications':
        set({ desktopNotifications: action.enabled });
        break;

      case 'set-gateway-independence-notifications':
        set({ gatewayIndependenceNotifications: action.enabled });
        break;

      case 'set-tunnel':
        set({ tunnel: action.tunnel });
        break;

      case 'set-tunnel-error':
        set({ tunnelError: action.error });
        break;

      case 'connect':
        set({ state: 'connecting' });
        break;

      case 'disconnect':
        set({ state: 'disconnecting' });
        break;

      case 'set-version':
        set({ version: action.version });
        break;

      case 'set-linux-app-updated':
        set({ linuxAppUpdated: action.updated });
        break;

      case 'set-diagnostics-suggested-reason':
        set({ diagnosticsSuggestedReason: action.reason });
        break;

      case 'set-tunnel-connected':
        set({
          state: 'connected',
          tunnel: action.tunnel,
          progressMessages: [],
          tunnelConnectedAt: action.tunnel.connectedAt
            ? dayjs.unix(action.tunnel.connectedAt as unknown as number)
            : dayjs(),
          tunnelError: null,
          error: null,
          connectingState: null,
        });
        break;

      case 'set-tunnel-disconnected':
        set({
          state: 'disconnected',
          tunnel: null,
          progressMessages: [],
          tunnelConnectedAt: null,
          tunnelError: null,
          connectingState: null,
        });
        break;

      case 'set-tunnel-connecting':
        set({
          state: 'connecting',
          connectingState: action.state,
          tunnelError: null,
        });
        break;

      case 'set-tunnel-disconnecting':
        set({
          state: 'disconnecting',
          tunnel: null,
          tunnelError: null,
          connectingState: null,
        });
        break;

      case 'set-tunnel-offline':
        set({
          state: action.reconnect ? 'offline-auto-reconnect' : 'offline',
          tunnel: null,
          tunnelError: null,
          connectingState: null,
        });
        break;

      case 'set-tunnel-inerror':
        set({
          state: 'error',
          tunnelError: action.error,
          connectingState: null,
        });
        break;

      case 'set-account':
        set({ account: action.stored });
        break;

      case 'set-error':
        set({ error: action.error });
        break;

      case 'reset-error':
        set({ error: null, tunnelError: null });
        break;

      case 'new-progress-message':
        set((s) => ({
          progressMessages: [...s.progressMessages, action.message],
        }));
        break;

      case 'set-ui-theme':
        set({ uiTheme: action.theme });
        break;

      case 'set-theme-mode':
        set({ themeMode: action.mode });
        break;

      case 'system-theme-changed': {
        const { themeMode, uiTheme } = get();
        if (themeMode === 'system' && uiTheme !== action.theme) {
          set({ uiTheme: action.theme });
        }
        break;
      }

      case 'set-root-font-size':
        set({ rootFontSize: action.size });
        break;

      case 'set-code-deps-js':
        set({ codeDepsJs: action.dependencies });
        break;

      case 'set-code-deps-rust':
        set({ codeDepsRust: action.dependencies });
        break;

      case 'set-account-links':
        set({ accountLinks: action.links });
        break;

      case 'set-autostart':
        set({ autostart: action.enabled });
        break;

      case 'set-network-compat':
        set({ networkCompat: action.compat });
        break;

      case 'set-network-stats':
        set({ networkStats: action.enabled });
        break;

      case 'set-account-state':
        set({ accountState: action.state });
        break;

      case 'set-account-mode':
        set({ accountMode: action.mode });
        break;

      case 'set-account-syncing':
        set({ accountSyncing: action.syncing });
        break;

      case 'set-account-error':
        set({ accountError: action.error });
        break;

      case 'set-technical-optin-seen':
        set({ technicalOptinSeen: action.seen });
        break;

      case 'set-backend-flags':
        set({ backendFlags: action.flags });
        break;

      case 'set-quic':
        set({ quic: action.enabled });
        break;

      case 'set-fronting-mode':
        set({ frontingMode: action.mode });
        break;

      case 'set-custom-dns-enabled':
        set({ customDnsEnabled: action.enabled });
        break;

      case 'set-custom-dns':
        set({ customDns: action.dns });
        break;

      case 'set-default-dns':
        set({ defaultDns: action.dns });
        break;

      case 'set-mixnet-traffic-config':
        set({ mixnetTrafficConfig: action.config });
        break;

      case 'set-account-summary':
        set({ accountSummary: action.summary });
        break;

      case 'set-enable-split-tunnel':
        set((s) => ({
          splitTunnel: { ...s.splitTunnel, enabled: action.enabled },
        }));
        break;

      case 'set-split-tunnel-apps':
        set((s) => ({ splitTunnel: { ...s.splitTunnel, apps: action.apps } }));
        break;

      case 'set-geo-exclusion-enabled':
        set((s) => ({
          geoExclusion: { ...s.geoExclusion, enabled: action.enabled },
        }));
        break;

      case 'set-geo-exclusion-listen-port':
        set((s) => ({
          geoExclusion: { ...s.geoExclusion, listenPort: action.port },
        }));
        break;

      case 'set-geo-exclusion-excluded-countries':
        set((s) => ({
          geoExclusion: {
            ...s.geoExclusion,
            excludedCountries: action.countries,
          },
        }));
        break;

      case 'reset':
        set({ ...initialState });
        break;
    }
  },
});
