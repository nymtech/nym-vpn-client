import i18n from 'i18next';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { isEnabled as isAutostartEnabled } from '@tauri-apps/plugin-autostart';
import { DefaultRootFontSize, DefaultThemeMode } from '../constants';
import { getJsLicenses, getRustLicenses } from '../data';
import { kvGet } from '../kvStore';
import {
  AccountLinks,
  CodeDependency,
  Favorites,
  FeatureFlags,
  NetworkCompat,
  TAccountMode,
  TAccountState,
  TAccountSummary,
  TTunnelState,
  ThemeMode,
  UiTheme,
} from '../types';
import { dispatch } from '../store';
import { useFavoritesStore } from '../store/favoritesState';
import { updateAccountState, updateTunnel } from './update';
import { TauriReq, fireRequests } from './helper';

const defaultNetStats = window._APP.defaultNetstats;

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

export async function initFirstBatch() {
  const initStateRq: TauriReq<typeof getInitialTunnelState> = {
    name: 'get_tunnel_state',
    request: () => getInitialTunnelState(),
    onFulfilled: (state) => {
      updateTunnel(state);
    },
  };

  const getAccountStateRq: TauriReq<() => Promise<TAccountState | undefined>> =
    {
      name: 'getAccountStateRq',
      request: () => invoke<TAccountState>('get_account_state'),
      onFulfilled: (state) => {
        if (state) {
          updateAccountState(state);
        }
      },
    };

  const getAccountModeRq: TauriReq<() => Promise<TAccountMode | undefined>> = {
    name: 'getAccountModeRq',
    request: () => invoke<TAccountMode>('get_account_mode'),
    onFulfilled: (mode) => {
      if (mode) {
        dispatch({ type: 'set-account-mode', mode });
      }
    },
  };

  const getStoredAccountRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getStoredAccountRq',
    request: () => invoke<boolean>('is_account_stored'),
    onFulfilled: (stored) => {
      dispatch({ type: 'set-account', stored: stored || false });
    },
  };

  const getAccountSummaryRq: TauriReq<
    () => Promise<TAccountSummary | undefined>
  > = {
    name: 'getAccountSummaryRq',
    request: () => invoke<TAccountSummary>('get_account_summary'),
    onFulfilled: (summary) => {
      if (summary) {
        dispatch({ type: 'set-account-summary', summary });
      }
    },
  };

  const getFeatureFlagsRq: TauriReq<() => Promise<FeatureFlags | undefined>> = {
    name: 'getFeatureFlagsRq',
    request: () => invoke<FeatureFlags>('feature_flags'),
    onFulfilled: (flags) => {
      if (flags) {
        dispatch({ type: 'set-backend-flags', flags });
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

  const getDesktopNotificationsRq: TauriReq<() => Promise<boolean | null>> = {
    name: 'getDesktopNotificationsRq',
    request: () => kvGet<boolean>('desktop-notifications'),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-desktop-notifications',
        enabled: enabled || false,
      });
    },
  };

  const getRootFontSizeRq: TauriReq<() => Promise<number | null>> = {
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

  const getDebugLoggingRq: TauriReq<() => Promise<boolean | undefined>> = {
    name: 'getDebugLogging',
    request: () => invoke<boolean>('debug_logging_enabled'),
    onFulfilled: (enabled) => {
      dispatch({ type: 'set-debug-logging', enabled: enabled || false });
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

  const getNetworkStatsRq: TauriReq<() => Promise<boolean | null>> = {
    name: 'getNetworkStats',
    request: () => kvGet<boolean>('network-stats-enabled'),
    onFulfilled: (enabled) => {
      dispatch({
        type: 'set-network-stats',
        enabled: enabled !== null ? enabled : defaultNetStats,
      });
    },
  };

  // fire all requests concurrently
  await fireRequests([
    getVersionRq,
    getThemeRq,
    getRootFontSizeRq,
    getMonitoringRq,
    getDebugLoggingRq,
    getDepsRustRq,
    getDepsJsRq,
    getDesktopNotificationsRq,
    getNetworkStatsRq,
    initStateRq,
    getStoredAccountRq,
    getAccountStateRq,
    getAccountModeRq,
    getAccountSummaryRq,
    getFeatureFlagsRq,
  ]);
}

/**
 * Loads persisted favorites.
 *
 * Deliberately not part of `initFirstBatch` or `initSecondBatch`: both are only
 * invoked once the daemon is reachable, and favorites are a local file owned by
 * the app with no daemon involvement at all — there is no favorites RPC. Folding
 * this into a gated batch would make purely local state depend on a daemon
 * connection.
 */
export async function initFavorites() {
  const getFavoritesRq: TauriReq<() => Promise<Favorites>> = {
    name: 'getFavoritesRq',
    request: () => invoke<Favorites>('get_favorites'),
    onFulfilled: (favorites) => {
      useFavoritesStore.getState().hydrate(favorites);
    },
  };

  await fireRequests([getFavoritesRq]);
}

export async function initSecondBatch() {
  const getAccountLinksRq: TauriReq<() => Promise<AccountLinks | undefined>> = {
    name: 'getAccountLinksRq',
    request: () =>
      invoke<AccountLinks>('account_links', { locale: i18n.language }),
    onFulfilled: (links) => {
      dispatch({ type: 'set-account-links', links: links || null });
    },
  };

  const getAutostart: TauriReq<() => Promise<boolean>> = {
    name: 'getAutostart',
    request: () => isAutostartEnabled(),
    onFulfilled: (enabled) => {
      dispatch({ type: 'set-autostart', enabled });
    },
  };

  const getNetworkCompatRq: TauriReq<() => Promise<NetworkCompat | undefined>> =
    {
      name: 'getNetworkCompatRq',
      request: () => invoke<NetworkCompat>('network_compat'),
      onFulfilled: (compat) => {
        dispatch({ type: 'set-network-compat', compat: compat || null });
      },
    };

  const getDefaultDnsRq: TauriReq<() => Promise<string[] | undefined>> = {
    name: 'getDefaultDnsRq',
    request: () => invoke<string[]>('get_default_dns'),
    onFulfilled: (dns) => {
      dispatch({ type: 'set-default-dns', dns: dns || [] });
    },
  };

  await fireRequests([
    getAutostart,
    getDefaultDnsRq,
    getAccountLinksRq,
    getNetworkCompatRq,
  ]);
}
