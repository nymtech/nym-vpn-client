import { lazy } from 'react';
import { createBrowserRouter } from 'react-router-dom';
import * as Sentry from '@sentry/react';
import {
  AddCredential,
  Display,
  Error,
  Feedback,
  Legal,
  LegalRouteIndex,
  LicenseDetails,
  LicenseList,
  MainLayout,
  NodeLocation,
  Notifications,
  Settings,
  SettingsRouteIndex,
  Support,
  Welcome,
} from './pages';

// Lazy loads Home
const Home = lazy(() => import('./pages/home/Home'));

export const routes = {
  root: '/',
  credential: '/credential',
  settings: '/settings',
  display: '/settings/display',
  notifications: '/settings/notifications',
  logs: '/settings/logs',
  feedback: '/settings/feedback',
  feedbackSend: '/settings/feedback/send',
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
        path: routes.credential,
        element: <AddCredential />,
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
            path: routes.display,
            element: <Display />,
            errorElement: <Error />,
          },
          {
            path: routes.notifications,
            element: <Notifications />,
            errorElement: <Error />,
          },
          {
            path: routes.feedback,
            element: <Feedback />,
            errorElement: <Error />,
            children: [
              {
                path: routes.feedbackSend,
                // To be implemented
                element: <div />,
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
