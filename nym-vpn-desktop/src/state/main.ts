import dayjs from 'dayjs';
import {
  DefaultNodeCountry,
  DefaultRootFontSize,
  DefaultThemeMode,
} from '../constants';
import {
  AppState,
  ConnectProgressMsg,
  ConnectionState,
  Country,
  NodeHop,
  NodeLocation,
  ThemeMode,
  UiTheme,
  VpnMode,
} from '../types';

export type StateAction =
  | { type: 'init-done' }
  | { type: 'set-partial-state'; partialState: Partial<AppState> }
  | { type: 'change-connection-state'; state: ConnectionState }
  | { type: 'set-vpn-mode'; mode: VpnMode }
  | { type: 'set-entry-selector'; entrySelector: boolean }
  | { type: 'set-error'; error: string }
  | { type: 'reset-error' }
  | { type: 'new-progress-message'; message: ConnectProgressMsg }
  | { type: 'connect' }
  | { type: 'disconnect' }
  | { type: 'set-version'; version: string }
  | { type: 'set-connected'; startTime: number }
  | { type: 'set-connection-start-time'; startTime?: number | null }
  | { type: 'set-disconnected' }
  | { type: 'set-auto-connect'; autoConnect: boolean }
  | { type: 'set-monitoring'; monitoring: boolean }
  | { type: 'reset' }
  | { type: 'set-ui-theme'; theme: UiTheme }
  | { type: 'set-theme-mode'; mode: ThemeMode }
  | { type: 'system-theme-changed'; theme: UiTheme }
  | { type: 'set-country-list'; countries: Country[] }
  | {
      type: 'set-node-location';
      payload: { hop: NodeHop; location: NodeLocation };
    }
  | { type: 'set-fastest-node-location'; country: Country }
  | { type: 'set-root-font-size'; size: number };

export const initialState: AppState = {
  initialized: false,
  state: 'Disconnected',
  version: null,
  loading: false,
  vpnMode: 'TwoHop',
  entrySelector: false,
  tunnel: { name: 'nym', id: 'nym' },
  uiTheme: 'Light',
  themeMode: DefaultThemeMode,
  progressMessages: [],
  autoConnect: false,
  monitoring: false,
  // TODO ⚠ these should be set to 'Fastest' when the backend is ready
  entryNodeLocation: DefaultNodeCountry,
  // TODO ⚠ these should be set to 'Fastest' when the backend is ready
  exitNodeLocation: DefaultNodeCountry,
  fastestNodeLocation: DefaultNodeCountry,
  countryList: [],
  rootFontSize: DefaultRootFontSize,
};

export function reducer(state: AppState, action: StateAction): AppState {
  switch (action.type) {
    case 'init-done':
      return {
        ...state,
        initialized: true,
      };
    case 'set-node-location':
      if (action.payload.hop === 'entry') {
        return {
          ...state,
          entryNodeLocation: action.payload.location,
        };
      }
      return {
        ...state,
        exitNodeLocation: action.payload.location,
      };
    case 'set-vpn-mode':
      return {
        ...state,
        vpnMode: action.mode,
      };
    case 'set-entry-selector':
      return {
        ...state,
        entrySelector: action.entrySelector,
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
    case 'set-country-list':
      return {
        ...state,
        countryList: action.countries,
      };
    case 'set-partial-state': {
      return { ...state, ...action.partialState };
    }
    case 'change-connection-state': {
      console.log(
        `__REDUCER [change-connection-state] changing connection state to ${action.state}`,
      );
      if (action.state === state.state) {
        return state;
      }
      return {
        ...state,
        state: action.state,
        loading:
          action.state === 'Connecting' || action.state === 'Disconnecting',
      };
    }
    case 'connect': {
      console.log(
        `__REDUCER [connect] changing connection state to Connecting`,
      );
      return { ...state, state: 'Connecting', loading: true };
    }
    case 'disconnect': {
      return { ...state, state: 'Disconnecting', loading: true };
    }
    case 'set-version':
      return {
        ...state,
        version: action.version,
      };
    case 'set-connected': {
      console.log(
        `__REDUCER [set-connected] changing connection state to Connected`,
      );
      return {
        ...state,
        state: 'Connected',
        loading: false,
        progressMessages: [],
        sessionStartDate: dayjs.unix(action.startTime),
      };
    }
    case 'set-disconnected': {
      return {
        ...state,
        state: 'Disconnected',
        loading: false,
        progressMessages: [],
        sessionStartDate: null,
      };
    }
    case 'set-connection-start-time':
      return {
        ...state,
        sessionStartDate:
          (action.startTime && dayjs.unix(action.startTime)) || null,
      };
    case 'set-error':
      return { ...state, error: action.error };
    case 'reset-error':
      return { ...state, error: null };
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
    case 'set-fastest-node-location':
      return {
        ...state,
        fastestNodeLocation: action.country,
      };
    case 'set-root-font-size':
      return {
        ...state,
        rootFontSize: action.size,
      };

    case 'reset':
      return initialState;
  }
}
