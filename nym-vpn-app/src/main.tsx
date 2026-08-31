import React from 'react';
import ReactDOM from 'react-dom/client';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import localizedFormat from 'dayjs/plugin/localizedFormat';
import duration from 'dayjs/plugin/duration';
import App from './App';
import { mockTauriIPC } from './dev/setup';
import { kvGet } from './kvStore';
import { InitState, VpndConfig, VpndStatus } from './types';
import { StartupError } from './screens';
import { init } from './log';
import { getTheme } from './util';
import { DefaultNode } from './constants';

// needed locales to load for dayjs
import 'dayjs/locale/ar';
import 'dayjs/locale/fa';
import 'dayjs/locale/bn';
import 'dayjs/locale/de';
import 'dayjs/locale/en';
import 'dayjs/locale/es';
import 'dayjs/locale/fr';
import 'dayjs/locale/hi';
import 'dayjs/locale/pt';
import 'dayjs/locale/ru';
import 'dayjs/locale/tr';
import 'dayjs/locale/uk';
import 'dayjs/locale/vi';
import 'dayjs/locale/zh';

console.log('env', window._APP);

const devMode = window._APP.devMode;
const startupError = window._APP.startupError;
const defaultVpnMode = window._APP.defaultVpnMode;
const defaultQuic = window._APP.defaultQuic;
const defaultNoIpv6 = false;
const defaultAllowLan = false;
const ErrorWindowLabel = 'error';
const defaultMixnetTrafficConfig = {
  poissonParameterForLoopCoverStream: null,
  averagePacketDelay: null,
  messageSendingAverageDelay: null,
  disablePoissonRate: false,
  disableBackgroundCoverTraffic: false,
  minMixnodePerformance: null,
  minGatewayMixnetPerformance: null,
};

const defaultMixnetTrafficDefaults = {
  mixingDelay: {
    minValue: 0,
    maxValue: 0,
    defaultValue: 0,
  },
  disablePoissonRate: false,
  defaultBackgroundTraffic: { value: 0, multiplier: '' },
  defaultContinuousTraffic: { value: 0, throughput: '' },
  allBackgroundTraffic: [],
  allContinuousTraffic: [],
};

const defaultSplitTunnel = {
  enabled: false,
  apps: [],
};

const defaultGeoExclusion = {
  enabled: false,
  listenPort: 1081,
  excludedCountries: ['CN'],
};

const defaultGatewaySelectionAlgorithmConfig = { enableGeoLocation: true };

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
dayjs.extend(localizedFormat);

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

  const config = await invoke<VpndConfig | undefined>('get_vpn_config');
  console.log('config', config);

  // pre-get and prepare some early stage state
  const initState: InitState = {
    vpnd: (await invoke<VpndStatus | undefined>('daemon_status')) || 'down',
    vpnMode: config?.vpnMode || defaultVpnMode,
    uiTheme: await getTheme(),
    technicalOptinSeen: (await kvGet<boolean>('technical-optin-seen')) || false,
    entryNode: config?.entryNode || DefaultNode,
    exitNode: config?.exitNode || DefaultNode,
    quic: config?.bridges !== undefined ? config.bridges : defaultQuic,
    noIpv6:
      config?.disableIpv6 !== undefined ? config.disableIpv6 : defaultNoIpv6,
    allowLan:
      config?.allowLan !== undefined ? config.allowLan : defaultAllowLan,
    enableAdBlocking:
      config?.enableAdBlocking !== undefined ? config.enableAdBlocking : false,
    customDnsEnabled:
      config?.enableCustomDns !== undefined ? config.enableCustomDns : false,
    customDns: !config?.customDns ? [] : config.customDns,
    mixnetTrafficConfig: config?.mixnetTraffic || defaultMixnetTrafficConfig,
    mixnetTrafficDefaults:
      config?.mixnetTrafficDefaults || defaultMixnetTrafficDefaults,
    splitTunnel: config?.splitTunnel || defaultSplitTunnel,
    geoExclusion: config?.geoExclusion || defaultGeoExclusion,
    gatewaySelectionAlgorithmConfig:
      config?.gatewaySelectionAlgorithmConfig ||
      defaultGatewaySelectionAlgorithmConfig,
    frontingMode: config?.frontingMode || 'onRetry',
    gatewayIndependenceNotifications:
      config?.gatewayIndependenceNotifications ?? true,
  };
  console.log('initial state:', initState);

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App init={initState} />
    </React.StrictMode>,
  );
})();
