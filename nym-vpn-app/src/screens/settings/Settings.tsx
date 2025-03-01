import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { openPath, openUrl } from '@tauri-apps/plugin-opener';
import {
  useAutostart,
  useDesktopNotifications,
  useThrottle,
} from '../../hooks';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { useInAppNotify, useMainDispatch, useMainState } from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { Button, MsIcon, PageAnim, SettingsMenuCard, Switch } from '../../ui';
import { capFirst } from '../../util';
import { InfoData } from './info-data';
import SettingsGroup from './SettingsGroup';
import Logout from './Logout';

const ThrottleDelay = 10000; // ms

function Settings() {
  const {
    monitoring,
    daemonStatus,
    account,
    desktopNotifications,
    accountLinks,
  } = useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { push } = useInAppNotify();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const toggleDNotifications = useDesktopNotifications();
  const accountLoginUrl = accountLinks?.signIn;

  useEffect(() => {
    const checkAccount = async () => {
      try {
        const stored = await invoke<boolean | undefined>('is_account_stored');
        dispatch({ type: 'set-account', stored: stored || false });
      } catch (e) {
        console.warn('error checking stored account:', e);
      }
    };

    checkAccount();
  }, [dispatch]);

  const handleAutostartChanged = async () => {
    await toggleAutostart();
  };

  const handleGoToAccount = () => {
    if (accountLoginUrl) {
      openUrl(accountLoginUrl);
    }
  };

  // notify the user at most once per every 10s when he toggles monitoring
  const showMonitoringAlert = useThrottle(() => {
    push({
      text: t('monitoring-alert'),
      position: 'top',
      closeIcon: true,
    });
  }, ThrottleDelay);

  const handleMonitoringChanged = () => {
    const isChecked = !monitoring;
    showMonitoringAlert();
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('monitoring', isChecked);
  };

  const handleLogs = async () => {
    try {
      const logDir = await invoke<string | undefined>('log_dir');
      if (logDir) {
        await openPath(logDir);
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      {account ? (
        <SettingsMenuCard
          title={capFirst(t('account', { ns: 'glossary' }))}
          onClick={handleGoToAccount}
          leadingIcon="person"
          trailingIcon="open_in_new"
          disabled={!accountLoginUrl}
        />
      ) : (
        <Button
          onClick={() => navigate(routes.login)}
          disabled={
            import.meta.env.MODE !== 'dev-browser' && daemonStatus === 'down'
          }
        >
          {t('login-button')}
        </Button>
      )}
      <SettingsGroup
        settings={[
          {
            title: t('support.title'),
            leadingIcon: 'question_answer',
            onClick: () => navigate(routes.support),
            trailing: (
              <MsIcon
                icon="arrow_right"
                className="dark:text-mercury-pinkish"
              />
            ),
          },
          {
            title: t('logs.title'),
            desc: t('logs.desc'),
            leadingIcon: 'sort',
            onClick: handleLogs,
            trailing: (
              <MsIcon
                icon="open_in_new"
                className="dark:text-mercury-pinkish"
              />
            ),
          },
          {
            title: t('error-monitoring.title'),
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
              <Switch checked={monitoring} onChange={handleMonitoringChanged} />
            ),
          },
        ]}
      />
      <SettingsMenuCard
        title={t('autostart.title')}
        desc={t('autostart.desc')}
        leadingIcon="computer"
        onClick={handleAutostartChanged}
        trailingComponent={
          <Switch
            checked={autostartEnabled}
            onChange={handleAutostartChanged}
          />
        }
      />
      <SettingsGroup
        settings={[
          {
            title: t('appearance', { ns: 'common' }),
            leadingIcon: 'view_comfy',
            onClick: () => navigate(routes.appearance),
            trailing: (
              <MsIcon
                icon="arrow_right"
                className="dark:text-mercury-pinkish"
              />
            ),
          },
          {
            title: t('notifications.title'),
            leadingIcon: 'notifications',
            onClick: toggleDNotifications,
            trailing: (
              <Switch
                checked={desktopNotifications}
                onChange={toggleDNotifications}
              />
            ),
          },
        ]}
      />
      <SettingsMenuCard
        title={t('legal.title')}
        onClick={() => navigate(routes.legal)}
        trailingIcon="arrow_right"
      />
      <Logout />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <InfoData />
    </PageAnim>
  );
}

export default Settings;
