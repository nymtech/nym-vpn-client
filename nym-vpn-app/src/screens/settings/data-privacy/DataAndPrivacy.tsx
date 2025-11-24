import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch } from '../../../types';
import {
  AnonNetworkStatsUrl,
  SentryPrivacyPolicyUrl,
} from '../../../constants';

function DataAndPrivacy() {
  const { monitoring, networkStats } = useMainState();

  const dispatch = useMainDispatch() as StateDispatch;

  const { t } = useTranslation('settings');

  const onNetStatsChange = async () => {
    const isChecked = !networkStats;
    dispatch({ type: 'set-network-stats', enabled: isChecked });
    try {
      if (isChecked) {
        await invoke('enable_netstats');
      } else {
        await invoke('disable_netstats');
      }
    } catch {}
  };

  const onMonitoringChange = async () => {
    const isChecked = !monitoring;
    dispatch({ type: 'set-monitoring', enabled: isChecked });
    try {
      if (isChecked) {
        await invoke('enable_sentry');
      } else {
        await invoke('disable_sentry');
      }
    } catch {}
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('privacy.network-stats.label')}
            subheader={t('privacy.network-stats.sublabel')}
            subheaderColor="king-nacho"
            checked={networkStats}
            onClick={onNetStatsChange}
          />
        }
      >
        <div className="flex flex-col gap-2">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('privacy.network-stats.desc')}
          </p>
          <Link
            className="w-fit text-sm mt-2"
            text={t('privacy.network-stats.link')}
            url={AnonNetworkStatsUrl}
            color="primary"
            icon
          />
        </div>
      </SettingsMenuCardBig>
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('privacy.error-monitoring.label')}
            subheader={t('privacy.error-monitoring.sublabel')}
            subheaderColor="king-nacho"
            checked={monitoring}
            onClick={onMonitoringChange}
          />
        }
      >
        <div className="flex flex-col gap-2">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('privacy.error-monitoring.desc')}
          </p>
          <Link
            className="w-fit text-sm mt-2"
            text={t('privacy.error-monitoring.link')}
            url={SentryPrivacyPolicyUrl}
            color="primary"
            icon
          />
        </div>
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default DataAndPrivacy;
