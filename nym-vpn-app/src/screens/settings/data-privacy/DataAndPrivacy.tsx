import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { PageAnim, SettingsMenuCard } from '../../../ui';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';
import { StateDispatch } from '../../../types';

function DataAndPrivacy() {
  const { monitoring, networkStats } = useMainState();

  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useInAppNotify();

  const { t } = useTranslation('settings');

  // notify the user at most once per every 10s when he toggles monitoring
  const showMonitoringAlert = () => {
    push({
      id: 'monitoring-alert',
      message: t('monitoring-alert'),
      close: true,
      type: 'warn',
      throttle: 10,
    });
  };

  const handleMonitoringChanged = async () => {
    const isChecked = !monitoring;
    showMonitoringAlert();
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    try {
      if (isChecked) {
        await invoke('enable_sentry');
      } else {
        await invoke('disable_sentry');
      }
    } catch {}
  };

  return (
    <PageAnim
      className="h-full flex flex-col mt-2 gap-6"
      data-testid="logs-page"
    >
      <SettingsMenuCard
        title={t('ERROR MONITORING')}
        leadingIcon="sort"
        onClick={handleMonitoringChanged}
        trailingIcon="open_in_new"
        data-testid="daemon-logs-button"
      />
    </PageAnim>
  );
}

export default DataAndPrivacy;
