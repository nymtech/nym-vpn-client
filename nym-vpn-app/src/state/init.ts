import i18n from 'i18next';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
  CountryCacheDuration,
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
  StateDispatch,
  ThemeMode,
  TunnelStateIpc,
  UiTheme,
  VpnMode,
  VpndStatus,
} from '../types';
import { S_STATE } from '../static';
import { MCache } from '../cache';
import { Notification } from '../contexts';
import { tunnelUpdate } from './tunnelUpdate';
import { TauriReq, daemonStatusUpdate, fireRequests } from './helper';

// initialize connection state
const getInitialTunnelState = async () => {
  return await invoke<TunnelStateIpc>('get_tunnel_state');
};

const getDaemonStatus = async () => {
  return await invoke<VpndStatus>('daemon_status');
};

// init country list
const getEntryCountries = async () => {
  const mode = (await kvGet<VpnMode>('VpnMode')) || DefaultVpnMode;
  const countries = await invoke<Country[]>('get_countries', {
    vpnMode: mode,
    nodeType: 'Entry',
  });
  return { countries, mode };
};
const getExitCountries = async () => {
  const mode = (await kvGet<VpnMode>('VpnMode')) || DefaultVpnMode;
  const countries = await invoke<Country[]>('get_countries', {
    vpnMode: mode,
    nodeType: 'Exit',
  });
  return { countries, mode };
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await getCurrentWebviewWindow().theme()) === 'dark' ? 'Dark' : 'Light';
  const themeMode = await kvGet<ThemeMode>('UiTheme');
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

  const getEntryLocationRq: TauriReq<() => Promise<Country | undefined>> = {
    name: 'getEntryLocation',
    request: () => kvGet<Country>('EntryNodeLocation'),
    onFulfilled: (location) => {
      if (location) {
        dispatch({
          type: 'set-node-location',
          payload: {
            hop: 'entry',
            location,
          },
        });
      } else {
        console.info('no entry country saved, using default', DefaultCountry);
      }
    },
  };

  const getExitLocationRq: TauriReq<() => Promise<Country | undefined>> = {
    name: 'getExitLocation',
    request: () => kvGet<Country>('ExitNodeLocation'),
    onFulfilled: (location) => {
      if (location) {
        dispatch({
          type: 'set-node-location',
          payload: {
            hop: 'exit',
            location,
          },
        });
      } else {
        console.info('no exit country saved, using default', DefaultCountry);
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
    request: () => kvGet<VpnMode>('VpnMode'),
    onFulfilled: (vpnMode) => {
      S_STATE.vpnModeInit = true;
      dispatch({ type: 'set-vpn-mode', mode: vpnMode || DefaultVpnMode });
    },
  };

  const getDesktopNotificationsRq: TauriReq<
    () => Promise<boolean | undefined>
  > = {
    name: 'getDesktopNotificationsRq',
    request: () => kvGet<boolean>('DesktopNotifications'),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-desktop-notifications',
        enabled: enabled || false,
      });
    },
  };

  const getRootFontSizeRq: TauriReq<() => Promise<number | undefined>> = {
    name: 'getRootFontSize',
    request: () => kvGet<number>('UiRootFontSize'),
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
    request: () => kvGet<boolean>('Monitoring'),
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
    getEntryLocationRq,
    getExitLocationRq,
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
  const getEntryCountriesRq: TauriReq<typeof getEntryCountries> = {
    name: 'get_countries',
    request: () => getEntryCountries(),
    onFulfilled: ({ countries, mode }) => {
      dispatch({
        type: 'set-country-list',
        payload: {
          hop: 'entry',
          countries,
        },
      });
      MCache.set(
        mode === 'Mixnet' ? `mn-entry-countries` : 'wg-countries',
        countries,
        CountryCacheDuration,
      );
      dispatch({
        type: 'set-countries-loading',
        payload: { hop: 'entry', loading: false },
      });
    },
  };

  const getExitCountriesRq: TauriReq<typeof getExitCountries> = {
    name: 'get_countries',
    request: () => getExitCountries(),
    onFulfilled: ({ countries, mode }) => {
      dispatch({
        type: 'set-country-list',
        payload: {
          hop: 'exit',
          countries,
        },
      });
      MCache.set(
        mode === 'Mixnet' ? `mn-exit-countries` : 'wg-countries',
        countries,
        CountryCacheDuration,
      );
      dispatch({
        type: 'set-countries-loading',
        payload: { hop: 'exit', loading: false },
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

  await fireRequests([
    getEntryCountriesRq,
    getExitCountriesRq,
    getAccountLinksRq,
  ]);
}
