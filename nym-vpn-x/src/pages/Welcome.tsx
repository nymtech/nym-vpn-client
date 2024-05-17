import { invoke } from '@tauri-apps/api';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { useNavigate } from 'react-router-dom';
import { NymDarkOutlineIcon, NymIcon } from '../assets';
import { useMainDispatch, useMainState, useNotifications } from '../contexts';
import { kvSet } from '../kvStore';
import { routes } from '../router';
import { CmdError, StateDispatch } from '../types';
import { Button, PageAnim, Switch } from '../ui';
import SettingsGroup from './settings/SettingsGroup';

function Welcome() {
  const { uiTheme, monitoring } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useNotifications();
  const navigate = useNavigate();
  const { t } = useTranslation('welcome');

  const handleClick = async () => {
    navigate(routes.root);
  };

  const showMonitoringAlert = () => {
    push({
      text: t('monitoring-alert', { ns: 'settings' }),
      position: 'top',
      closeIcon: true,
    });
  };

  const handleMonitoringChanged = async () => {
    const isChecked = !monitoring;
    showMonitoringAlert();
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('Monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-14 select-none cursor-default">
      <div className="flex flex-col items-center gap-4 px-4">
        {uiTheme === 'Dark' ? (
          <NymDarkOutlineIcon className="w-28 h-28" />
        ) : (
          <NymIcon className="w-28 h-28 fill-ghost" />
        )}
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white">
          <h1 className="truncate">{t('title.part1')}</h1>
          <h1 className="truncate">{t('title.part2')}</h1>
        </div>
        <h2 className="text-center dark:text-laughing-jack w-72">
          {`${t('description.part1')} `}
          <span className="underline">{t('description.part2')}</span>
          {` ${t('description.part3')}`}
          <span className="text-melon">{` ${t('sentry', { ns: 'common' })}`}</span>
          {t('description.part4')}
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
              desc: t('error-monitoring.desc', { ns: 'settings' }),
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
        <Button className="mt-1" onClick={handleClick}>
          {t('continue-button')}
        </Button>
        <p className="text-xs text-center text-dim-gray dark:text-mercury-mist w-80">
          {t('tos.part1')}
          <span className="dark:text-white">{` ${t('tos', { ns: 'common' })} `}</span>
          {t('tos.part2')}
          <span className="dark:text-white">{` ${t('privacy-statement', { ns: 'common' })}`}</span>
          {t('tos.part3')}
        </p>
      </div>
    </PageAnim>
  );
}

export default Welcome;
