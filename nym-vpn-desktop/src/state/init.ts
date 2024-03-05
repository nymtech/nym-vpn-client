import { invoke } from '@tauri-apps/api';
import { getVersion } from '@tauri-apps/api/app';
import { appWindow } from '@tauri-apps/api/window';
import { DefaultRootFontSize, DefaultThemeMode } from '../constants';
import {
  AppDataFromBackend,
  ConnectionState,
  Country,
  NodeLocationBackend,
  StateDispatch,
  UiTheme,
} from '../types';
import fireRequests, { TauriReq } from './helper';
import { initialState } from './main';

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

// get saved on disk app data and restore state from it
const getAppData = async () => {
  const theme = await appWindow.theme();
  const winTheme: UiTheme = theme === 'dark' ? 'Dark' : 'Light';
  const appData = await invoke<AppDataFromBackend>('db_get_batch');
  return { winTheme, data: appData };
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

  const getSavedAppDataRq: TauriReq<typeof getAppData> = {
    name: 'get_app_data',
    request: () => getAppData(),
    onFulfilled: ({ winTheme, data }) => {
      console.log('app data read from disk:');
      console.log(data);

      if (data.ui_root_font_size) {
        document.documentElement.style.fontSize = `${data.ui_root_font_size}px`;
      }

      let uiTheme: UiTheme = 'Light';
      if (data.ui_theme === 'System') {
        uiTheme = winTheme;
      } else {
        // if no theme has been saved, fallback to system theme
        uiTheme = data.ui_theme || winTheme;
      }

      const partialState: Partial<typeof initialState> = {
        entrySelector: data.entry_location_enabled || false,
        uiTheme,
        themeMode: data.ui_theme || DefaultThemeMode,
        vpnMode: data.vpn_mode || 'TwoHop',
        autoConnect: data.autoconnect || false,
        monitoring: data.monitoring || false,
        rootFontSize: data.ui_root_font_size || DefaultRootFontSize,
      };
      dispatch({
        type: 'set-partial-state',
        partialState,
      });
    },
  };

  // fire all requests concurrently
  await fireRequests([
    initStateRq,
    syncConTimeRq,
    getEntryCountriesRq,
    getExitCountriesRq,
    getEntryLocationRq,
    getExitLocationRq,
    getFastestLocationRq,
    getVersionRq,
    getSavedAppDataRq,
  ]);
}

export default init;
