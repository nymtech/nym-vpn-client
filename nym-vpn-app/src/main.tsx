import React from 'react';
import ReactDOM from 'react-dom/client';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import duration from 'dayjs/plugin/duration';
import App from './App';
import { mockTauriIPC } from './dev/setup';
import { kvGet } from './kvStore';
import { InitState, SelectedNode, VpnMode, VpndStatus } from './types';
import { StartupError } from './screens';
import { init } from './log';
import { getTheme } from './util';
import { FEATURE_KEY as STREAMING_OPTIMIZED_LABEL_FEATURE_KEY } from './screens/home/new-feature-alert/streaming-optimized-label/constants';
import { DefaultNode } from './constants';

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

console.log('env', window._APP);

const devMode = window._APP.devMode;
const startupError = window._APP.startupError;
const defaultVpnMode = window._APP.defaultVpnMode;
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

(async () => {
  if (isTauri()) {
    init();
  }
  console.info('starting UI');

  const window = getCurrentWebviewWindow();

  if (devMode) {
    console.info('dev mode enabled');
  }

  // check for unrecoverable errors
  if (startupError) {
    console.info('startup error');
    if (window.label !== ErrorWindowLabel) {
      // the index.html entry point is called by all webview windows rendering it
      // so check which window is calling it, if it's not the error window, return
      return;
    }
    const theme = await window.theme();

    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <StartupError error={startupError} theme={theme} />
      </React.StrictMode>,
    );
    return;
  }

  // pre-get and prepare some early stage state
  const initState: InitState = {
    vpnd: (await invoke<VpndStatus | undefined>('daemon_status')) || 'down',
    vpnMode: (await kvGet<VpnMode>('vpn-mode')) || defaultVpnMode,
    uiTheme: await getTheme(),
    welcomeChecked: (await kvGet<boolean>('welcome-screen-seen')) || false,
    streamingOptimizedLabelSeen:
      (await kvGet<boolean>(STREAMING_OPTIMIZED_LABEL_FEATURE_KEY)) || false,
    entryNode: (await kvGet<SelectedNode>('entry-node')) || DefaultNode,
    exitNode: (await kvGet<SelectedNode>('exit-node')) || DefaultNode,
  };
  console.log('initial state:', initState);

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App init={initState} />
    </React.StrictMode>,
  );
})();
