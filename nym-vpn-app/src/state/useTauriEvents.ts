import { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import i18n from 'i18next';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
  AccountLinks,
  BackendError,
  FeatureFlags,
  MixnetEventPayload,
  TAccountState,
  TunnelStateEvent as TunnelStatePayload,
  VpndConfig,
  VpndStatus,
  isMixnetEventError,
  isVpndNonCompat,
  isVpndOk,
} from '../types';
import {
  AccountStateEvent,
  DaemonEvent,
  MixnetEvent,
  TunnelStateEvent,
  UpdatePendingEvent,
  VpnConfigEvent,
} from '../constants';
import { CCache } from '../cache';
import { ToastAddData } from '../hooks';
import { dispatch } from '../store';
import { daemonStatusUpdate, networkEnvChanged } from './helper';
import { updateAccountState, updateTunnel } from './update';

export function useTauriEvents(
  add: (data: ToastAddData) => string,
  close: (id: string) => void,
) {
  const registerDaemonListener = useCallback(() => {
    let liveEventSeen = false;
    return listen<VpndStatus>(
      DaemonEvent,
      async ({ event, payload: status }) => {
        liveEventSeen = true;
        console.log(
          `received event [${event}], status: ${status === 'down' ? status : JSON.stringify(status)}`,
        );
        daemonStatusUpdate(status, add, close);
        const changed = await networkEnvChanged(status);
        if (changed) {
          console.info('network env changed, clearing cache');
          await CCache.clear();
        } else {
          await CCache.del('cache-account-id');
          await CCache.del('cache-device-id');
        }

        // when (re)connected to daemon, refresh some state
        if (isVpndOk(status) || isVpndNonCompat(status)) {
          try {
            const stored = await invoke<boolean | undefined>(
              'is_account_stored',
            );
            dispatch({ type: 'set-account', stored: stored || false });
          } catch {}
          try {
            const links = await invoke<AccountLinks>('account_links', {
              locale: i18n.language,
            });
            dispatch({ type: 'set-account-links', links });
          } catch {}
          try {
            const flags = await invoke<FeatureFlags>('feature_flags');
            dispatch({ type: 'set-backend-flags', flags });
          } catch {}
        }
      },
    ).then(async (unlisten) => {
      try {
        const status =
          (await invoke<VpndStatus | undefined>('daemon_status')) || 'down';
        if (!liveEventSeen) {
          daemonStatusUpdate(status, add, close);
        }
      } catch (error) {
        console.info('command [daemon_status] catch-up failed', error);
      }
      return unlisten;
    });
  }, [add, close]);

  const registerTunnelStateListener = useCallback(() => {
    return listen<TunnelStatePayload>(TunnelStateEvent, (event) => {
      console.log('tunnel state update', event);
      updateTunnel(event.payload.state);
      if (event.payload.error) {
        console.log('tunnel error', event.payload.error);
        dispatch({
          type: 'set-error',
          error: event.payload.error as BackendError,
        });
      }
    });
  }, []);

  const registerAccountStateListener = useCallback(() => {
    return listen<TAccountState>(AccountStateEvent, ({ payload }) => {
      updateAccountState(payload);
    });
  }, []);

  const registerMixnetEventListener = useCallback(() => {
    return listen<MixnetEventPayload>(MixnetEvent, (event) => {
      const { payload } = event;
      if (isMixnetEventError(payload)) {
        console.info(`received mixnet event [${event.event}]`, payload);
        dispatch({
          type: 'set-error',
          error: { key: payload.error, message: payload.error },
        });
      }
    });
  }, []);

  const registerThemeChangedListener = useCallback(() => {
    const window = getCurrentWebviewWindow();
    return window.onThemeChanged(({ payload }) => {
      console.log(`system theme changed: ${payload}`);
      dispatch({
        type: 'system-theme-changed',
        theme: payload === 'dark' ? 'dark' : 'light',
      });
    });
  }, []);

  const registerVpnConfigListener = useCallback(() => {
    return listen<VpndConfig>(VpnConfigEvent, ({ payload }) => {
      dispatch({ type: 'update-tunnel-config', config: payload });
    });
  }, []);

  const registerUpdatePendingListener = useCallback(() => {
    return listen(UpdatePendingEvent, () => {
      console.info('[update] new version installed, restart required');
      dispatch({ type: 'set-linux-app-updated', updated: true });
    });
  }, []);

  // register/unregister event listeners
  useEffect(() => {
    const unlistenDaemon = registerDaemonListener();
    const unlistenTunnelState = registerTunnelStateListener();
    const unlistenAccountState = registerAccountStateListener();
    const unlistenMixnetEvent = registerMixnetEventListener();
    const unlistenThemeChanges = registerThemeChangedListener();
    const unlistenVpnConfig = registerVpnConfigListener();
    const unlistenUpdatePending = registerUpdatePendingListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenTunnelState.then((f) => f());
      unlistenAccountState.then((f) => f());
      unlistenMixnetEvent.then((f) => f());
      unlistenThemeChanges.then((f) => f());
      unlistenVpnConfig.then((f) => f());
      unlistenUpdatePending.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerTunnelStateListener,
    registerAccountStateListener,
    registerMixnetEventListener,
    registerThemeChangedListener,
    registerVpnConfigListener,
    registerUpdatePendingListener,
  ]);
}
