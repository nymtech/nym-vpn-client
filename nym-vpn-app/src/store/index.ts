import { create } from 'zustand';
import { useShallow } from 'zustand/react/shallow';
import type { AppState, InitState } from '../types';
import { createGatewaysSlice } from './slices/gateways/createGatewaysSlice';
import type { GatewayListsState, GatewaysSlice } from './slices/gateways/types';
import { createMainSlice } from './slices/createMainSlice';
import type { MainSlice, StateAction } from './slices/createMainSlice';
import { createSocks5Slice } from './slices/createSocks5Slice';
import type { Socks5Slice } from './slices/createSocks5Slice';

export type AppStore = MainSlice & GatewaysSlice & Socks5Slice;

export const useAppStore = create<AppStore>()((...args) => ({
  ...createMainSlice(...args),
  ...createGatewaysSlice(...args),
  ...createSocks5Slice(...args),
}));

export function dispatch(action: StateAction): void {
  useAppStore.getState()._dispatch(action);
}

let storeInitialized = false;

export function initMainStore(init: InitState): void {
  if (storeInitialized) return;
  storeInitialized = true;
  useAppStore.setState({
    vpnMode: init.vpnMode,
    uiTheme: init.uiTheme,
    technicalOptinSeen: init.technicalOptinSeen,
    entryNode: init.entryNode,
    exitNode: init.exitNode,
    quic: init.quic,
    enableAdBlocking: init.enableAdBlocking,
    enableConflictDetection: init.enableConflictDetection,
    ipv6Support: !init.noIpv6,
    allowLan: init.allowLan,
    customDnsEnabled: init.customDnsEnabled,
    customDns: init.customDns,
    mixnetTrafficConfig: init.mixnetTrafficConfig,
    mixnetTrafficDefaults: init.mixnetTrafficDefaults,
    splitTunnel: init.splitTunnel,
    geoExclusion: init.geoExclusion,
    gatewaySelectionAlgorithmConfig: init.gatewaySelectionAlgorithmConfig,
    frontingMode: init.frontingMode,
    gatewayIndependenceNotifications: init.gatewayIndependenceNotifications,
  });
}

// main state
export const useMainState = (): AppState =>
  useAppStore(
    useShallow((s) => ({
      initialized: s.initialized,
      state: s.state,
      tunnel: s.tunnel,
      connectingState: s.connectingState,
      tunnelError: s.tunnelError,
      accountState: s.accountState,
      accountMode: s.accountMode,
      accountSummary: s.accountSummary,
      accountError: s.accountError,
      accountSyncing: s.accountSyncing,
      daemonStatus: s.daemonStatus,
      daemonVersion: s.daemonVersion,
      backendFlags: s.backendFlags,
      networkEnv: s.networkEnv,
      version: s.version,
      linuxAppUpdated: s.linuxAppUpdated,
      diagnosticsSuggestedReason: s.diagnosticsSuggestedReason,
      error: s.error,
      progressMessages: s.progressMessages,
      tunnelConnectedAt: s.tunnelConnectedAt,
      vpnMode: s.vpnMode,
      uiTheme: s.uiTheme,
      themeMode: s.themeMode,
      autostart: s.autostart,
      autoConnect: s.autoConnect,
      monitoring: s.monitoring,
      debugLogging: s.debugLogging,
      desktopNotifications: s.desktopNotifications,
      entryNode: s.entryNode,
      exitNode: s.exitNode,
      rootFontSize: s.rootFontSize,
      codeDepsJs: s.codeDepsJs,
      codeDepsRust: s.codeDepsRust,
      account: s.account,
      accountLinks: s.accountLinks,
      networkCompat: s.networkCompat,
      ipv6Support: s.ipv6Support,
      allowLan: s.allowLan,
      enableAdBlocking: s.enableAdBlocking,
      enableConflictDetection: s.enableConflictDetection,
      networkStats: s.networkStats,
      technicalOptinSeen: s.technicalOptinSeen,
      quic: s.quic,
      customDnsEnabled: s.customDnsEnabled,
      customDns: s.customDns,
      defaultDns: s.defaultDns,
      mixnetTrafficConfig: s.mixnetTrafficConfig,
      mixnetTrafficDefaults: s.mixnetTrafficDefaults,
      splitTunnel: s.splitTunnel,
      geoExclusion: s.geoExclusion,
      gatewaySelectionAlgorithmConfig: s.gatewaySelectionAlgorithmConfig,
      frontingMode: s.frontingMode,
      gatewayIndependenceNotifications: s.gatewayIndependenceNotifications,
    })),
  );

// gateways state
export const useGateways = (): GatewayListsState =>
  useAppStore(
    useShallow((s) => ({
      mxEntry: s.mxEntry,
      mxExit: s.mxExit,
      wg: s.wg,
      mxEntryLoading: s.mxEntryLoading,
      mxExitLoading: s.mxExitLoading,
      wgLoading: s.wgLoading,
      mxEntryError: s.mxEntryError,
      mxExitError: s.mxExitError,
      wgError: s.wgError,
    })),
  );
export const useFetchGateways = () => useAppStore((s) => s.fetchGateways);

export const useFetchRecents = () => useAppStore((s) => s.fetchRecents);

export const useLookupGw = () => useAppStore((s) => s.lookupGw);

// socks5 state
export const useSocks5 = () =>
  useAppStore(
    useShallow((s) => ({
      status: s.status,
      isLoading: s.isLoading,
      enable: s.enable,
      disable: s.disable,
      refresh: s.refresh,
    })),
  );
