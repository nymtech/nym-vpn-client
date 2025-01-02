import { lazy } from 'react';
import { createBrowserRouter } from 'react-router';
import * as Sentry from '@sentry/react';
import {
  Appearance,
  AppearanceRouteIndex,
  Display,
  Error,
  Lang,
  Legal,
  LegalRouteIndex,
  LicenseDetails,
  LicenseList,
  Login,
  MainLayout,
  NodeLocation,
  Settings,
  SettingsRouteIndex,
  Support,
  Welcome,
} from './screens';

// Lazy loads Home
const Home = lazy(() => import('./screens/home/Home'));

export const routes = {
  root: '/',
  login: '/login',
  settings: '/settings',
  appearance: '/settings/appearance',
  display: '/settings/appearance/display',
  lang: '/settings/appearance/lang',
  logs: '/settings/logs',
  support: '/settings/support',
  legal: '/settings/legal',
  licensesRust: '/settings/legal/licenses-rust',
  licensesJs: '/settings/legal/licenses-js',
  licenseDetails: '/settings/legal/license-details',
  entryNodeLocation: '/entry-node-location',
  exitNodeLocation: '/exit-node-location',
  hideout: '/hideout',
  welcome: '/hideout/welcome',
} as const;

// Even if Sentry is not instantiated, wrapping the router seems OK
const createRouterFn = Sentry.wrapCreateBrowserRouter(createBrowserRouter);

// ⚠ router instance creation must remain outside of React
// tree with routes statically defined
const router = createRouterFn([
  {
    path: routes.root,
    element: <MainLayout />,
    children: [
      {
        element: <Home />,
        errorElement: <Error />,
        index: true,
      },
      {
        path: routes.login,
        element: <Login />,
        errorElement: <Error />,
      },
      {
        path: routes.settings,
        element: <SettingsRouteIndex />,
        errorElement: <Error />,
        children: [
          {
            element: <Settings />,
            errorElement: <Error />,
            index: true,
          },
          {
            path: routes.appearance,
            element: <AppearanceRouteIndex />,
            errorElement: <Error />,
            children: [
              {
                element: <Appearance />,
                errorElement: <Error />,
                index: true,
              },
              {
                path: routes.lang,
                element: <Lang />,
                errorElement: <Error />,
              },
              {
                path: routes.display,
                element: <Display />,
                errorElement: <Error />,
              },
            ],
          },
          {
            path: routes.support,
            element: <Support />,
            errorElement: <Error />,
          },
          {
            path: routes.legal,
            element: <LegalRouteIndex />,
            errorElement: <Error />,
            children: [
              {
                element: <Legal />,
                errorElement: <Error />,
                index: true,
              },
              {
                path: routes.licensesRust,
                element: <LicenseList language="rust" />,
                errorElement: <Error />,
              },
              {
                path: routes.licensesJs,
                element: <LicenseList language="js" />,
                errorElement: <Error />,
              },
              {
                path: routes.licenseDetails,
                element: <LicenseDetails />,
                errorElement: <Error />,
              },
            ],
          },
        ],
      },
      {
        path: routes.entryNodeLocation,
        element: <NodeLocation node="entry" />,
        errorElement: <Error />,
      },
      {
        path: routes.exitNodeLocation,
        element: <NodeLocation node="exit" />,
        errorElement: <Error />,
      },
    ],
  },
  {
    path: routes.hideout,
    element: <MainLayout noTopBar noNotifications noDaemonDot />,
    children: [
      {
        path: routes.welcome,
        element: <Welcome />,
        errorElement: <Error />,
      },
    ],
  },
]);

export default router;
