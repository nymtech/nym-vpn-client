import { lazy } from 'react';
import { createBrowserRouter } from 'react-router';
import {
  AccountRouteIndex,
  AntiCensorship,
  Appearance,
  AppearanceRouteIndex,
  CustomDNS,
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
  NodeDetails,
  NodeEntry,
  Onboarding,
  PassphraseLogin,
  SelectPlan,
  Settings,
  SettingsRouteIndex,
  Socks5,
  Support,
  Welcome,
} from './screens';

// Lazy loads Home
const Home = lazy(() => import('./screens/home/Home'));

export const routes = {
  root: '/',
  login: '/login',
  passphraseLogin: '/login/passphrase',
  account: '/account',
  selectPlan: '/account/select-a-plan',
  settings: '/settings',
  appearance: '/settings/appearance',
  display: '/settings/appearance/display',
  lang: '/settings/appearance/lang',
  logs: '/settings/logs',
  dns: '/settings/dns',
  antiCensorship: '/settings/anti-censorship',
  socks5: '/settings/socks5',
  dataPrivacy: '/settings/data-privacy',
  support: '/settings/support',
  legal: '/settings/legal',
  licensesRust: '/settings/legal/licenses-rust',
  licensesJs: '/settings/legal/licenses-js',
  licenseDetails: '/settings/legal/license-details',
  dev: '/settings/dev',
  entryNodeLocation: '/entry-node-location',
  exitNodeLocation: '/exit-node-location',
  nodeDetails: '/node-details',
  hideout: '/hideout',
  welcome: '/hideout/welcome',
  onboarding: '/onboarding',
} as const;

// ⚠ router instance creation must remain outside of React
// tree with routes statically defined
const router = createBrowserRouter([
  {
    path: routes.root,
    Component: MainLayout,
    children: [
      {
        Component: Home,
        errorElement: <Error />,
        index: true,
      },
      {
        path: routes.login,
        Component: Login,
        errorElement: <Error />,
      },
      {
        path: routes.passphraseLogin,
        Component: PassphraseLogin,
        errorElement: <Error />,
      },
      {
        path: routes.onboarding,
        Component: Onboarding,
        errorElement: <Error />,
      },
      {
        path: routes.account,
        Component: AccountRouteIndex,
        errorElement: <Error />,
        children: [
          {
            path: routes.selectPlan,
            Component: SelectPlan,
            errorElement: <Error />,
          },
        ],
      },
      {
        path: routes.settings,
        Component: SettingsRouteIndex,
        errorElement: <Error />,
        children: [
          {
            Component: Settings,
            errorElement: <Error />,
            index: true,
          },
          {
            path: routes.dev,
            Component: Dev,
            errorElement: <Error />,
          },
          {
            path: routes.appearance,
            Component: AppearanceRouteIndex,
            errorElement: <Error />,
            children: [
              {
                Component: Appearance,
                errorElement: <Error />,
                index: true,
              },
              {
                path: routes.lang,
                Component: Lang,
                errorElement: <Error />,
              },
              {
                path: routes.display,
                Component: Display,
                errorElement: <Error />,
              },
            ],
          },
          {
            path: routes.dataPrivacy,
            Component: DataAndPrivacy,
            errorElement: <Error />,
          },
          {
            path: routes.logs,
            Component: Logs,
            errorElement: <Error />,
          },
          {
            path: routes.dns,
            Component: CustomDNS,
            errorElement: <Error />,
          },
          {
            path: routes.antiCensorship,
            Component: AntiCensorship,
            errorElement: <Error />,
          },
          {
            path: routes.socks5,
            Component: Socks5,
            errorElement: <Error />,
          },
          {
            path: routes.support,
            Component: Support,
            errorElement: <Error />,
          },
          {
            path: routes.legal,
            Component: LegalRouteIndex,
            errorElement: <Error />,
            children: [
              {
                Component: Legal,
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
                Component: LicenseDetails,
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
      {
        path: routes.nodeDetails,
        Component: NodeDetails,
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
        Component: Welcome,
        errorElement: <Error />,
      },
    ],
  },
]);

export default router;
