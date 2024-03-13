import React from 'react';
import {
  createRoutesFromChildren,
  matchRoutes,
  useLocation,
  useNavigationType,
} from 'react-router-dom';
import * as Sentry from '@sentry/react';
import { captureConsoleIntegration } from '@sentry/integrations';
import { getVersion } from '@tauri-apps/api/app';

function initSentry() {
  const dsn = import.meta.env.APP_SENTRY_DSN;

  if (!dsn) {
    console.warn(`unable to initialize sentry, APP_SENTRY_DSN env var not set`);
    return;
  }
  console.log('⚠ performance monitoring and error reporting enabled');
  console.log('initializing sentry');

  Sentry.init({
    dsn,
    integrations: [
      Sentry.reactRouterV6BrowserTracingIntegration({
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
      captureConsoleIntegration({ levels: ['error', 'warn'] }),
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
  });

  getVersion().then((version) => {
    Sentry.setTag('app_version', version);
  });

  Sentry.setUser({ id: 'nym', ip_address: undefined });
}

export default initSentry;
