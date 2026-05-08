import { useCallback, useEffect } from 'react';
import {
  isPermissionGranted,
  requestPermission,
} from '@tauri-apps/plugin-notification';
import { kvSet } from '../kvStore';
import { dispatch, useAppStore } from '../store';

function useDesktopNotifications() {
  const desktopNotifications = useAppStore((s) => s.desktopNotifications);

  useEffect(() => {
    const checkPermission = async () => {
      const granted = await isPermissionGranted();
      if (desktopNotifications && !granted) {
        const permission = await requestPermission();
        dispatch({
          type: 'set-desktop-notifications',
          enabled: permission === 'granted',
        });
        kvSet('desktop-notifications', permission === 'granted');
      }
    };

    checkPermission();
  }, [desktopNotifications]);

  const toggle = useCallback(async () => {
    let enabled = !desktopNotifications;
    const granted = await isPermissionGranted();

    if (enabled && !granted) {
      const permission = await requestPermission();
      enabled = permission === 'granted';
    }

    if (enabled !== desktopNotifications) {
      dispatch({ type: 'set-desktop-notifications', enabled });
      kvSet('desktop-notifications', enabled);
    }
  }, [desktopNotifications]);

  return toggle;
}

export default useDesktopNotifications;
