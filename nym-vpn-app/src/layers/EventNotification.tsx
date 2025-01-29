import React, { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { TunnelStateEvent } from '../constants';
import { useNotify } from '../hooks';
import { routes } from '../router';
import {
  TunnelStateEvent as TunnelStateEventPayload,
  isTunnelConnected,
  isTunnelError,
  isTunnelOffline,
} from '../types';

export default function EventNotification({
  children,
}: {
  children: React.ReactNode;
}) {
  const { notify } = useNotify();

  const { t } = useTranslation('notifications');

  const registerStateListener = useCallback(() => {
    return listen<TunnelStateEventPayload>(TunnelStateEvent, async (event) => {
      if (event.payload.state === 'disconnected') {
        await notify(t('vpn-tunnel-state.disconnected'), {
          locationPath: routes.root,
          noSpamCheck: true,
        });
        return;
      }
      if (isTunnelConnected(event.payload.state)) {
        await notify(t('vpn-tunnel-state.connected'), {
          locationPath: routes.root,
          noSpamCheck: true,
        });
        return;
      }
      if (isTunnelOffline(event.payload.state)) {
        await notify(t('vpn-tunnel-state.offline'), {
          locationPath: routes.root,
          noSpamCheck: true,
        });
        return;
      }
      if (isTunnelError(event.payload.state)) {
        await notify(t('vpn-tunnel-state.error'), {
          locationPath: routes.root,
          noSpamCheck: true,
        });
        return;
      }
    });
  }, [t, notify]);

  useEffect(() => {
    const unlistenState = registerStateListener();

    return () => {
      unlistenState.then((f) => f());
    };
  }, [registerStateListener]);

  return <>{children}</>;
}
