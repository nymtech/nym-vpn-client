import { openPath } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { PageAnim } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Logs() {
  const { t } = useTranslation('settings');

  const handleAppLogs = async () => {
    try {
      const dir = await invoke<string | undefined>('log_dir');
      console.log("app logs dir", dir);
      if (dir) {
        await openPath(dir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleDaemonLogs = async () => {
    try {
      const dir = await invoke<string | undefined>('vpnd_log_dir');
      console.log('daemon logs dir', dir);
      if (dir) {
        await openPath(dir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <PageAnim
      className="h-full flex flex-col mt-2 gap-6"
      data-testid="logs-page"
    >
      <SettingsGroup
        settings={[
          {
            title: t('logs.app'),
            leadingIcon: 'sort',
            onClick: handleAppLogs,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('logs.daemon'),
            leadingIcon: 'sort',
            onClick: handleDaemonLogs,
            trailingIcon: 'open_in_new',
          },
        ]}
      />
    </PageAnim>
  );
}

export default Logs;
