import { useCallback, useEffect, useRef, useState } from 'react';
import { useLocation } from 'react-router';
import {
  isPermissionGranted,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { type } from '@tauri-apps/plugin-os';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { AppName } from '../constants';
import { useMainState } from '../contexts';

const AntiSpamTimeout = 60000; // 1min

/**
 * Desktop notification options
 */
export type NotifyOptions = {
  // By default, the notification is not sent if the app is focused and visible
  // or if a notification has already been sent just before with the same text
  // Set this to `true` to send the notification anyway
  force?: boolean;
  // The pathname of a location, if the notification should
  // trigger when the user is **not** on a specific screen.
  // Ignored if `force` is `true`
  locationPath?: string;
  // If `true` the check for consecutive identical notifications will not be done
  noSpamCheck?: boolean;
};

/**
 * Hook to send desktop notifications
 *
 * @returns The `notify` function
 */
function useNotify() {
  const { desktopNotifications } = useMainState();
  const location = useLocation();

  const [lastNotification, setLastNotification] = useState<string | null>(null);
  const id = useRef<number>(0);

  useEffect(() => {
    if (lastNotification) {
      clearTimeout(id.current);
      id.current = setTimeout(() => {
        setLastNotification(null);
      }, AntiSpamTimeout) as unknown as number;
    }
  }, [lastNotification]);

  /**
   * Sends desktop notifications. Also checks if the permission is granted
   * and desktop notifications are enabled.
   *
   * @param body - The text to display in the notification
   * @param opts - Notification options
   */
  const notify = useCallback(
    async (body: string, opts: NotifyOptions = {}) => {
      const { force = false, locationPath, noSpamCheck } = opts;
      const os = type();
      const window = getCurrentWebviewWindow();

      if (!desktopNotifications) {
        return;
      }

      if (!force) {
        const windowFocused = await window.isFocused();
        const windowVisible = await window.isVisible();
        const onRightScreen = locationPath
          ? location.pathname === locationPath
          : true;
        if (windowFocused && windowVisible && onRightScreen) {
          return;
        }
        if (!noSpamCheck && body === lastNotification) {
          return;
        }
      }

      const granted = await isPermissionGranted();
      if (!granted) {
        console.log('Desktop notifications permission not granted');
        return;
      }

      if (os === 'linux') {
        sendNotification({ title: AppName, body });
      } else {
        sendNotification(body);
      }
      setLastNotification(body);
    },
    [desktopNotifications, location, lastNotification],
  );

  return { notify };
}

export default useNotify;
