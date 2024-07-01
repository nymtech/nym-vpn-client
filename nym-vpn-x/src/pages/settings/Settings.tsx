import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import { useThrottle } from '../../hooks';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import {
  useMainDispatch,
  useMainState,
  useNotifications,
} from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { Button, MsIcon, PageAnim, SettingsMenuCard, Switch } from '../../ui';
import SettingsGroup from './SettingsGroup';
import { useEffect, useState } from 'react';
import { capFirst } from '../../helpers';

const ThrottleDelay = 10000; // ms

function Settings() {
  const {
    entrySelector,
    autoConnect,
    version,
    monitoring,
    daemonStatus,
    credentialExpiry,
  } = useMainState();

  const [hasValidCredential, setHasValidCredential] = useState(false);

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { push } = useNotifications();

  useEffect(() => {
    if (!credentialExpiry || dayjs().isAfter(credentialExpiry)) {
      setHasValidCredential(false);
    } else {
      setHasValidCredential(true);
    }
  }, [credentialExpiry]);

  const handleEntrySelectorChange = () => {
    const isChecked = !entrySelector;
    dispatch({ type: 'set-entry-selector', entrySelector: isChecked });
    kvSet('EntryLocationEnabled', isChecked);
  };

  const handleAutoConnectChanged = () => {
    const isChecked = !autoConnect;
    dispatch({ type: 'set-auto-connect', autoConnect: isChecked });
    kvSet('Autoconnect', isChecked);
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
    kvSet('Monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      {import.meta.env.APP_DISABLE_DATA_STORAGE !== 'true' &&
        !hasValidCredential && (
          <Button
            onClick={() => navigate(routes.credential)}
            disabled={
              import.meta.env.MODE !== 'dev-browser' && daemonStatus !== 'Ok'
            }
          >
            {t('add-credential-button')}
          </Button>
        )}
      {credentialExpiry && hasValidCredential && (
        <SettingsMenuCard
          title={t('credential.title')}
          desc={`${capFirst(dayjs().to(credentialExpiry, true))} ${t('left', { ns: 'glossary' })}`}
          leadingIcon="account_circle"
        />
      )}
      <SettingsGroup
        settings={[
          {
            title: t('auto-connect.title'),
            desc: t('auto-connect.desc'),
            leadingIcon: 'hdr_auto',
            disabled: true,
            onClick: handleAutoConnectChanged,
            trailing: (
              <Switch
                checked={autoConnect}
                onChange={handleAutoConnectChanged}
                disabled
              />
            ),
          },
          {
            title: t('entry-selector.title'),
            desc: t('entry-selector.desc'),
            leadingIcon: 'looks_two',
            onClick: handleEntrySelectorChange,
            trailing: (
              <Switch
                checked={entrySelector}
                onChange={handleEntrySelectorChange}
              />
            ),
          },
        ]}
      />
      <SettingsMenuCard
        title={t('display-theme')}
        onClick={() => navigate(routes.display)}
        leadingIcon="contrast"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('logs')}
        onClick={() => navigate(routes.logs)}
        leadingIcon="sort"
        trailingIcon="arrow_right"
        disabled
      />
      <SettingsGroup
        settings={[
          {
            title: t('feedback.title'),
            leadingIcon: 'edit_note',
            onClick: () => navigate(routes.feedback),
            trailing: (
              <MsIcon
                icon="arrow_right"
                className="dark:text-mercury-pinkish"
              />
            ),
          },
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
            title: t('error-monitoring.title'),
            desc: (
              <span>
                {`(${t('via', { ns: 'glossary' })} `}
                <span className="text-melon">
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
        title={t('legal.title')}
        onClick={() => navigate(routes.legal)}
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('quit')}
        onClick={() => {
          exit();
        }}
      />
      <div className="flex grow flex-col justify-end text-comet text-sm tracking-tight leading-tight mb-4">
        Version {version}
      </div>
    </PageAnim>
  );
}

export default Settings;
