import {
  isPermissionGranted,
  sendNotification,
} from '@tauri-apps/api/notification';
import { appWindow } from '@tauri-apps/api/window';
import { useCallback } from 'react';
import { useLocation } from 'react-router-dom';
import { useMainState } from '../contexts';

/**
 * Hook to send desktop notifications
 *
 * @returns The `notify` function
 */
function useNotify() {
  const { desktopNotifications } = useMainState();
  const location = useLocation();

  /**
   * Sends desktop notifications. Also checks if the permission is granted
   * and desktop notifications are enabled.
   *
   * @param body - The text to display in the notification
   * @param title - The title of the notification (optional)
   * @param force - Whether to send the notification even if the app is focused and visible
   * @param locationPath - The pathname of a location, if the notification should
   *   only be sent when the user is **not** on a specific screen
   */
  const notify = useCallback(
    async (
      body: string,
      title: string | null,
      force = false,
      locationPath?: string,
    ) => {
      if (!desktopNotifications) {
        return;
      }

      if (!force) {
        const windowFocused = await appWindow.isFocused();
        const windowVisible = await appWindow.isVisible();
        const onRightScreen = locationPath
          ? location.pathname === locationPath
          : true;
        if (windowFocused && windowVisible && onRightScreen) {
          return;
        }
      }

      const granted = await isPermissionGranted();
      if (!granted) {
        console.warn('Desktop notifications permission not granted');
        return;
      }

      if (title) {
        sendNotification({ title, body });
      } else {
        sendNotification(body);
      }
    },
    [desktopNotifications, location],
  );

  return { notify };
}

export default useNotify;
