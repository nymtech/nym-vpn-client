import { lazy } from 'react';
import { createBrowserRouter } from 'react-router';
import {
  AccountRouteIndex,
  AntiCensorship,
  Appearance,
  AppearanceRouteIndex,
  DataAndPrivacy,
  Dev,
  Display,
  Error,
  Lang,
  Legal,
  LegalRouteIndex,
  LicenseDetails,
  LicenseList,
  Login,
  Logs,
  MainLayout,
  NodeEntry,
  SelectPlan,
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
  account: '/account',
  selectPlan: '/account/select-a-plan',
  settings: '/settings',
  appearance: '/settings/appearance',
  display: '/settings/appearance/display',
  lang: '/settings/appearance/lang',
  logs: '/settings/logs',
  antiCensorship: '/settings/anti-censorship',
  dataPrivacy: '/settings/data-privacy',
  support: '/settings/support',
  legal: '/settings/legal',
  licensesRust: '/settings/legal/licenses-rust',
  licensesJs: '/settings/legal/licenses-js',
  licenseDetails: '/settings/legal/license-details',
  dev: '/settings/dev',
  entryNodeLocation: '/entry-node-location',
  exitNodeLocation: '/exit-node-location',
  hideout: '/hideout',
  welcome: '/hideout/welcome',
} as const;

// ⚠ router instance creation must remain outside of React
// tree with routes statically defined
const router = createBrowserRouter([
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
        path: routes.account,
        element: <AccountRouteIndex />,
        errorElement: <Error />,
        children: [
          {
            path: routes.selectPlan,
            element: <SelectPlan />,
            errorElement: <Error />,
          },
        ],
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
            path: routes.dev,
            element: <Dev />,
            errorElement: <Error />,
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
            path: routes.dataPrivacy,
            element: <DataAndPrivacy />,
            errorElement: <Error />,
          },
          {
            path: routes.logs,
            element: <Logs />,
            errorElement: <Error />,
          },
          {
            path: routes.antiCensorship,
            element: <AntiCensorship />,
            errorElement: <Error />,
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
        element: <NodeEntry node="entry" />,
        errorElement: <Error />,
      },
      {
        path: routes.exitNodeLocation,
        element: <NodeEntry node="exit" />,
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
