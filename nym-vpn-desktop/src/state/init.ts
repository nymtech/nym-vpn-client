import { invoke } from '@tauri-apps/api';
import { getVersion } from '@tauri-apps/api/app';
import { appWindow } from '@tauri-apps/api/window';
import { DefaultRootFontSize, DefaultThemeMode } from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  CodeDependency,
  ConnectionState,
  Country,
  NodeLocationBackend,
  StateDispatch,
  ThemeMode,
  UiTheme,
  VpnMode,
  WindowSize,
} from '../types';
import fireRequests, { TauriReq } from './helper';

// initialize connection state
const getInitialConnectionState = async () => {
  return await invoke<ConnectionState>('get_connection_state');
};

// initialize session start time
const getSessionStartTime = async () => {
  return await invoke<number | undefined>('get_connection_start_time');
};

// init country list
const getEntryCountries = async () => {
  return await invoke<Country[]>('get_countries', {
    nodeType: 'Entry',
  });
};
const getExitCountries = async () => {
  return await invoke<Country[]>('get_countries', {
    nodeType: 'Exit',
  });
};

// init node locations
const getEntryNodeLocation = async () => {
  return await invoke<NodeLocationBackend>('get_node_location', {
    nodeType: 'Entry',
  });
};
const getExitNodeLocation = async () => {
  return await invoke<NodeLocationBackend>('get_node_location', {
    nodeType: 'Exit',
  });
};

// init fastest node location
const getFastestNodeLocation = async () => {
  return await invoke<Country>('get_fastest_node_location');
};

const getTheme = async () => {
  const winTheme: UiTheme =
    (await appWindow.theme()) === 'dark' ? 'Dark' : 'Light';
  const themeMode = await kvGet<ThemeMode>('UiTheme');
  return { winTheme, themeMode };
};

async function init(dispatch: StateDispatch) {
  const initStateRq: TauriReq<typeof getInitialConnectionState> = {
    name: 'get_connection_state',
    request: () => getInitialConnectionState(),
    onFulfilled: (value) => {
      dispatch({ type: 'change-connection-state', state: value });
    },
  };

  const syncConTimeRq: TauriReq<typeof getSessionStartTime> = {
    name: 'get_connection_start_time',
    request: () => getSessionStartTime(),
    onFulfilled: (startTime) => {
      dispatch({ type: 'set-connection-start-time', startTime });
    },
  };

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
    },
  };

  const getEntryLocationRq: TauriReq<typeof getEntryNodeLocation> = {
    name: 'get_node_location',
    request: () => getEntryNodeLocation(),
    onFulfilled: (location) => {
      dispatch({
        type: 'set-node-location',
        payload: {
          hop: 'entry',
          location: location === 'Fastest' ? 'Fastest' : location.Country,
        },
      });
    },
  };

  const getExitLocationRq: TauriReq<typeof getExitNodeLocation> = {
    name: 'get_node_location',
    request: () => getExitNodeLocation(),
    onFulfilled: (location) => {
      dispatch({
        type: 'set-node-location',
        payload: {
          hop: 'exit',
          location: location === 'Fastest' ? 'Fastest' : location.Country,
        },
      });
    },
  };

  const getFastestLocationRq: TauriReq<typeof getFastestNodeLocation> = {
    name: 'get_fastest_node_location',
    request: () => getFastestNodeLocation(),
    onFulfilled: (country) => {
      dispatch({ type: 'set-fastest-node-location', country });
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
      dispatch({ type: 'set-vpn-mode', mode: vpnMode || 'TwoHop' });
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
    request: () => kvGet<boolean>('EntryLocationEnabled'),
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

  const getWindowSizeRq: TauriReq<() => Promise<WindowSize | undefined>> = {
    name: 'getWindowSize',
    request: () => kvGet<WindowSize>('WindowSize'),
    onFulfilled: (size) => {
      if (size) {
        dispatch({ type: 'set-window-size', size });
      }
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
    getVpnModeRq,
    syncConTimeRq,
    getEntryCountriesRq,
    getExitCountriesRq,
    getEntryLocationRq,
    getExitLocationRq,
    getFastestLocationRq,
    getVersionRq,
    getThemeRq,
    getRootFontSizeRq,
    getEntrySelectorRq,
    getMonitoringRq,
    getDepsRustRq,
    getDepsJsRq,
    getWindowSizeRq,
  ]);
}

export default init;
