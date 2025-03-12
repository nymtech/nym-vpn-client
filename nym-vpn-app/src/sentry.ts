import React from 'react';
import {
  createRoutesFromChildren,
  matchRoutes,
  useLocation,
  useNavigationType,
} from 'react-router';
import * as Sentry from '@sentry/react';
import { getVersion } from '@tauri-apps/api/app';

async function initSentry() {
  const dsn = import.meta.env.APP_SENTRY_DSN;
  let version = '0.0.0-unknown';
  try {
    version = await getVersion();
  } catch (e) {
    console.warn('failed to get app version from tauri:', e);
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

    release: `nym-vpn-app@${version}`,
  });

  Sentry.setTag('app_version', version);
  Sentry.setTag('client_version', 'x');
  Sentry.setUser({ id: 'nym', ip_address: undefined });
}

export default initSentry;
