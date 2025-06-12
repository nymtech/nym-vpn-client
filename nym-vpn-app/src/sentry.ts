import React from 'react';
import {
  createRoutesFromChildren,
  matchRoutes,
  useLocation,
  useNavigationType,
} from 'react-router';
import * as Sentry from '@sentry/react';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { type as osType } from '@tauri-apps/plugin-os';
import { OsInfo } from './types';

const os = osType();

async function initSentry() {
  const dsn = import.meta.env.APP_SENTRY_DSN;
  let version = '0.0.0-unknown';
  let osInfo;
  try {
    version = await getVersion();
    osInfo = await invoke<OsInfo>('os_info');
  } catch (e) {
    console.warn('failed to get system info:', e);
  }

  if (!dsn) {
    console.warn(`unable to initialize Sentry, APP_SENTRY_DSN env var not set`);
    return;
  }
  console.info(
    '⚠ performance monitoring and error reporting enabled, initializing Sentry',
  );

  Sentry.init({
    dsn,
    integrations: [
      Sentry.reactRouterV7BrowserTracingIntegration({
        useEffect: React.useEffect,
        useLocation,
        useNavigationType,
        createRoutesFromChildren,
        matchRoutes,
      }),
      Sentry.replayIntegration({
        maskAllText: false,
        blockAllMedia: false,
      }),
      // captures console API calls
      Sentry.captureConsoleIntegration({ levels: ['error', 'warn'] }),
    ],
    tracePropagationTargets: ['localhost'],

    // TODO adjust this in the future, 100% is not recommended for production
    tracesSampleRate: 1.0,

    // Capture Replay for 10% of all sessions,
    // plus for 100% of sessions with an error
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,

    // import.meta.env.MODE is set by Vite and is either
    // 'development' or 'production'
    environment: import.meta.env.MODE,

    release: version,
  });

  Sentry.setTag('app_version', version);
  Sentry.setUser({ id: 'nym', ip_address: undefined });
  if (osInfo) {
    Sentry.setTag('os_long', osInfo?.name || 'unknown');
    Sentry.setTag('os_kernel', osInfo?.kernel || 'unknown');
    if (os === 'linux') {
      Sentry.setTag('gpu', osInfo?.gpu || 'unknown');
      Sentry.setTag('display_server', osInfo?.displayServer || 'unknown');
    }
  }
}

export default initSentry;
