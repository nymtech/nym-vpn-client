import i18n from 'i18next';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { isEnabled as isAutostartEnabled } from '@tauri-apps/plugin-autostart';
import {
  DefaultCountry,
  DefaultRootFontSize,
  DefaultThemeMode,
  DefaultVpnMode,
  GatewaysCacheDuration,
} from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  AccountLinks,
  CodeDependency,
  Country,
  Gateway,
  GatewaysByCountry,
  NodeHop,
  StateDispatch,
  ThemeMode,
  TunnelStateIpc,
  UiTheme,
  VpnMode,
  VpndStatus,
} from '../types';
import { S_STATE } from '../static';
import { Notification } from '../contexts';
import { CCache } from '../cache';
import { tunnelUpdate } from './tunnelUpdate';
import { TauriReq, daemonStatusUpdate, fireRequests } from './helper';

// initialize connection state
const getInitialTunnelState = async () => {
  return await invoke<TunnelStateIpc>('get_tunnel_state');
};

const getDaemonStatus = async () => {
  return await invoke<VpndStatus>('daemon_status');
};

// init gateway list
const getMxGateways = async (node: NodeHop) => {
  let gateways = await CCache.get<GatewaysByCountry[]>(
    `cache-mx-${node}-gateways`,
  );
  if (!gateways) {
    gateways = await invoke<GatewaysByCountry[] | null>('get_gateways', {
      nodeType: node === 'entry' ? 'mx-entry' : 'mx-exit',
    });
    await CCache.set(
      `cache-mx-${node}-gateways`,
      gateways || [],
      GatewaysCacheDuration,
    );
  }
  return gateways;
};

const getWgGateways = async () => {
  let gateways = await CCache.get<GatewaysByCountry[]>(`cache-wg-gateways`);
  if (!gateways) {
    gateways = await invoke<GatewaysByCountry[] | null>('get_gateways', {
      nodeType: 'wg',
    });
    await CCache.set(
      'cache-wg-gateways',
      gateways || [],
      GatewaysCacheDuration,
    );
  }
  return gateways;
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await getCurrentWebviewWindow().theme()) === 'dark' ? 'Dark' : 'Light';
  const themeMode = await kvGet<ThemeMode>('ui-theme');
  return { winTheme, themeMode };
};

export async function initFirstBatch(
  dispatch: StateDispatch,
  push: (notification: Notification) => void,
) {
  const initStateRq: TauriReq<typeof getInitialTunnelState> = {
    name: 'get_tunnel_state',
    request: () => getInitialTunnelState(),
    onFulfilled: (state) => {
      tunnelUpdate(state, dispatch);
    },
  };

  const initDaemonStatusRq: TauriReq<() => Promise<VpndStatus>> = {
    name: 'daemon_status',
    request: () => getDaemonStatus(),
    onFulfilled: (status) => {
      daemonStatusUpdate(status, dispatch, push);
    },
  };

  const getEntryNodeRq: TauriReq<() => Promise<Gateway | Country | undefined>> =
    {
      name: 'getEntryNode',
      request: () => kvGet<Gateway | Country>('entry-node'),
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
          console.info(
            'no entry node saved, using default country',
            DefaultCountry,
          );
        }
      },
    };

  const getExitNodeRq: TauriReq<() => Promise<Gateway | Country | undefined>> =
    {
      name: 'getExitNode',
      request: () => kvGet<Gateway | Country>('exit-node'),
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
          console.info(
            'no exit node saved, using default country',
            DefaultCountry,
          );
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
      let uiTheme: UiTheme = 'Light';
      if (themeMode === 'System') {
        uiTheme = winTheme;
      } else {
        // if no theme has been saved, fallback to system theme
        uiTheme = themeMode || winTheme;
      }
      dispatch({ type: 'set-ui-theme', theme: uiTheme });
      dispatch({ type: 'set-theme-mode', mode: themeMode || DefaultThemeMode });
    },
  };

  const getVpnModeRq: TauriReq<() => Promise<VpnMode | undefined>> = {
    name: 'getVpnMode',
    request: () => kvGet<VpnMode>('vpn-mode'),
    onFulfilled: (vpnMode) => {
      S_STATE.vpnModeInit = true;
      dispatch({ type: 'set-vpn-mode', mode: vpnMode || DefaultVpnMode });
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
    request: () => kvGet<boolean>('monitoring'),
    onFulfilled: (monitoring) => {
      dispatch({ type: 'set-monitoring', monitoring: monitoring || false });
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

  // fire all requests concurrently
  await fireRequests([
    initStateRq,
    initDaemonStatusRq,
    getVpnModeRq,
    getEntryNodeRq,
    getExitNodeRq,
    getVersionRq,
    getThemeRq,
    getStoredAccountRq,
    getRootFontSizeRq,
    getMonitoringRq,
    getDepsRustRq,
    getDepsJsRq,
    getDesktopNotificationsRq,
  ]);
}

export async function initSecondBatch(dispatch: StateDispatch) {
  const getMxEntryGatewaysRq: TauriReq<typeof getMxGateways> = {
    name: 'get_mx_entry_gateways',
    request: () => getMxGateways('entry'),
    onFulfilled: (gateways) => {
      if (!gateways) return;
      dispatch({
        type: 'set-gateways',
        payload: {
          type: 'mx-entry',
          gateways: gateways || [],
        },
      });
      dispatch({
        type: 'set-gateways-loading',
        payload: { type: 'mx-entry', loading: false },
      });
    },
  };

  const getMxExitGatewaysRq: TauriReq<typeof getMxGateways> = {
    name: 'get_mx_exit_gateways',
    request: () => getMxGateways('exit'),
    onFulfilled: (gateways) => {
      dispatch({
        type: 'set-gateways',
        payload: {
          type: 'mx-exit',
          gateways: gateways || [],
        },
      });
      dispatch({
        type: 'set-gateways-loading',
        payload: { type: 'mx-exit', loading: false },
      });
    },
  };

  const getWgGatewaysRq: TauriReq<typeof getWgGateways> = {
    name: 'get_wg_gateways',
    request: () => getWgGateways(),
    onFulfilled: (gateways) => {
      dispatch({
        type: 'set-gateways',
        payload: {
          type: 'wg',
          gateways: gateways || [],
        },
      });
      dispatch({
        type: 'set-gateways-loading',
        payload: { type: 'wg', loading: false },
      });
    },
  };

  const getAccountLinksRq: TauriReq<() => Promise<AccountLinks | undefined>> = {
    name: 'getAccountLinksRq',
    request: () =>
      invoke<AccountLinks>('account_links', { locale: i18n.language }),
    onFulfilled: (links) => {
      dispatch({
        type: 'set-account-links',
        links: links as AccountLinks | null,
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

  let gatewayRequests;
  if (S_STATE.vpnModeAtStart === 'wg') {
    gatewayRequests = [getWgGatewaysRq];
  } else {
    gatewayRequests = [getMxEntryGatewaysRq, getMxExitGatewaysRq];
  }

  await fireRequests([...gatewayRequests, getAccountLinksRq, getAutostart]);
}
