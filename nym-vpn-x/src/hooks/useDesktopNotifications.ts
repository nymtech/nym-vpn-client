import { useCallback, useEffect } from 'react';
import {
  isPermissionGranted,
  requestPermission,
} from '@tauri-apps/api/notification';
import { useMainDispatch, useMainState } from '../contexts';
import { kvSet } from '../kvStore';
import { StateDispatch } from '../types';

function useDesktopNotifications() {
  const { desktopNotifications } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  useEffect(() => {
    const checkPermission = async () => {
      const granted = await isPermissionGranted();
      if (desktopNotifications && !granted) {
        const permission = await requestPermission();
        dispatch({
          type: 'set-desktop-notifications',
          enabled: permission === 'granted',
        });
        kvSet('DesktopNotifications', permission === 'granted');
      }
    };

    checkPermission();
  }, [desktopNotifications, dispatch]);

  const toggle = useCallback(async () => {
    let enabled = !desktopNotifications;
    const granted = await isPermissionGranted();

    if (enabled && !granted) {
      const permission = await requestPermission();
      enabled = permission === 'granted';
    }

    if (enabled !== desktopNotifications) {
      dispatch({
        type: 'set-desktop-notifications',
        enabled: enabled,
      });
      kvSet('DesktopNotifications', enabled);
    }
  }, [dispatch, desktopNotifications]);

  return toggle;
}

export default useDesktopNotifications;
