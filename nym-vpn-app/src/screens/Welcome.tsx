import { useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { PrivacyPolicyUrl, SentryHomePage, ToSUrl } from '../constants';
import { useMainDispatch } from '../contexts';
import { kvSet } from '../kvStore';
import { routes } from '../router';
import { StateDispatch } from '../types';
import { Button, Link, PageAnim, Switch } from '../ui';
import SettingsGroup from './settings/SettingsGroup';

const defaultSentry = window._APP.defaultSentryEnabled;
const defaultNetstats = window._APP.defaultNetstatsEnabled;

function Welcome() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();
  const { t } = useTranslation('welcome');

  const handleContinue = () => {
    initMonitoring();
    initNetworkStats();
    dispatch({ type: 'set-welcome-checked', checked: true });
    kvSet('welcome-screen-seen', true);
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
    <PageAnim
      className="xs:max-w-lg h-full flex flex-col justify-end items-center gap-4 select-none cursor-default"
      data-testid="welcome-page"
    >
      <div
        className="grow flex flex-col justify-center items-center gap-4 px-4"
        data-testid="welcome-header"
      >
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white hsm:mt-24">
          <h1 className="truncate" data-testid="welcome-title">
            {t('title')}
          </h1>
        </div>
        <h2
          className="text-center dark:text-bombay w-72"
          data-testid="welcome-description"
        >
          <Trans
            i18nKey="description"
            ns="welcome"
            components={{
              sentryLink: (
                <Link
                  text={t('sentry', { ns: 'common' })}
                  url={SentryHomePage}
                  data-testid="welcome-sentry-link"
                />
              ),
            }}
          />
        </h2>
      </div>
      <div
        className="flex flex-col items-center gap-4 w-full"
        data-testid="welcome-content"
      >
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('error-monitoring-label'),
              desc: t('anon-toggle-desc'),
              leadingIcon: 'bug_report',
              onClick: toggleMonitoring,
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
              title: t('network-statistic'),
              leadingIcon: 'analytics',
              onClick: toggleNetstats,
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
        <Button
          className="mt-1"
          onClick={handleContinue}
          data-testid="welcome-continue-button"
        >
          {t('continue-button')}
        </Button>
        <p
          className="text-xs text-center text-iron dark:text-bombay w-80"
          data-testid="welcome-tos-notice"
        >
          <Trans
            i18nKey="tos-notice"
            ns="welcome"
            components={{
              tosLink: (
                <Link
                  text={t('tos', { ns: 'common' })}
                  url={ToSUrl}
                  className="text-black dark:text-white"
                  textClassName="underline-offset-2"
                  data-testid="welcome-tos-link"
                />
              ),
              privacyLink: (
                <Link
                  text={t('privacy-statement', { ns: 'common' })}
                  url={PrivacyPolicyUrl}
                  className="text-black dark:text-white"
                  textClassName="underline-offset-2"
                  data-testid="welcome-privacy-link"
                />
              ),
            }}
          />
        </p>
      </div>
    </PageAnim>
  );
}

export default Welcome;
