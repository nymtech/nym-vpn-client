import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  isPermissionGranted,
  requestPermission,
} from '@tauri-apps/api/notification';
import { useMainDispatch, useMainState } from '../../../contexts';
import { kvSet } from '../../../kvStore';
import { StateDispatch } from '../../../types';
import { PageAnim, SettingsMenuCard, Switch } from '../../../ui';

function Notifications() {
  const { desktopNotifications } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');

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

  const handleNotificationsChange = async () => {
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
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCard
        title={t('notifications.title')}
        onClick={handleNotificationsChange}
        leadingIcon="notifications_active"
        trailingComponent={
          <Switch
            checked={desktopNotifications}
            onChange={handleNotificationsChange}
          />
        }
      />
    </PageAnim>
  );
}

export default Notifications;
