import { useCallback, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import {
  BackendError,
  ConnectionEvent as ConnectionEventData,
  DaemonInfo,
  DaemonStatus,
  ProgressEventPayload,
  StateDispatch,
  StatusUpdatePayload,
} from '../types';
import {
  ConnectionEvent,
  DaemonEvent,
  ErrorEvent,
  ProgressEvent,
  StatusUpdateEvent,
} from '../constants';

function handleError(dispatch: StateDispatch, error?: BackendError | null) {
  if (!error) {
    dispatch({ type: 'reset-error' });
    return;
  }
  console.log('received backend error:', error);
  dispatch({ type: 'set-error', error });
}

export function useTauriEvents(dispatch: StateDispatch) {
  const registerDaemonListener = useCallback(() => {
    return listen<DaemonStatus>(DaemonEvent, async (event) => {
      console.info(`received event [${event.event}], status: ${event.payload}`);
      dispatch({
        type: 'set-daemon-status',
        status: event.payload,
      });

      // refresh daemon info and network env
      if (event.payload === 'Ok') {
        try {
          const info = await invoke<DaemonInfo>('daemon_info');
          dispatch({ type: 'set-daemon-info', info });
        } catch (e: unknown) {
          console.error('failed to get daemon info', e);
        }
      }
    });
  }, [dispatch]);

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
          dispatch({ type: 'change-connection-state', state: 'Connecting' });
          handleError(dispatch, event.payload.error);
          break;
        case 'Disconnecting':
          dispatch({ type: 'change-connection-state', state: 'Disconnecting' });
          handleError(dispatch, event.payload.error);
          break;
        case 'Unknown':
          dispatch({ type: 'change-connection-state', state: 'Unknown' });
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
