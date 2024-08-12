import React, { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { ConnectionEvent } from '../constants';
import { useMainState } from '../contexts';
import { useI18nError, useNotify } from '../hooks';
import { routes } from '../router';
import { ConnectionEvent as ConnectionEventData } from '../types';

export default function EventNotification({
  children,
}: {
  children: React.ReactNode;
}) {
  const { error } = useMainState();
  const { notify } = useNotify();
  const { tE } = useI18nError();

  const { t } = useTranslation('notifications');

  const registerStateListener = useCallback(() => {
    return listen<ConnectionEventData>(ConnectionEvent, async (event) => {
      if (event.payload.type === 'Failed') {
        await notify(t('vpn-tunnel-state.failed'), {
          locationPath: routes.root,
          noSpamCheck: true,
        });
        return;
      }

      switch (event.payload.state) {
        case 'Connected':
          await notify(t('vpn-tunnel-state.connected'), {
            locationPath: routes.root,
            noSpamCheck: true,
          });
          break;
        case 'Disconnected':
          await notify(t('vpn-tunnel-state.disconnected'), {
            locationPath: routes.root,
            noSpamCheck: true,
          });
          break;
        default:
          break;
      }
    });
  }, [t, notify]);

  useEffect(() => {
    const unlistenState = registerStateListener();

    return () => {
      unlistenState.then((f) => f());
    };
  }, [registerStateListener]);

  useEffect(() => {
    if (error && error.key === 'EntryGatewayNotRouting') {
      notify(tE(error.key), {
        locationPath: routes.root,
      });
    }
  }, [tE, error, notify]);

  return <>{children}</>;
}
