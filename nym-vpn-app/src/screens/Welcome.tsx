import { useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { SentryHomePage } from '../constants';
import { useMainDispatch, useMainState } from '../contexts';
import { kvSet } from '../kvStore';
import { routes } from '../router';
import { StateDispatch } from '../types';
import { Button, Link, PageAnim, Switch } from '../ui';
import { NymSplash } from '../assets/index';
import SettingsGroup from './settings/SettingsGroup';

const defaultSentry = window._APP.defaultSentry;
const defaultNetstats = window._APP.defaultNetstats;

function Welcome() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  const { uiTheme } = useMainState();
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
      className="h-full flex flex-col justify-end items-center gap-4 select-none cursor-default"
      data-testid="welcome-page"
    >
      <div
        className="grow flex flex-col justify-center items-center gap-4 px-4"
        data-testid="welcome-header"
      >
        <div className="flex flex-col items-center gap-8 text-2xl text-center dark:text-white hsm:mt-24">
          <NymSplash
            className={clsx(
              'w-32',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
          <h1 className="truncate" data-testid="welcome-title">
            {t('title')}
          </h1>
        </div>
        <h2
          className="text-center dark:text-bombay w-80"
          data-testid="welcome-description"
        >
          {t('description')}
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
              desc: (
                <span className="whitespace-normal">
                  <Trans
                    i18nKey="anon-toggle-desc"
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
                </span>
              ),
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
              desc: (
                <span className="whitespace-normal">
                  <Trans
                    i18nKey="network-statistic-toggle-desc"
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
                </span>
              ),
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
      </div>
    </PageAnim>
  );
}

export default Welcome;
