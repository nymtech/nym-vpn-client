import * as H from 'history';
import { useEffect, useRef } from 'react';
import { Outlet, useLocation } from 'react-router';
import clsx from 'clsx';
import { EventNotification } from '../layers';
import { CardAnimationProvider } from '../contexts/CardAnimationContext';
import { routes } from '../router';
import { DaemonDot, TopBar } from '../ui';
import { ToastList } from '../components/toast';
import { useAppStore } from '../store';
import { SystemAuthentication } from './SystemAuthentication';

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
  const daemonStatus = useAppStore((s) => s.daemonStatus);
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
        'flex h-full min-w-64 flex-col',
        'bg-gray dark:bg-background',
        'text-primary',
      ])}
    >
      <SystemAuthentication />
      {!noNotifications && <ToastList />}
      {!noTopBar && <TopBar />}
      {!noDaemonDot && <DaemonDot status={daemonStatus} />}
      <div
        className={clsx([
          'flex h-full flex-col overflow-auto overscroll-auto p-4',
          (location.pathname === routes.licensesRust ||
            location.pathname === routes.licensesJs ||
            location.pathname === routes.entryNodeLocation ||
            location.pathname === routes.exitNodeLocation ||
            location.pathname === routes.nodeLocation ||
            location.pathname === routes.nodeDetails) &&
            'p-0!',
        ])}
      >
        <CardAnimationProvider>
          <div
            ref={rootRef}
            className={clsx([
              'grow',
              location.pathname === routes.nodeDetails && 'h-full',
              location.pathname === routes.diagnostic && 'h-full',
              location.pathname === routes.nodeLocation && 'h-full',
            ])}
          >
            <EventNotification>
              <Outlet />
            </EventNotification>
          </div>
        </CardAnimationProvider>
      </div>
    </div>
  );
}

export default MainLayout;
