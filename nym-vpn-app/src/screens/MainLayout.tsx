import * as H from 'history';
import { useEffect, useRef } from 'react';
import { Outlet, useLocation } from 'react-router';
import clsx from 'clsx';
import { useMainState } from '../contexts';
import { EventNotification } from '../layers';
import { routes } from '../router';
import { DaemonDot, Notifications, TopBar } from '../ui';
import InitialNavigation from './InitialNavigation';

type MainLayoutProps = {
  noTopBar?: boolean;
  noNotifications?: boolean;
  noDaemonDot?: boolean;
};

type RouteState = {
  resetScroll?: boolean;
};

function MainLayout({
  noTopBar,
  noNotifications,
  noDaemonDot,
}: MainLayoutProps) {
  const { daemonStatus } = useMainState();
  const location = useLocation() as H.Location<RouteState | undefined>;
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!location.state?.resetScroll || !rootRef) {
      return;
    }
    rootRef.current?.scrollIntoView({
      behavior: 'instant',
      block: 'start',
      inline: 'start',
    });
  }, [location]);

  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-64',
        'bg-faded-lavender text-baltic-sea',
        'dark:bg-ash dark:text-white',
      ])}
    >
      <InitialNavigation />
      {!noNotifications && <Notifications />}
      {!noTopBar && <TopBar />}
      {!noDaemonDot && <DaemonDot status={daemonStatus} />}
      <div
        className={clsx([
          'h-full flex flex-col overflow-auto overscroll-auto p-4',
          (location.pathname === routes.licensesRust ||
            location.pathname === routes.licensesJs ||
            location.pathname === routes.entryNodeLocation ||
            location.pathname === routes.exitNodeLocation ||
            location.pathname === routes.nodeDetails) &&
            'p-0!',
        ])}
      >
        <div
          ref={rootRef}
          className={clsx([
            'grow',
            location.pathname === routes.nodeDetails && 'h-full',
          ])}
        >
          <EventNotification>
            <Outlet />
          </EventNotification>
        </div>
      </div>
    </div>
  );
}

export default MainLayout;
