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
  LayoutError,
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
    // for throws in the layout itself; screens are covered one level down so
    // that their errors keep the layout and stay recoverable by navigation
    errorElement: <LayoutError />,
    children: [
      {
        errorElement: <Error />,
        children: [
          {
            path: routes.root,
            Component: Home,
          },
          {
            path: routes.technicalOptin,
            Component: TechnicalOptin,
          },
          {
            path: routes.welcome,
            Component: WelcomeContainer,
          },
          {
            path: routes.account,
            Component: AccountRouteIndex,
            children: [
              {
                path: routes.selectPlan,
                Component: SelectPlan,
              },
            ],
          },
          {
            path: routes.settings,
            Component: SettingsRouteIndex,
            children: [
              {
                path: routes.accountSettings,
                Component: AccountScreen,
              },
              {
                Component: Settings,
                index: true,
              },
              {
                path: routes.dev,
                Component: Dev,
              },
              {
                path: routes.appearance,
                Component: AppearanceRouteIndex,
                children: [
                  {
                    Component: Appearance,
                    index: true,
                  },
                  {
                    path: routes.lang,
                    Component: Lang,
                  },
                  {
                    path: routes.display,
                    Component: Display,
                  },
                ],
              },
              {
                path: routes.dataPrivacy,
                children: [
                  {
                    Component: DataAndPrivacy,
                    index: true,
                  },
                  {
                    path: routes.logs,
                    Component: Logs,
                  },
                  {
                    path: routes.diagnostic,
                    Component: Diagnostic,
                  },
                ],
              },
              {
                path: routes.dns,
                Component: CustomDNS,
              },
              {
                path: routes.mixnetTuning,
                Component: MixnetTuning,
              },
              {
                path: routes.splitTunneling,
                Component: SplitTunneling,
              },
              {
                path: routes.geoExclusion,
                Component: GeoExclusion,
              },
              {
                path: routes.geoExclusionSetup,
                Component: GeoExclusionSetup,
              },
              {
                path: routes.geoExclusionSelectRegion,
                Component: GeoExclusionSelectRegion,
              },
              {
                path: routes.antiCensorship,
                Component: AntiCensorship,
              },
              {
                path: routes.socks5,
                Component: Socks5,
              },
              {
                path: routes.notifications,
                Component: Notifications,
              },
              {
                path: routes.support,
                Component: Support,
              },
              {
                path: routes.legal,
                Component: LegalRouteIndex,
                children: [
                  {
                    Component: Legal,
                    index: true,
                  },
                  {
                    path: routes.licensesRust,
                    element: <LicenseList language="rust" />,
                  },
                  {
                    path: routes.licensesJs,
                    element: <LicenseList language="js" />,
                  },
                  {
                    path: routes.licenseDetails,
                    Component: LicenseDetails,
                  },
                ],
              },
            ],
          },
          {
            path: routes.nodeLocation,
            Component: NodeLocation,
          },
          {
            path: routes.nodeDetails,
            Component: NodeDetails,
          },
        ],
      },
    ],
  },
  {
    path: routes.hideout,
    element: <MainLayout noTopBar noNotifications noDaemonDot />,
    errorElement: <LayoutError />,
    children: [
      {
        errorElement: <Error />,
        children: [
          {
            path: routes.onboarding,
            Component: Onboarding,
          },
        ],
      },
    ],
  },
]);

export default router;
