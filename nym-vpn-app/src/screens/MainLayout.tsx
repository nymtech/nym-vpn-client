import { Outlet, useLocation } from 'react-router';
import clsx from 'clsx';
import { useMainState } from '../contexts';
import { EventNotification } from '../layers';
import { routes } from '../router';
import { DaemonDot, Notifications, TopBar } from '../ui';

type MainLayoutProps = {
  noTopBar?: boolean;
  noNotifications?: boolean;
  noDaemonDot?: boolean;
};

function MainLayout({
  noTopBar,
  noNotifications,
  noDaemonDot,
}: MainLayoutProps) {
  const { daemonStatus } = useMainState();
  const location = useLocation();

  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-64',
        'bg-faded-lavender text-baltic-sea',
        'dark:bg-ash dark:text-white',
      ])}
    >
      {!noTopBar && <TopBar />}
      {!noNotifications && <Notifications />}
      {!noDaemonDot && <DaemonDot status={daemonStatus} />}
      <div
        className={clsx([
          'h-full flex flex-col overflow-auto overscroll-auto p-4',
          (location.pathname === routes.licensesRust ||
            location.pathname === routes.licensesJs ||
            location.pathname === routes.entryNodeLocation ||
            location.pathname === routes.exitNodeLocation) &&
            'p-0!',
        ])}
      >
        <div className="grow">
          <EventNotification>
            <Outlet />
          </EventNotification>
        </div>
      </div>
    </div>
  );
}

export default MainLayout;
