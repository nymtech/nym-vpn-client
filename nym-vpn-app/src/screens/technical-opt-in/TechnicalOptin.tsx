import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { PrivacyPolicyUrl } from '../../constants';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { Button, Link, Switch } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import SettingsGroup from '../settings/SettingsGroup';
import { dispatch } from '../../store';
import { InteractiveCard } from '../home/InteractiveCard';
import TelemetryInfoDialog, {
  InfoButton,
  TelemetryInfoKind,
} from './TelemetryInfoDialog';

const defaultSentry = window._APP.defaultSentry;
const defaultNetstats = window._APP.defaultNetstats;

function TechnicalOptin() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  const [info, setInfo] = useState<TelemetryInfoKind | null>(null);
  const navigate = useNavigate();
  const { t } = useTranslation('welcome');

  const handleContinue = () => {
    initMonitoring();
    initNetworkStats();
    dispatch({ type: 'set-technical-optin-seen', seen: true });
    kvSet('technical-optin-seen', true);
    navigate(routes.root);
  };

  const toggleMonitoring = () => {
    setMonitoring(!monitoring);
  };

  const toggleNetstats = () => {
    setNetstats(!netstats);
  };

  const initMonitoring = async () => {
    dispatch({ type: 'set-monitoring', enabled: monitoring });
    // sentry is disabled by default
    if (monitoring) {
      try {
        await invoke('enable_sentry');
      } catch {}
    }
  };

  const initNetworkStats = async () => {
    dispatch({ type: 'set-network-stats', enabled: netstats });
    try {
      if (netstats) {
        await invoke('enable_netstats');
      } else {
        await invoke('disable_netstats');
      }
    } catch {}
  };

  return (
    <InteractiveCard>
      <div className="flex flex-col items-center justify-center gap-4">
        <NymVpnTextLogo className="fill-text-primary h-6 w-24" />
        {/* Title & description */}
        <div className="space-y-2 text-center">
          <h1 className="text-text-primary text-2xl">{t('title')}</h1>
          <p className="text-text-tertiary text-sm whitespace-pre-line">
            {t('description')}
          </p>
          <Link
            color="iron"
            text={t('privacy-statement-link')}
            url={PrivacyPolicyUrl}
          />
        </div>

        {/* Opt-in toggles */}
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('connection.label'),
              titleTrailing: (
                <InfoButton
                  kind="connection"
                  label={t('connection.label')}
                  onOpen={setInfo}
                  data-testid="welcome-netstats-info"
                />
              ),
              desc: t('connection.desc'),
              leadingIcon: 'analytics',
              trailing: (
                <Switch
                  checked={netstats}
                  onChange={toggleNetstats}
                  data-testid="welcome-netstats-switch"
                />
              ),
              'data-testid': 'welcome-netstats-option',
            },
            {
              title: t('monitoring.label'),
              titleTrailing: (
                <InfoButton
                  kind="monitoring"
                  label={t('monitoring.label')}
                  onOpen={setInfo}
                  data-testid="welcome-monitoring-info"
                />
              ),
              desc: t('monitoring.desc'),
              leadingIcon: 'bug_report',
              trailing: (
                <Switch
                  checked={monitoring}
                  onChange={toggleMonitoring}
                  data-testid="welcome-monitoring-switch"
                />
              ),
              'data-testid': 'welcome-monitoring-option',
            },
          ]}
          data-testid="welcome-optin-group"
        />
        <Button onClick={handleContinue}>{t('continue-button')}</Button>
      </div>

      <TelemetryInfoDialog kind={info} onClose={() => setInfo(null)} />
    </InteractiveCard>
  );
}

export default TechnicalOptin;
