import { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import i18n from 'i18next';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
  AccountLinks,
  MixnetEventPayload,
  ProgressEventPayload,
  StateDispatch,
  TunnelStateEvent as TunnelStatePayload,
  VpndStatus,
  isMixnetEventError,
  isVpndNonCompat,
  isVpndOk,
} from '../types';
import {
  DaemonEvent,
  MixnetEvent,
  ProgressEvent,
  TunnelStateEvent,
} from '../constants';
import { Notification } from '../contexts';
import { CCache } from '../cache';
import { daemonStatusUpdate } from './helper';
import { tunnelUpdate } from './tunnelUpdate';

export function useTauriEvents(
  dispatch: StateDispatch,
  push: (notification: Notification) => void,
) {
  const registerDaemonListener = useCallback(() => {
    return listen<VpndStatus>(
      DaemonEvent,
      async ({ event, payload: status }) => {
        console.info(
          `received event [${event}], status: ${status === 'down' ? status : JSON.stringify(status)}`,
        );
        daemonStatusUpdate(status, dispatch, push);
        await CCache.del('cache-account-id');
        await CCache.del('cache-device-id');

        // refresh account status
        if (isVpndOk(status) || isVpndNonCompat(status)) {
          try {
            const stored = await invoke<boolean | undefined>(
              'is_account_stored',
            );
            dispatch({ type: 'set-account', stored: stored || false });
          } catch (e: unknown) {
            console.error('failed to refresh daemon info', e);
          }
          try {
            const links = await invoke<AccountLinks>('account_links', {
              locale: i18n.language,
            });
            dispatch({ type: 'set-account-links', links });
          } catch (e: unknown) {
            console.warn('failed to get account links', e);
          }
        }
      },
    );
  }, [dispatch, push]);

  const registerStateListener = useCallback(() => {
    return listen<TunnelStatePayload>(TunnelStateEvent, (event) => {
      tunnelUpdate(event.payload.state, dispatch);
      if (event.payload.error) {
        console.log('tunnel error', event.payload.error);
        dispatch({
          type: 'set-error',
          error: event.payload.error,
        });
      }
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

  const registerProgressListener = useCallback(() => {
    return listen<ProgressEventPayload>(ProgressEvent, (event) => {
      console.log(
        `received event [${event.event}], message: ${event.payload.key}`,
      );
      dispatch({
        type: 'new-progress-message',
        message: event.payload.key,
      });
    });
  }, [dispatch]);

  const registerThemeChangedListener = useCallback(() => {
    const window = getCurrentWebviewWindow();
    return window.onThemeChanged(({ payload }) => {
      console.log(`system theme changed: ${payload}`);
      dispatch({
        type: 'system-theme-changed',
        theme: payload === 'dark' ? 'Dark' : 'Light',
      });
    });
  }, [dispatch]);

  // register/unregister event listener
  useEffect(() => {
    const unlistenDaemon = registerDaemonListener();
    const unlistenState = registerStateListener();
    const unlistenMixnetEvent = registerMixnetEventListener();
    const unlistenProgress = registerProgressListener();
    const unlistenThemeChanges = registerThemeChangedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenState.then((f) => f());
      unlistenMixnetEvent.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenThemeChanges.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerStateListener,
    registerMixnetEventListener,
    registerProgressListener,
    registerThemeChangedListener,
  ]);
}
