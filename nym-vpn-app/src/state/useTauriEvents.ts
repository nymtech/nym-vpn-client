import { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import i18n from 'i18next';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import {
  AccountLinks,
  ProgressEventPayload,
  StateDispatch,
  StatusUpdatePayload,
  TunnelStateIpc,
  VpndStatus,
  isTunnelConnected,
  isTunnelConnecting,
  isTunnelDisconnecting,
  isTunnelError,
  isTunnelOffline,
  isVpndNonCompat,
  isVpndOk,
} from '../types';
import {
  DaemonEvent,
  ProgressEvent,
  StatusUpdateEvent,
  TunnelStateEvent,
} from '../constants';
import { Notification } from '../contexts';
import { daemonStatusUpdate } from './helper';
import { MCache } from '../cache';

export function useTauriEvents(
  dispatch: StateDispatch,
  push: (notification: Notification) => void,
) {
  const registerDaemonListener = useCallback(() => {
    return listen<VpndStatus>(
      DaemonEvent,
      async ({ event, payload: status }) => {
        console.info(
          `received event [${event}], status: ${status === 'notOk' ? status : JSON.stringify(status)}`,
        );
        daemonStatusUpdate(status, dispatch, push);
        MCache.del('account-id');
        MCache.del('device-id');

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
    return listen<TunnelStateIpc>(TunnelStateEvent, (event) => {
      if (event.payload === 'disconnected') {
        console.log('tunnel event [disconnected]');
        dispatch({ type: 'set-tunnel-disconnected' });
        return;
      }
      if (isTunnelConnected(event.payload)) {
        console.log('tunnel event [connected]');
        dispatch({
          type: 'set-tunnel-connected',
          tunnel: event.payload.connected,
        });
        return;
      }
      if (isTunnelConnecting(event.payload)) {
        console.log('tunnel event [connecting]');
        dispatch({
          type: 'set-tunnel-connecting',
          tunnel: event.payload.connecting,
        });
        return;
      }
      if (isTunnelDisconnecting(event.payload)) {
        console.log(
          `tunnel event [disconnecting], action ${event.payload.disconnecting}`,
        );
        dispatch({
          type: 'set-tunnel-disconnecting',
          action: event.payload.disconnecting,
        });
        return;
      }
      if (isTunnelOffline(event.payload)) {
        console.log(
          `tunnel event [offline], reconnect: ${event.payload.offline.reconnect}`,
        );
        dispatch({
          type: 'set-tunnel-offline',
          reconnect: event.payload.offline.reconnect,
        });
        return;
      }
      if (isTunnelError(event.payload)) {
        console.log(`tunnel event [error] - ${event.payload.error}`);
        dispatch({
          type: 'set-tunnel-inerror',
          error: event.payload.error,
        });
        return;
      }
    });
  }, [dispatch]);

  const registerStatusUpdateListener = useCallback(() => {
    return listen<StatusUpdatePayload>(StatusUpdateEvent, (event) => {
      const { payload } = event;
      console.log(`received event [${event.event}]`, payload);
      if (payload.error) {
        dispatch({
          type: 'set-error',
          error: payload.error,
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
    const unlistenStatusUpdate = registerStatusUpdateListener();
    const unlistenProgress = registerProgressListener();
    const unlistenThemeChanges = registerThemeChangedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenState.then((f) => f());
      unlistenStatusUpdate.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenThemeChanges.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerStateListener,
    registerStatusUpdateListener,
    registerProgressListener,
    registerThemeChangedListener,
  ]);
}
