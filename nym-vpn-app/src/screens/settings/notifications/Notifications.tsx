import { useTranslation } from 'react-i18next';
import { PageAnim, Switch } from '../../../ui/index';
import SettingsGroup from '../SettingsGroup';
import { useMainState } from '../../../store/index';
import {
  useDesktopNotifications,
  useServerFamilyReminders,
} from '../../../hooks/index';

function Notifications() {
  const { t } = useTranslation('settings');
  const { desktopNotifications } = useMainState();
  const toggleDesktopNotifications = useDesktopNotifications();
  const { enabled: familyReminders, toggle: toggleFamilyReminders } =
    useServerFamilyReminders();
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
