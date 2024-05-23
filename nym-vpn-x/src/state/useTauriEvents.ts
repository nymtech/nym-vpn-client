import * as _ from 'lodash-es';
import { useCallback, useEffect } from 'react';
import { EventCallback, listen } from '@tauri-apps/api/event';
import { PhysicalSize, appWindow } from '@tauri-apps/api/window';
import dayjs from 'dayjs';
import { kvSet } from '../kvStore';
import {
  AppState,
  BkdError,
  ConnectionEventPayload,
  DaemonStatus,
  ProgressEventPayload,
  StateDispatch,
  WindowSize,
} from '../types';
import { ConnectionEvent, DaemonEvent, ProgressEvent } from '../constants';

function handleError(dispatch: StateDispatch, error?: BkdError | null) {
  if (!error) {
    dispatch({ type: 'reset-error' });
    return;
  }
  console.log('received backend error:', error);
  dispatch({ type: 'set-error', error });
}

export function useTauriEvents(dispatch: StateDispatch, state: AppState) {
  const registerDaemonListener = useCallback(() => {
    return listen<DaemonStatus>(DaemonEvent, (event) => {
      console.log(`received event ${event.event}, status: ${event.payload}`);
      dispatch({
        type: 'set-daemon-status',
        status: event.payload,
      });
    });
  }, [dispatch]);

  const registerStateListener = useCallback(() => {
    return listen<ConnectionEventPayload>(ConnectionEvent, (event) => {
      console.log(
        `received event ${event.event}, state: ${event.payload.state}`,
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

  const registerProgressListener = useCallback(() => {
    return listen<ProgressEventPayload>(ProgressEvent, (event) => {
      console.log(
        `received event ${event.event}, message: ${event.payload.key}`,
      );
      dispatch({
        type: 'new-progress-message',
        message: event.payload.key,
      });
    });
  }, [dispatch]);

  const registerThemeChangedListener = useCallback(() => {
    return appWindow.onThemeChanged(({ payload }) => {
      console.log(`system theme changed: ${payload}`);
      dispatch({
        type: 'system-theme-changed',
        theme: payload === 'dark' ? 'Dark' : 'Light',
      });
    });
  }, [dispatch]);

  const registerWindowResizedListener = useCallback(() => {
    return appWindow.onResized(
      _.debounce<EventCallback<PhysicalSize>>(
        ({ payload }) => {
          if (
            payload.width !== state.windowSize?.width ||
            payload.height !== state.windowSize?.height
          ) {
            const size: WindowSize = {
              type: 'Physical',
              width: payload.width,
              height: payload.height,
            };
            kvSet<WindowSize>('WindowSize', size);
            dispatch({ type: 'set-window-size', size });
          }
        },
        200,
        {
          leading: false,
          trailing: true,
        },
      ),
    );
  }, [dispatch, state.windowSize]);

  // register/unregister event listener
  useEffect(() => {
    const unlistenDaemon = registerDaemonListener();
    const unlistenState = registerStateListener();
    const unlistenProgress = registerProgressListener();
    const unlistenThemeChanges = registerThemeChangedListener();
    const unlistenWindowResized = registerWindowResizedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenState.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenThemeChanges.then((f) => f());
      unlistenWindowResized.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerStateListener,
    registerProgressListener,
    registerThemeChangedListener,
    registerWindowResizedListener,
  ]);
}
