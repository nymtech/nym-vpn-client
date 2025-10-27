import i18n from 'i18next';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { isEnabled as isAutostartEnabled } from '@tauri-apps/plugin-autostart';
import {
  DefaultNode,
  DefaultRootFontSize,
  DefaultThemeMode,
} from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  AccountLinks,
  CodeDependency,
  FeatureFlags,
  InitState,
  NetworkCompat,
  SelectedNode,
  StateDispatch,
  TAccountState,
  TTunnelState,
  ThemeMode,
  UiTheme,
} from '../types';
import { updateAccountState, updateTunnel } from './update';
import { TauriReq, fireRequests } from './helper';

const defaultQuic = window._APP.defaultQuic;
const defaultDomFront = window._APP.defaultDomainFronting;

// initialize connection state
const getInitialTunnelState = async () => {
  return await invoke<TTunnelState>('get_tunnel_state');
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await getCurrentWebviewWindow().theme()) === 'dark' ? 'dark' : 'light';
  const themeMode = await kvGet<ThemeMode>('ui-theme');
  return { winTheme, themeMode };
};

export async function initFirstBatch(
  dispatch: StateDispatch,
  initState: InitState,
) {
  const initStateRq: TauriReq<typeof getInitialTunnelState> = {
    name: 'get_tunnel_state',
    request: () => getInitialTunnelState(),
    onFulfilled: (state) => {
      updateTunnel(state, dispatch);
    },
  };

  const getEntryNodeRq: TauriReq<() => Promise<SelectedNode | undefined>> = {
    name: 'getEntryNode',
    request: () => kvGet<SelectedNode>('entry-node'),
    onFulfilled: (node) => {
      if (node) {
        dispatch({
          type: 'set-node',
          payload: {
            hop: 'entry',
            node,
          },
        });
      } else {
        console.info('no entry node saved, using default country', DefaultNode);
      }
    },
  };

  const getExitNodeRq: TauriReq<() => Promise<SelectedNode | undefined>> = {
    name: 'getExitNode',
    request: () => kvGet<SelectedNode>('exit-node'),
    onFulfilled: (node) => {
      if (node) {
        dispatch({
          type: 'set-node',
          payload: {
            hop: 'exit',
            node,
          },
        });
      } else {
        console.info('no exit node saved, using default country', DefaultNode);
      }
    },
  };

  const getAccountStateRq: TauriReq<() => Promise<TAccountState | undefined>> =
    {
      name: 'getAccountStateRq',
      request: () => invoke<TAccountState>('get_account_state'),
      onFulfilled: (state) => {
        if (state) {
          updateAccountState(state, dispatch);
        }
      },
    };

  const getStoredAccountRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getStoredAccountRq',
    request: () => invoke<boolean>('is_account_stored'),
    onFulfilled: (stored) => {
      dispatch({
        type: 'set-account',
        stored: stored || false,
      });
    },
  };

  const getFeatureFlagsRq: TauriReq<() => Promise<FeatureFlags | undefined>> = {
    name: 'getFeatureFlagsRq',
    request: () => invoke<FeatureFlags>('feature_flags'),
    onFulfilled: (flags) => {
      if (flags) {
        dispatch({
          type: 'set-backend-flags',
          flags,
        });
      }
    },
  };

  const getVersionRq: TauriReq<typeof getVersion> = {
    name: 'getVersion',
    request: () => getVersion(),
    onFulfilled: (version) => {
      dispatch({ type: 'set-version', version });
    },
  };

  const getThemeRq: TauriReq<typeof getTheme> = {
    name: 'getTheme',
    request: () => getTheme(),
    onFulfilled: ({ winTheme, themeMode }) => {
      let uiTheme: UiTheme = 'light';
      if (themeMode === 'system') {
        uiTheme = winTheme;
      } else {
        // if no theme has been saved, fallback to system theme
        uiTheme = themeMode || winTheme;
      }
      dispatch({ type: 'set-ui-theme', theme: uiTheme });
      dispatch({ type: 'set-theme-mode', mode: themeMode || DefaultThemeMode });
    },
  };

  const getDesktopNotificationsRq: TauriReq<
    () => Promise<boolean | undefined>
  > = {
    name: 'getDesktopNotificationsRq',
    request: () => kvGet<boolean>('desktop-notifications'),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-desktop-notifications',
        enabled: enabled || false,
      });
    },
  };

  const getRootFontSizeRq: TauriReq<() => Promise<number | undefined>> = {
    name: 'getRootFontSize',
    request: () => kvGet<number>('ui-root-font-size'),
    onFulfilled: (size) => {
      // if a font size was saved, set the UI font size accordingly
      if (size) {
        document.documentElement.style.fontSize = `${size}px`;
      }
      dispatch({
        type: 'set-root-font-size',
        size: size || DefaultRootFontSize,
      });
    },
  };

  const getMonitoringRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getMonitoring',
    request: () => invoke<boolean>('sentry_enabled'),
    onFulfilled: (enabled) => {
      dispatch({ type: 'set-monitoring', enabled: enabled || false });
    },
  };

  const getDepsRustRq: TauriReq<() => Promise<CodeDependency[] | undefined>> = {
    name: 'getDepsRustRq',
    request: () => getRustLicenses(),
    onFulfilled: (dependencies) => {
      dispatch({
        type: 'set-code-deps-rust',
        dependencies: dependencies || [],
      });
    },
  };

  const getDepsJsRq: TauriReq<() => Promise<CodeDependency[] | undefined>> = {
    name: 'getDepsJsRq',
    request: () => getJsLicenses(),
    onFulfilled: (dependencies) => {
      dispatch({
        type: 'set-code-deps-js',
        dependencies: dependencies || [],
      });
    },
  };

  const getIpv6SupportRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getIpv6Support',
    request: () => kvGet<boolean>('disable-ipv6'),
    onFulfilled: (disabled) => {
      if (disabled) {
        dispatch({ type: 'set-ipv6-support', enabled: false });
      }
    },
  };

  const getQuicRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getQuicRq',
    request: () => kvGet<boolean>('quic-enabled'),
    onFulfilled: (enabled) => {
      dispatch({ type: 'set-quic', enabled: enabled || defaultQuic });
    },
  };

  const getDomainFrontingRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getDomainFrontingRq',
    request: () => kvGet<boolean>('domain-fronting-enabled'),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-domain-fronting',
        enabled: enabled || defaultDomFront,
      });
    },
  };

  const getNetworkStatsRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getNetworkStats',
    request: () => kvGet<boolean>('network-stats-enabled'),
    onFulfilled: (enabled) => {
      if (enabled !== undefined) {
        dispatch({ type: 'set-network-stats', enabled });
      }
    },
  };

  let requests: TauriReq<never>[] = [
    getEntryNodeRq,
    getExitNodeRq,
    getVersionRq,
    getThemeRq,
    getRootFontSizeRq,
    getMonitoringRq,
    getDepsRustRq,
    getDepsJsRq,
    getDesktopNotificationsRq,
    getIpv6SupportRq,
    getNetworkStatsRq,
    getQuicRq,
    getDomainFrontingRq,
  ];

  if (initState.vpnd !== 'down') {
    requests = [
      initStateRq,
      getStoredAccountRq,
      getAccountStateRq,
      getFeatureFlagsRq,
      ...requests,
    ];
  }

  // fire all requests concurrently
  await fireRequests(requests);
}

export async function initSecondBatch(
  dispatch: StateDispatch,
  initState: InitState,
) {
  const getAccountLinksRq: TauriReq<() => Promise<AccountLinks | undefined>> = {
    name: 'getAccountLinksRq',
    request: () =>
      invoke<AccountLinks>('account_links', { locale: i18n.language }),
    onFulfilled: (links) => {
      dispatch({
        type: 'set-account-links',
        links: links || null,
      });
    },
  };

  const getAutostart: TauriReq<() => Promise<boolean>> = {
    name: 'getAutostart',
    request: () => isAutostartEnabled(),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-autostart',
        enabled,
      });
    },
  };

  const getNetworkCompatRq: TauriReq<() => Promise<NetworkCompat | undefined>> =
    {
      name: 'getNetworkCompatRq',
      request: () => invoke<NetworkCompat>('network_compat'),
      onFulfilled: (compat) => {
        dispatch({
          type: 'set-network-compat',
          compat: compat || null,
        });
      },
    };

  let requests: TauriReq<never>[] = [getAutostart];
  if (initState.vpnd !== 'down') {
    requests = [getAccountLinksRq, getNetworkCompatRq, ...requests];
  }

  await fireRequests(requests);
}
