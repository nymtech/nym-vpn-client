import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useAutostart, useDesktopNotifications } from '../../hooks';
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

    if (daemonStatus !== 'down') {
      checkAccount();
    }
  }, [daemonStatus, dispatch]);

  const handleAutostartChanged = async () => {
    await toggleAutostart();
  };

  const handleGoToAccount = () => {
    if (accountLoginUrl) {
      openUrl(accountLoginUrl);
    }
  };

  // notify the user at most once per every 10s when he toggles monitoring
  const showMonitoringAlert = () => {
    push({
      id: 'monitoring-alert',
      message: t('monitoring-alert'),
      close: true,
      type: 'warn',
      throttle: 10,
    });
  };

  const handleMonitoringChanged = () => {
    const isChecked = !monitoring;
    showMonitoringAlert();
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6" data-test-id="settings-page">
      {account ? (
        <SettingsMenuCard
          title={capFirst(t('account', { ns: 'glossary' }))}
          onClick={handleGoToAccount}
          leadingIcon="person"
          trailingIcon="open_in_new"
          disabled={!accountLoginUrl}
          data-test-id="account-button"
        />
      ) : (
        <Button
          onClick={() => navigate(routes.login)}
          disabled={daemonStatus === 'down'}
          data-test-id="login-button"
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
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
            'data-test-id': 'support-button',
          },
          {
            title: t('logs.title'),
            desc: t('logs.desc'),
            leadingIcon: 'sort',
            onClick: () => navigate(routes.logs),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
            'data-test-id': 'logs-button',
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
              <Switch 
                checked={monitoring} 
                onChange={handleMonitoringChanged}
                data-test-id="error-monitoring-switch" 
              />
            ),
            'data-test-id': 'error-monitoring-option',
          },
        ]}
        data-test-id="support-settings-group"
      />
      <SettingsGroup
        settings={[
          {
            title: t('autostart.title'),
            desc: t('autostart.desc'),
            leadingIcon: 'computer',
            onClick: handleAutostartChanged,
            trailing: (
              <Switch
                checked={autostartEnabled}
                onChange={handleAutostartChanged}
                data-test-id="autostart-switch"
              />
            ),
            'data-test-id': 'autostart-option',
          },
          {
            title: t('killswitch.title'),
            desc: t('killswitch.desc'),
            leadingIcon: 'power_settings_new',
            onClick: () => {
              /**/
            },
            trailing: (
              <Switch
                checked={true}
                onChange={() => {
                  /* */
                }}
                disabled
                data-test-id="killswitch-switch"
              />
            ),
            disabled: true,
            'data-test-id': 'killswitch-option',
          },
        ]}
        data-test-id="system-settings-group"
      />
      <SettingsGroup
        settings={[
          {
            title: t('appearance', { ns: 'common' }),
            leadingIcon: 'view_comfy',
            onClick: () => navigate(routes.appearance),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
            'data-test-id': 'appearance-button',
          },
          {
            title: t('notifications.title'),
            leadingIcon: 'notifications',
            onClick: toggleDNotifications,
            trailing: (
              <Switch
                checked={desktopNotifications}
                onChange={toggleDNotifications}
                data-test-id="notifications-switch"
              />
            ),
            'data-test-id': 'notifications-option',
          },
        ]}
        data-test-id="appearance-settings-group"
      />
      <SettingsMenuCard
        title={t('legal.title')}
        onClick={() => navigate(routes.legal)}
        trailingIcon="arrow_right"
        data-test-id="legal-button"
      />
      <Logout />
      <SettingsMenuCard 
        title={t('quit')} 
        onClick={exit}
        data-test-id="quit-button" 
      />
      <InfoData />
    </PageAnim>
  );
}

export default Settings;