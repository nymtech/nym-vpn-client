import React from 'react';
import ReactDOM from 'react-dom/client';
import { invoke, isTauri } from '@tauri-apps/api/core';
import {
  WebviewWindow,
  getCurrentWebviewWindow,
} from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import duration from 'dayjs/plugin/duration';
import App from './App';
import { mockTauriIPC } from './dev/setup';
import { kvGet } from './kvStore';
import initSentry from './sentry';
import {
  StartupError as TStartupError,
  ThemeMode,
  VpnMode,
  VpndStatus,
} from './types';
import { StartupError } from './screens';
import { init } from './log';
import { DefaultVpnMode } from './constants';
import { S_STATE } from './static';

// needed locales to load for dayjs
import 'dayjs/locale/es';
import 'dayjs/locale/fr';
import 'dayjs/locale/hi';
import 'dayjs/locale/it';
import 'dayjs/locale/pt-br';
import 'dayjs/locale/ru';
import 'dayjs/locale/tr';
import 'dayjs/locale/uk';
import 'dayjs/locale/zh-cn';

const ErrorWindowLabel = 'error';

if (!import.meta.env.DEV) {
  // In production env, disable right-click context menu
  document.oncontextmenu = (event) => {
    event.preventDefault();
  };
}

if (import.meta.env.MODE === 'dev-browser') {
  console.log('Running in dev-browser mode. Mocking tauri window and IPCs');
  mockTauriIPC();
}

dayjs.extend(relativeTime);
dayjs.extend(duration);

async function setSplashTheme(window: WebviewWindow) {
  let isDarkMode = false;

  const mode = await kvGet<ThemeMode>('ui-theme');
  if (mode === 'dark') {
    isDarkMode = true;
  }
  if (!mode || mode === 'system') {
    const theme = await window.theme();
    if (theme === 'dark') {
      isDarkMode = true;
    }
  }
  if (isDarkMode) {
    const splash = document.getElementById('splash');
    splash?.classList.add('dark');
  }
}

(async () => {
  if (isTauri()) {
    init();
  }
  console.info('starting UI');

  const window = getCurrentWebviewWindow();
  if (window.label === 'main') {
    await setSplashTheme(window);
  }

  const env = await invoke<Record<string, unknown>>('env');
  if (env.DEV_MODE === true) {
    console.info('dev mode enabled');
    S_STATE.devMode = true;
  }

  S_STATE.vpnd =
    (await invoke<VpndStatus | undefined>('daemon_status')) || 'down';
  S_STATE.vpnModeAtStart = (await kvGet<VpnMode>('vpn-mode')) || DefaultVpnMode;

  // check for unrecoverable errors
  const error = await invoke<TStartupError | undefined>('startup_error');
  if (error) {
    console.info('get unrecoverable error');
    if (window.label !== ErrorWindowLabel) {
      // the index.html entry point is called by all webview windows rendering it
      // so check which window is calling it, if it's not the error window, return
      return;
    }
    const theme = await window.theme();
    document.getElementById('splash')?.remove();

    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <StartupError error={error} theme={theme} />
      </React.StrictMode>,
    );
    return;
  }

  const monitoring = await kvGet<boolean>('monitoring');

  if (monitoring) {
    await initSentry();
  }

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
})();
