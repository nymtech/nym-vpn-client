import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
  DefaultRootFontSize,
  DefaultThemeMode,
  DefaultVpnMode,
} from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  CodeDependency,
  ConnectionStateResponse,
  Country,
  DaemonInfo,
  DaemonStatus,
  NodeLocation,
  StateDispatch,
  ThemeMode,
  UiTheme,
  VpnMode,
} from '../types';
import fireRequests, { TauriReq } from './helper';
import { S_STATE } from '../static';

// initialize connection state
const getInitialConnectionState = async () => {
  return await invoke<ConnectionStateResponse>('get_connection_state');
};

const getDaemonStatus = async () => {
  return await invoke<DaemonStatus>('daemon_status');
};

const getDaemonInfo = async () => {
  return await invoke<DaemonInfo>('daemon_info');
};

// initialize session start time
const getSessionStartTime = async () => {
  return await invoke<number | undefined>('get_connection_start_time');
};

// init country list
const getEntryCountries = async () => {
  const mode = await kvGet<VpnMode>('VpnMode');
  return await invoke<Country[]>('get_countries', {
    vpnMode: mode || DefaultVpnMode,
    nodeType: 'Entry',
  });
};
const getExitCountries = async () => {
  const mode = await kvGet<VpnMode>('VpnMode');
  return await invoke<Country[]>('get_countries', {
    vpnMode: mode || DefaultVpnMode,
    nodeType: 'Exit',
  });
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await getCurrentWebviewWindow().theme()) === 'dark' ? 'Dark' : 'Light';
  const themeMode = await kvGet<ThemeMode>('UiTheme');
  return { winTheme, themeMode };
};

export async function initFirstBatch(dispatch: StateDispatch) {
  const initStateRq: TauriReq<typeof getInitialConnectionState> = {
    name: 'get_connection_state',
    request: () => getInitialConnectionState(),
    onFulfilled: ({ state, error }) => {
      dispatch({ type: 'change-connection-state', state });
      if (error) {
        dispatch({ type: 'set-error', error });
      }
    },
  };

  const initDaemonStatusRq: TauriReq<() => Promise<DaemonStatus>> = {
    name: 'daemon_status',
    request: () => getDaemonStatus(),
    onFulfilled: (status) => {
      dispatch({ type: 'set-daemon-status', status });
    },
  };

  const initDaemonInfoRq: TauriReq<() => Promise<DaemonInfo>> = {
    name: 'daemon_status',
    request: () => getDaemonInfo(),
    onFulfilled: (info) => {
      dispatch({ type: 'set-daemon-info', info });
    },
  };

  const syncConTimeRq: TauriReq<typeof getSessionStartTime> = {
    name: 'get_connection_start_time',
    request: () => getSessionStartTime(),
    onFulfilled: (startTime) => {
      dispatch({ type: 'set-connection-start-time', startTime });
    },
  };

  const getEntryLocationRq: TauriReq<() => Promise<NodeLocation | undefined>> =
    {
      name: 'getEntryLocation',
      request: () => kvGet<NodeLocation>('EntryNodeLocation'),
      onFulfilled: (location) => {
        if (location) {
          dispatch({
            type: 'set-node-location',
            payload: {
              hop: 'entry',
              location: location === 'Fastest' ? 'Fastest' : location,
            },
          });
        }
      },
    };

  const getExitLocationRq: TauriReq<() => Promise<NodeLocation | undefined>> = {
    name: 'getExitLocation',
    request: () => kvGet<NodeLocation>('ExitNodeLocation'),
    onFulfilled: (location) => {
      if (location) {
        dispatch({
          type: 'set-node-location',
          payload: {
            hop: 'exit',
            location: location === 'Fastest' ? 'Fastest' : location,
          },
        });
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

  const getEntrySelectorRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getEntrySelector',
    request: () => kvGet<boolean>('UiShowEntrySelect'),
    onFulfilled: (enabled) => {
      dispatch({ type: 'set-entry-selector', entrySelector: enabled || false });
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
    initDaemonInfoRq,
    getVpnModeRq,
    syncConTimeRq,
    getEntryLocationRq,
    getExitLocationRq,
    getVersionRq,
    getThemeRq,
    getStoredAccountRq,
    getRootFontSizeRq,
    getEntrySelectorRq,
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
    onFulfilled: (countries) => {
      dispatch({
        type: 'set-country-list',
        payload: {
          hop: 'entry',
          countries,
        },
      });
      dispatch({
        type: 'set-countries-loading',
        payload: { hop: 'entry', loading: false },
      });
    },
  };

  const getExitCountriesRq: TauriReq<typeof getExitCountries> = {
    name: 'get_countries',
    request: () => getExitCountries(),
    onFulfilled: (countries) => {
      dispatch({
        type: 'set-country-list',
        payload: {
          hop: 'exit',
          countries,
        },
      });
      dispatch({
        type: 'set-countries-loading',
        payload: { hop: 'exit', loading: false },
      });
    },
  };

  await fireRequests([getEntryCountriesRq, getExitCountriesRq]);
}
