import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { MsIcon, PageAnim, Switch } from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import SettingsGroup from '../SettingsGroup';
import { routes } from '../../../router';
import TelemetryInfoDialog, {
  InfoButton,
  TelemetryInfoKind,
} from '../../technical-opt-in/TelemetryInfoDialog';

function DataAndPrivacy() {
  const { monitoring, networkStats } = useMainState();
  const [info, setInfo] = useState<TelemetryInfoKind | null>(null);

  const { t } = useTranslation(['settings', 'welcome']);
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
    <PageAnim className="mt-2 flex h-full flex-col gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('logs.title'),
            leadingIcon: 'sort',
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
            onClick: () => navigate(routes.logs),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('diagnostic.title'),
            leadingIcon: 'monitor_heart',
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
            onClick: () => navigate(routes.diagnostic),
          },
        ]}
      />

      {/* Telemetry opt-ins — identical to the onboarding screen */}
      <SettingsGroup
        settings={[
          {
            title: t('connection.label', { ns: 'welcome' }),
            titleTrailing: (
              <InfoButton
                kind="connection"
                label={t('connection.label', { ns: 'welcome' })}
                onOpen={setInfo}
                data-testid="data-privacy-netstats-info"
              />
            ),
            desc: t('connection.desc', { ns: 'welcome' }),
            leadingIcon: 'analytics',
            trailing: (
              <Switch
                checked={networkStats}
                onChange={onNetStatsChange}
                data-testid="data-privacy-netstats-switch"
              />
            ),
            'data-testid': 'data-privacy-netstats-option',
          },
          {
            title: t('monitoring.label', { ns: 'welcome' }),
            titleTrailing: (
              <InfoButton
                kind="monitoring"
                label={t('monitoring.label', { ns: 'welcome' })}
                onOpen={setInfo}
                data-testid="data-privacy-monitoring-info"
              />
            ),
            desc: (
              <div className="flex flex-col">
                <span>{t('monitoring.desc', { ns: 'welcome' })}</span>
                <span className="text-text-tertiary">
                  {t('privacy.monitoring-restart-note')}
                </span>
              </div>
            ),
            leadingIcon: 'bug_report',
            trailing: (
              <Switch
                checked={monitoring}
                onChange={onMonitoringChange}
                data-testid="data-privacy-monitoring-switch"
              />
            ),
            'data-testid': 'data-privacy-monitoring-option',
          },
        ]}
      />

      <TelemetryInfoDialog kind={info} onClose={() => setInfo(null)} />
    </PageAnim>
  );
}

export default DataAndPrivacy;
