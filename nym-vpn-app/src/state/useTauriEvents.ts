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
  StateDispatch,
  TAccountState,
  TunnelStateEvent as TunnelStatePayload,
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
} from '../constants';
import { Notification } from '../contexts';
import { CCache } from '../cache';
import { daemonStatusUpdate, networkEnvChanged } from './helper';
import { updateAccountState, updateTunnel } from './update';

export function useTauriEvents(
  dispatch: StateDispatch,
  push: (notification: Notification) => void,
) {
  const registerDaemonListener = useCallback(() => {
    return listen<VpndStatus>(
      DaemonEvent,
      async ({ event, payload: status }) => {
        console.log(
          `received event [${event}], status: ${status === 'down' ? status : JSON.stringify(status)}`,
        );
        daemonStatusUpdate(status, dispatch, push);
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
            dispatch({
              type: 'set-backend-flags',
              flags,
            });
          } catch {}
        }
      },
    );
  }, [dispatch, push]);

  const registerTunnelStateListener = useCallback(() => {
    return listen<TunnelStatePayload>(TunnelStateEvent, (event) => {
      updateTunnel(event.payload.state, dispatch);
      if (event.payload.error) {
        console.log('tunnel error', event.payload.error);
        dispatch({
          type: 'set-error',
          error: event.payload.error as BackendError,
        });
      }
    });
  }, [dispatch]);

  const registerAccountStateListener = useCallback(() => {
    return listen<TAccountState>(AccountStateEvent, ({ payload }) => {
      updateAccountState(payload, dispatch);
    });
  }, [dispatch]);

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
  }, [dispatch]);

  const registerThemeChangedListener = useCallback(() => {
    const window = getCurrentWebviewWindow();
    return window.onThemeChanged(({ payload }) => {
      console.log(`system theme changed: ${payload}`);
      dispatch({
        type: 'system-theme-changed',
        theme: payload === 'dark' ? 'dark' : 'light',
      });
    });
  }, [dispatch]);

  // register/unregister event listener
  useEffect(() => {
    const unlistenDaemon = registerDaemonListener();
    const unlistenTunnelState = registerTunnelStateListener();
    const unlistenAccountState = registerAccountStateListener();
    const unlistenMixnetEvent = registerMixnetEventListener();
    const unlistenThemeChanges = registerThemeChangedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenTunnelState.then((f) => f());
      unlistenAccountState.then((f) => f());
      unlistenMixnetEvent.then((f) => f());
      unlistenThemeChanges.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerTunnelStateListener,
    registerAccountStateListener,
    registerMixnetEventListener,
    registerThemeChangedListener,
  ]);
}
