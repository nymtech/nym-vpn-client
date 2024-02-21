import { createBrowserRouter } from 'react-router-dom';
import {
  Display,
  Error,
  Feedback,
  Home,
  Legal,
  LogIn,
  MainLayout,
  NodeLocation,
  Settings,
  SettingsLayout,
  Support,
} from './pages';

export const routes = {
  root: '/',
  login: '/login',
  settings: '/settings',
  display: '/settings/display',
  logs: '/settings/logs',
  feedback: '/settings/feedback',
  feedbackSend: '/settings/feedback/send',
  support: '/settings/support',
  legal: '/settings/legal',
  entryNodeLocation: '/entry-node-location',
  exitNodeLocation: '/exit-node-location',
} as const;

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
        element: <LogIn />,
        errorElement: <Error />,
      },
      {
        path: routes.settings,
        element: <SettingsLayout />,
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
            element: <Legal />,
            errorElement: <Error />,
          },
        ],
      },
      {
        path: routes.entryNodeLocation,
        // eslint-disable-next-line react/jsx-no-undef
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
]);

export default router;
