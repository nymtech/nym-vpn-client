import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { PageAnim, Switch } from '../../../ui/index';
import SettingsGroup from '../SettingsGroup';
import { dispatch, useAppStore, useMainState } from '../../../store/index';
import { useDesktopNotifications } from '../../../hooks/index';

function Notifications() {
  const { t } = useTranslation('settings');
  const { desktopNotifications } = useMainState();
  const toggleDesktopNotifications = useDesktopNotifications();
  const familyReminders = useAppStore(
    (s) => s.gatewayIndependenceNotifications,
  );

  const toggleFamilyReminders = useCallback(async () => {
    const next = !familyReminders;
    // optimistic update; daemon is the source of truth
    dispatch({ type: 'set-gateway-independence-notifications', enabled: next });
    try {
      await invoke('set_gateway_independence_notifications', { enabled: next });
    } catch (e) {
      console.error('failed to set gateway independence notifications', e);
      // revert on failure
      dispatch({
        type: 'set-gateway-independence-notifications',
        enabled: familyReminders,
      });
    }
  }, [familyReminders]);

  return (
    <PageAnim className="flex h-full flex-col gap-4 select-none">
      <SettingsGroup
        settings={[
          {
            title: t('notifications.server-family-reminders.title'),
            desc: t('notifications.server-family-reminders.description'),
            leadingIcon: 'groups',
            onClick: toggleFamilyReminders,
            trailing: (
              <Switch
                checked={familyReminders}
                onChange={toggleFamilyReminders}
              />
            ),
          },
          {
            title: t('notifications.desktop-notifications.title'),
            leadingIcon: 'notifications',
            onClick: toggleDesktopNotifications,
            trailing: (
              <Switch
                checked={desktopNotifications}
                onChange={toggleDesktopNotifications}
              />
            ),
          },
        ]}
      />
    </PageAnim>
  );
}

export default Notifications;
