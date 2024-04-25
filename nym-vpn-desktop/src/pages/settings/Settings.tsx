import { useCallback } from 'react';
import * as _ from 'lodash-es';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
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

const ThrottleDelay = 10000; // ms

function Settings() {
  const { entrySelector, autoConnect, version, monitoring } = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { push } = useNotifications();

  const handleEntrySelectorChange = async () => {
    const isChecked = !entrySelector;
    dispatch({ type: 'set-entry-selector', entrySelector: isChecked });
    kvSet('EntryLocationEnabled', isChecked);
  };

  const handleAutoConnectChanged = async () => {
    const isChecked = !autoConnect;
    dispatch({ type: 'set-auto-connect', autoConnect: isChecked });
    kvSet('Autoconnect', isChecked);
  };

  // notify the user at most once per every 10s when he toggles monitoring
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const showMonitoringAlert = useCallback(
    _.throttle(
      () => {
        push({
          text: t('monitoring-alert'),
          position: 'top',
          closeIcon: true,
        });
      },
      ThrottleDelay,
      {
        leading: true,
        trailing: false,
      },
    ),
    [],
  );

  const handleMonitoringChanged = async () => {
    const isChecked = !monitoring;
    showMonitoringAlert();
    dispatch({ type: 'set-monitoring', monitoring: isChecked });
    kvSet('Monitoring', isChecked);
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      {import.meta.env.APP_DISABLE_DATA_STORAGE !== 'true' && (
        <Button onClick={async () => navigate(routes.credential)}>
          {t('add-credential-button')}
        </Button>
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
        onClick={async () => {
          navigate(routes.display);
        }}
        leadingIcon="contrast"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('logs')}
        onClick={async () => {
          navigate(routes.logs);
        }}
        leadingIcon="sort"
        trailingIcon="arrow_right"
        disabled
      />
      <SettingsGroup
        settings={[
          {
            title: t('feedback.title'),
            leadingIcon: 'edit_note',
            onClick: async () => {
              navigate(routes.feedback);
            },
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
            onClick: async () => {
              navigate(routes.support);
            },
            trailing: (
              <MsIcon
                icon="arrow_right"
                className="dark:text-mercury-pinkish"
              />
            ),
          },
          {
            title: t('error-monitoring.title'),
            desc: t('error-monitoring.desc'),
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
        onClick={async () => {
          navigate(routes.legal);
        }}
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <div className="flex grow flex-col justify-end text-comet text-sm tracking-tight leading-tight mb-4">
        Version {version}
      </div>
    </PageAnim>
  );
}

export default Settings;
