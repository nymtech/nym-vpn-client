import { Outlet, useLocation } from 'react-router-dom';
import clsx from 'clsx';
import { useMainState } from '../contexts';
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
  const location = useLocation();
  const { daemonStatus } = useMainState();

  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-64',
        'bg-blanc-nacre text-baltic-sea',
        'dark:bg-baltic-sea dark:text-white',
      ])}
    >
      {!noTopBar && <TopBar />}
      {!noNotifications && <Notifications />}
      {!noDaemonDot && <DaemonDot status={daemonStatus} />}
      <div
        className={clsx([
          'h-full flex flex-col overflow-auto overscroll-auto p-4',
          (location.pathname === routes.licensesRust ||
            location.pathname === routes.licensesJs) &&
            '!p-0',
        ])}
      >
        <div className="grow">
          <Outlet />
        </div>
      </div>
    </div>
  );
}

export default MainLayout;
