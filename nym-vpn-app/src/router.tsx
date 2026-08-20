import { createBrowserRouter } from 'react-router';
import {
  AccountRouteIndex,
  AccountScreen,
  AntiCensorship,
  Appearance,
  AppearanceRouteIndex,
  CustomDNS,
  DataAndPrivacy,
  Dev,
  Diagnostic,
  Display,
  Error,
  GeoExclusion,
  GeoExclusionSelectRegion,
  GeoExclusionSetup,
  Lang,
  Legal,
  LegalRouteIndex,
  LicenseDetails,
  LicenseList,
  Logs,
  MainLayout,
  MixnetTuning,
  NodeDetails,
  NodeLocation,
  Notifications,
  Onboarding,
  Profiles,
  SelectPlan,
  Settings,
  SettingsRouteIndex,
  Socks5,
  SplitTunneling,
  Support,
  TechnicalOptin,
  WelcomeContainer,
} from './screens';

import Home from './screens/home/Home';
import StartupGate from './screens/StartupGate';

export const routes = {
  root: '/home',
  account: '/account',
  selectPlan: '/account/select-a-plan',
  settings: '/settings',
  appearance: '/settings/appearance',
  display: '/settings/appearance/display',
  lang: '/settings/appearance/lang',
  dns: '/settings/dns',
  antiCensorship: '/settings/anti-censorship',
  profiles: '/settings/profiles',
  socks5: '/settings/socks5',
  dataPrivacy: '/settings/data-privacy',
  logs: '/settings/data-privacy/logs',
  diagnostic: '/settings/data-privacy/diagnostic',
  splitTunneling: '/settings/split-tunneling',
  geoExclusion: '/settings/geo-exclusion',
  geoExclusionSetup: '/settings/geo-exclusion/setup-instructions',
  geoExclusionSelectRegion: '/settings/geo-exclusion/select-region',
  notifications: '/settings/notifications',
  support: '/settings/support',
  legal: '/settings/legal',
  licensesRust: '/settings/legal/licenses-rust',
  licensesJs: '/settings/legal/licenses-js',
  licenseDetails: '/settings/legal/license-details',
  dev: '/settings/dev',
  entryNodeLocation: '/entry-node-location',
  exitNodeLocation: '/exit-node-location',
  nodeLocation: '/node-location',
  nodeDetails: '/node-details',
  hideout: '/hideout',
  onboarding: '/hideout/onboarding',
  mixnetTuning: '/settings/mixnet-tuning',
  accountSettings: '/settings/account',
  welcome: '/welcome',
  technicalOptin: '/technical-optin',
} as const;

// ⚠ router instance creation must remain outside of React
// tree with routes statically defined
const router = createBrowserRouter([
  {
    path: '/',
    Component: StartupGate,
    index: true,
  },
  {
    element: <MainLayout />,
    children: [
      {
        path: routes.root,
        Component: Home,
        errorElement: <Error />,
      },
      {
        path: routes.technicalOptin,
        Component: TechnicalOptin,
        errorElement: <Error />,
      },
      {
        path: routes.welcome,
        Component: WelcomeContainer,
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
            path: routes.accountSettings,
            Component: AccountScreen,
            errorElement: <Error />,
          },
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
            errorElement: <Error />,
            children: [
              {
                Component: DataAndPrivacy,
                errorElement: <Error />,
                index: true,
              },
              {
                path: routes.logs,
                Component: Logs,
                errorElement: <Error />,
              },
              {
                path: routes.diagnostic,
                Component: Diagnostic,
                errorElement: <Error />,
              },
            ],
          },
          {
            path: routes.dns,
            Component: CustomDNS,
            errorElement: <Error />,
          },
          {
            path: routes.mixnetTuning,
            Component: MixnetTuning,
            errorElement: <Error />,
          },
          {
            path: routes.splitTunneling,
            Component: SplitTunneling,
            errorElement: <Error />,
          },
          {
            path: routes.geoExclusion,
            Component: GeoExclusion,
            errorElement: <Error />,
          },
          {
            path: routes.geoExclusionSetup,
            Component: GeoExclusionSetup,
            errorElement: <Error />,
          },
          {
            path: routes.geoExclusionSelectRegion,
            Component: GeoExclusionSelectRegion,
            errorElement: <Error />,
          },
          {
            path: routes.antiCensorship,
            Component: AntiCensorship,
            errorElement: <Error />,
          },
          {
            path: routes.profiles,
            Component: Profiles,
            errorElement: <Error />,
          },
          {
            path: routes.socks5,
            Component: Socks5,
            errorElement: <Error />,
          },
          {
            path: routes.notifications,
            Component: Notifications,
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
        path: routes.nodeLocation,
        Component: NodeLocation,
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
        path: routes.onboarding,
        Component: Onboarding,
        errorElement: <Error />,
      },
    ],
  },
]);

export default router;
