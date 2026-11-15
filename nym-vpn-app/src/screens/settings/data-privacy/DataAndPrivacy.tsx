import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useToast } from '../../../hooks';
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
  const [netStatsPending, setNetStatsPending] = useState(false);
  const [monitoringPending, setMonitoringPending] = useState(false);

  const { t } = useTranslation(['settings', 'welcome']);
  const navigate = useNavigate();
  const { add } = useToast();

  const choices = {
    netstats: {
      checked: networkStats,
      pending: netStatsPending,
      setPending: setNetStatsPending,
      enable: 'enable_netstats',
      disable: 'disable_netstats',
      action: 'set-network-stats',
      label: 'network stats',
    },
    monitoring: {
      checked: monitoring,
      pending: monitoringPending,
      setPending: setMonitoringPending,
      enable: 'enable_sentry',
      disable: 'disable_sentry',
      action: 'set-monitoring',
      label: 'monitoring',
    },
  } as const;

  const saveChoice = async (name: keyof typeof choices) => {
    const choice = choices[name];
    if (choice.pending) {
      return;
    }
    choice.setPending(true);
    const isChecked = !choice.checked;
    try {
      await invoke(isChecked ? choice.enable : choice.disable);
      dispatch({ type: choice.action, enabled: isChecked });
    } catch (error) {
      console.error(`[data-privacy] ${choice.label} error`, error);
      add({
        title: t('privacy.errors.failed'),
        type: 'error',
      });
    } finally {
      choice.setPending(false);
    }
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
                onChange={() => saveChoice('netstats')}
                disabled={netStatsPending}
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
                onChange={() => saveChoice('monitoring')}
                disabled={monitoringPending}
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
