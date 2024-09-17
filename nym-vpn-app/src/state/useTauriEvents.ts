import * as _ from 'lodash-es';
import { useCallback, useEffect } from 'react';
import { EventCallback, listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import dayjs from 'dayjs';
import { kvSet } from '../kvStore';
import {
  AppState,
  BackendError,
  ConnectionEvent as ConnectionEventData,
  DaemonStatus,
  ProgressEventPayload,
  StateDispatch,
  StatusUpdatePayload,
  WindowPosition,
  WindowSize,
} from '../types';
import {
  ConnectionEvent,
  DaemonEvent,
  ProgressEvent,
  StatusUpdateEvent,
} from '../constants';
import logu from '../log';
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';

const appWindow = getCurrentWebviewWindow();

function handleError(dispatch: StateDispatch, error?: BackendError | null) {
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
      console.log(`received event [${event.event}], status: ${event.payload}`);
      dispatch({
        type: 'set-daemon-status',
        status: event.payload,
      });
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
          if (payload.width === 0 || payload.height === 0) {
            // that happens when window is minimized
            return;
          }
          if (
            payload.width !== state.windowSize?.width ||
            payload.height !== state.windowSize.height
          ) {
            const size: WindowSize = {
              type: 'Physical',
              width: payload.width,
              height: payload.height,
            };
            logu.trace(
              `window resized ${payload.type} ${size.width}x${size.height}`,
            );
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

  const registerWindowMovedListener = useCallback(() => {
    return appWindow.onMoved(
      _.debounce<EventCallback<PhysicalPosition>>(
        ({ payload }) => {
          if (payload.x < 0 || payload.y < 0) {
            // that happens when moving the window on a secondary monitor
            return;
          }
          if (
            payload.x !== state.windowPosition?.x ||
            payload.y !== state.windowPosition.y
          ) {
            const position: WindowPosition = {
              type: 'Physical',
              x: payload.x,
              y: payload.y,
            };
            logu.trace(
              `window moved ${payload.type} ${payload.x},${payload.y}`,
            );
            kvSet<WindowPosition>('WindowPosition', position);
            dispatch({ type: 'set-window-position', position });
          }
        },
        200,
        {
          leading: false,
          trailing: true,
        },
      ),
    );
  }, [dispatch, state.windowPosition]);

  // register/unregister event listener
  useEffect(() => {
    const unlistenDaemon = registerDaemonListener();
    const unlistenState = registerStateListener();
    const unlistenStatusUpdate = registerStatusUpdateListener();
    const unlistenProgress = registerProgressListener();
    const unlistenThemeChanges = registerThemeChangedListener();
    const unlistenWindowResized = registerWindowResizedListener();
    const unlistenWindowMoved = registerWindowMovedListener();

    return () => {
      unlistenDaemon.then((f) => f());
      unlistenState.then((f) => f());
      unlistenStatusUpdate.then((f) => f());
      unlistenProgress.then((f) => f());
      unlistenThemeChanges.then((f) => f());
      unlistenWindowResized.then((f) => f());
      unlistenWindowMoved.then((f) => f());
    };
  }, [
    registerDaemonListener,
    registerStateListener,
    registerStatusUpdateListener,
    registerProgressListener,
    registerThemeChangedListener,
    registerWindowResizedListener,
    registerWindowMovedListener,
  ]);
}
