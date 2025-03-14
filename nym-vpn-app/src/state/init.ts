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
} from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  AccountLinks,
  CodeDependency,
  Country,
  Gateway,
  NetworkCompat,
  StateDispatch,
  ThemeMode,
  TunnelStateIpc,
  UiTheme,
  VpnMode,
} from '../types';
import { S_STATE } from '../static';
import { tunnelUpdate } from './tunnelUpdate';
import { TauriReq, fireRequests } from './helper';

// initialize connection state
const getInitialTunnelState = async () => {
  return await invoke<TunnelStateIpc>('get_tunnel_state');
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await getCurrentWebviewWindow().theme()) === 'dark' ? 'Dark' : 'Light';
  const themeMode = await kvGet<ThemeMode>('ui-theme');
  return { winTheme, themeMode };
};

export async function initFirstBatch(dispatch: StateDispatch) {
  const initStateRq: TauriReq<typeof getInitialTunnelState> = {
    name: 'get_tunnel_state',
    request: () => getInitialTunnelState(),
    onFulfilled: (state) => {
      tunnelUpdate(state, dispatch);
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

  let requests: TauriReq<never>[] = [
    getVpnModeRq,
    getEntryNodeRq,
    getExitNodeRq,
    getVersionRq,
    getThemeRq,
    getRootFontSizeRq,
    getMonitoringRq,
    getDepsRustRq,
    getDepsJsRq,
    getDesktopNotificationsRq,
  ];

  if (S_STATE.vpnd !== 'down') {
    requests = [initStateRq, getStoredAccountRq, ...requests];
  }

  // fire all requests concurrently
  await fireRequests(requests);
}

export async function initSecondBatch(dispatch: StateDispatch) {
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
  if (S_STATE.vpnd !== 'down') {
    requests = [getAccountLinksRq, getNetworkCompatRq, ...requests];
  }

  await fireRequests(requests);
}
