import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import {
  CardSwitch,
  Link,
  MsIcon,
  PageAnim,
  SettingsMenuCardBig,
} from '../../../ui';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch } from '../../../types';
import {
  AnonNetworkStatsUrl,
  SentryPrivacyPolicyUrl,
} from '../../../constants';
import SettingsGroup from '../SettingsGroup';
import { routes } from '../../../router';

function DataAndPrivacy() {
  const { monitoring, networkStats } = useMainState();

  const dispatch = useMainDispatch() as StateDispatch;

  const { t } = useTranslation('settings');
  const navigate = useNavigate();

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
      <SettingsGroup
        settings={[
          {
            title: t('logs.title'),
            leadingIcon: 'sort',
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
            onClick: () => navigate(routes.logs),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('diagnostic.title'),
            leadingIcon: 'monitor_heart',
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
            onClick: () => navigate(routes.diagnostic),
          },
        ]}
      />

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
