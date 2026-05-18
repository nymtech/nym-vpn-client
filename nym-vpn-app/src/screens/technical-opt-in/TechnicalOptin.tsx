import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { AnonNetworkStatsUrl, PrivacyPolicyUrl } from '../../constants';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { ButtonNew, Link, Switch } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import SettingsGroup from '../settings/SettingsGroup';
import { dispatch, useAppStore } from '../../store';
import { InteractiveCard } from '../home/InteractiveCard';

const defaultSentry = window._APP.defaultSentry;
const defaultNetstats = window._APP.defaultNetstats;

function TechnicalOptin() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  const uiTheme = useAppStore((state) => state.uiTheme);
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
        <NymVpnTextLogo
          className={clsx(
            'h-6 w-24',
            uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
          )}
        />
        {/* Title & description */}
        <div className="space-y-2 text-center">
          <h1 className="text-text-primary text-2xl">{t('title')}</h1>
          <p className="text-bombay text-sm">{t('description')}</p>
        </div>

        {/* Buttons */}
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('anonymous-network-stats'),
              desc: (
                <Link
                  color="iron"
                  text={t('anonymous-network-stats-toggle-desc')}
                  url={AnonNetworkStatsUrl}
                />
              ),
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
            {
              title: t('error-and-crash-reports'),
              desc: (
                <Link
                  color="iron"
                  text={t('error-and-crash-reports-toggle-desc')}
                  url={PrivacyPolicyUrl}
                />
              ),
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
          ]}
          data-testid="welcome-netstats-group"
        />
        <ButtonNew onClick={handleContinue}>{t('continue-button')}</ButtonNew>
      </div>
    </InteractiveCard>
  );
}

export default TechnicalOptin;
