import { useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { PrivacyPolicyUrl, SentryHomePage, ToSUrl } from '../constants';
import { useMainDispatch } from '../contexts';
import { kvSet } from '../kvStore';
import { routes } from '../router';
import { StateDispatch } from '../types';
import { Button, Link, PageAnim, Switch } from '../ui';
import SettingsGroup from './settings/SettingsGroup';

function Welcome() {
  const [monitoring, setMonitoring] = useState<boolean>(false);
  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();
  const { t } = useTranslation('welcome');

  const handleContinue = () => {
    kvSet('welcome-screen-seen', true).then(() => {
      navigate(routes.root);
    });
  };

  const handleMonitoringChanged = () => {
    const isChecked = !monitoring;
    setMonitoring(isChecked);
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-14 select-none cursor-default">
      <div className="flex flex-col items-center gap-4 px-4 mt-4">
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white">
          <h1 className="truncate">{t('title')}</h1>
        </div>
        <h2 className="text-center dark:text-bombay w-72">
          <Trans
            i18nKey="description"
            ns="welcome"
            components={{
              sentryLink: (
                <Link
                  text={t('sentry', { ns: 'common' })}
                  url={SentryHomePage}
                />
              ),
            }}
          />
        </h2>
      </div>
      <div className="flex flex-col items-center gap-4 w-full">
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('error-monitoring.title', { ns: 'settings' }),
              desc: t('anon-toggle-desc'),
              leadingIcon: 'bug_report',
              onClick: handleMonitoringChanged,
              trailing: (
                <Switch
                  checked={monitoring}
                  onChange={handleMonitoringChanged}
                />
              ),
            },
          ]}
        />
        <Button className="mt-1" onClick={handleContinue}>
          {t('continue-button')}
        </Button>
        <p className="text-xs text-center text-iron dark:text-bombay w-80">
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
                />
              ),
              privacyLink: (
                <Link
                  text={t('privacy-statement', { ns: 'common' })}
                  url={PrivacyPolicyUrl}
                  className="text-black dark:text-white"
                  textClassName="underline-offset-2"
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
