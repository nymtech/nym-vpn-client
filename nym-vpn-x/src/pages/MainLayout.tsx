import { useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Outlet, useLocation } from 'react-router-dom';
import clsx from 'clsx';
import { listen } from '@tauri-apps/api/event';
import { AppName, ConnectionEvent } from '../constants';
import { useMainState } from '../contexts';
import { useNotify } from '../hooks';
import { routes } from '../router';
import { ConnectionEvent as ConnectionEventData } from '../types';
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
  const { notify } = useNotify();

  const location = useLocation();
  const { t } = useTranslation('notifications');

  const registerStateListener = useCallback(() => {
    return listen<ConnectionEventData>(ConnectionEvent, (event) => {
      if (event.payload.type === 'Failed') {
        notify(t('vpn-tunnel-state.failed'), AppName, false, routes.root);
        return;
      }

      switch (event.payload.state) {
        case 'Connected':
          notify(t('vpn-tunnel-state.connected'), AppName, false, routes.root);
          break;
        case 'Disconnected':
          notify(
            t('vpn-tunnel-state.disconnected'),
            AppName,
            false,
            routes.root,
          );
          break;
        default:
          break;
      }
    });
  }, [notify, t]);

  useEffect(() => {
    const unlistenState = registerStateListener();

    return () => {
      unlistenState.then((f) => f());
    };
  }, [registerStateListener]);

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
