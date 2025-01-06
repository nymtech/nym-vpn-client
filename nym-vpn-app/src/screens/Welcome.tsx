import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
    kvSet('WelcomeScreenSeen', true).then(() => {
      navigate(routes.root);
    });
  };

  const handleMonitoringChanged = () => {
    const isChecked = !monitoring;
    setMonitoring(isChecked);
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('Monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-14 select-none cursor-default">
      <div className="flex flex-col items-center gap-4 px-4 mt-4">
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white">
          <h1 className="truncate">{t('title.part1')}</h1>
          <h1 className="truncate">{t('title.part2')}</h1>
        </div>
        <h2 className="text-center dark:text-laughing-jack w-72">
          {`${t('description.part1')} `}
          <span className="underline">{t('description.part2')}</span>
          {` ${t('description.part3')} (${t('via', { ns: 'glossary' })} `}
          <Link
            text={t('sentry', { ns: 'common' })}
            url={SentryHomePage}
            className=""
          />
          {`) ${t('description.part4')}`}
        </h2>
        <p className="text-xs text-center text-dim-gray dark:text-mercury-mist w-80">
          {t('experimental')}
        </p>
      </div>
      <div className="flex flex-col items-center gap-4 w-full">
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('error-monitoring.title', { ns: 'settings' }),
              desc: (
                <span>
                  {`(${t('via', { ns: 'glossary' })} `}
                  <span className="text-malachite-moss dark:text-malachite">
                    {t('sentry', { ns: 'common' })}
                  </span>
                  {`), ${t('error-monitoring.desc', { ns: 'settings' })}`}
                </span>
              ),
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
        <p className="text-xs text-center text-dim-gray dark:text-mercury-mist w-80">
          {`${t('tos.part1')} `}
          <Link
            text={t('tos', { ns: 'common' })}
            url={ToSUrl}
            className="text-black dark:text-white"
            textClassName="underline-offset-2"
          />
          {` ${t('tos.part2')} `}
          <Link
            text={t('privacy-statement', { ns: 'common' })}
            url={PrivacyPolicyUrl}
            className="text-black dark:text-white"
            textClassName="underline-offset-2"
          />
          {t('tos.part3')}
        </p>
      </div>
    </PageAnim>
  );
}

export default Welcome;
