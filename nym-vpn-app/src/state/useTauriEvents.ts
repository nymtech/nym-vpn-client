import { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import i18n from 'i18next';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import {
  AccountLinks,
  BackendError,
  ConnectionEvent as ConnectionEventData,
  ProgressEventPayload,
  StateDispatch,
  StatusUpdatePayload,
  VpndStatus,
  isVpndNonCompat,
  isVpndOk,
} from '../types';
import {
  ConnectionEvent,
  DaemonEvent,
  ErrorEvent,
  ProgressEvent,
  StatusUpdateEvent,
} from '../constants';
import { Notification } from '../contexts';
import { daemonStatusUpdate } from './helper';
import { MCache } from '../cache';

function handleError(dispatch: StateDispatch, error?: BackendError | null) {
  if (!error) {
    dispatch({ type: 'reset-error' });
    return;
  }
  console.log('received backend error:', error);
  // TODO remove this dirty hack once switched to the new tunnel API
  if (
    error.key === 'CSDaemonInternal' &&
    error.data?.reason.includes('SameEntryAndExitGateway')
  ) {
    dispatch({
      type: 'set-error',
      error: {
        key: 'CStateGwDirSameEntryAndExitGw',
        message: 'Cannot connect to the same entry and exit gateway',
      },
    });
    return;
  }
  dispatch({ type: 'set-error', error });
}

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
    return listen<ConnectionEventData>(ConnectionEvent, (event) => {
      if (event.payload.type === 'Failed') {
        console.log(`received event [${event.event}], connection failed`);
        handleError(dispatch, event.payload);
        return;
      }
      console.log(
        `received event [${event.event}], state: ${event.payload.state}`,
      );
      switch (event.payload.state) {
        case 'Connected':
          dispatch({
            type: 'set-connected',
            startTime:
              (event.payload.start_time as unknown as number) || dayjs().unix(),
          });
          handleError(dispatch, event.payload.error);
          break;
        case 'Disconnected':
          dispatch({ type: 'set-disconnected' });
          handleError(dispatch, event.payload.error);
          break;
        case 'Connecting':
          dispatch({ type: 'update-connection-state', state: 'Connecting' });
          handleError(dispatch, event.payload.error);
          break;
        case 'Disconnecting':
          dispatch({ type: 'update-connection-state', state: 'Disconnecting' });
          handleError(dispatch, event.payload.error);
          break;
        case 'Unknown':
          dispatch({ type: 'update-connection-state', state: 'Unknown' });
          handleError(dispatch, event.payload.error);
          break;
      }
    });
  }, [dispatch]);

  const registerErrorListener = useCallback(() => {
    return listen<BackendError>(ErrorEvent, (event) => {
      console.info(`received event [${event.event}]`, event.payload);
      dispatch({
        type: 'set-error',
        error: event.payload,
      });
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
    const unlistenError = registerErrorListener();
    const unlistenStatusUpdate = registerStatusUpdateListener();
    const unlistenProgress = registerProgressListener();
    const unlistenThemeChanges = registerThemeChangedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenState.then((f) => f());
      unlistenError.then((f) => f());
      unlistenStatusUpdate.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenThemeChanges.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerStateListener,
    registerErrorListener,
    registerStatusUpdateListener,
    registerProgressListener,
    registerThemeChangedListener,
  ]);
}
