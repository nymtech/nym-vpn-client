import { useCallback } from 'react';
import { openPath } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { PageAnim, Switch } from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import SettingsGroup from '../SettingsGroup';

function Logs() {
  const { t } = useTranslation('settings');
  const { debugLogging } = useMainState();

  const toggleDebugLogging = useCallback(async () => {
    const next = !debugLogging;
    dispatch({ type: 'set-debug-logging', enabled: next });
    try {
      await invoke('set_debug_logging', { enabled: next });
    } catch (e) {
      console.error('failed to set debug logging', e);
      dispatch({ type: 'set-debug-logging', enabled: debugLogging });
    }
  }, [debugLogging]);

  const handleAppLogs = async () => {
    try {
      const dir = await invoke<string | undefined>('log_dir');
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
      if (dir) {
        await openPath(dir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <PageAnim
      className="mt-2 flex h-full flex-col gap-6"
      data-testid="logs-page"
    >
      <SettingsGroup
        settings={[
          {
            title: t('logs.debug-logging.title'),
            leadingIcon: 'bug_report',
            onClick: toggleDebugLogging,
            trailing: (
              <Switch
                checked={debugLogging}
                onChange={toggleDebugLogging}
                data-testid="debug-logging-switch"
              />
            ),
          },
        ]}
      />
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
