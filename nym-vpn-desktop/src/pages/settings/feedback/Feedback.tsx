import { open } from '@tauri-apps/api/shell';
import * as _ from 'lodash-es';
import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  DiscordInviteUrl,
  EmailSupportUrl,
  GitHubIssuesUrl,
  MatrixRoomUrl,
} from '../../../constants';
import {
  useMainDispatch,
  useMainState,
  useNotifications,
} from '../../../contexts';
import { kvSet } from '../../../kvStore';
import { StateDispatch } from '../../../types';
import { PageAnim, SettingsMenuCard, Switch } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets';

const ThrottleDelay = 1000; // ms

function Feedback() {
  const state = useMainState();
  const [monitoring, setMonitoring] = useState(state.monitoring);
  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useNotifications();

  const { t } = useTranslation('settings');

  useEffect(() => {
    setMonitoring(state.monitoring);
  }, [state]);

  // notify the user at most once per every 10s when he toggles monitoring
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const showSnackbar = useCallback(
    _.throttle(
      () => {
        push({
          text: t('feedback.monitoring-alert'),
          position: 'bottom',
          closeIcon: true,
        });
      },
      ThrottleDelay,
      {
        leading: true,
        trailing: false,
      },
    ),
    [push],
  );

  const handleMonitoringChanged = async () => {
    showSnackbar();
    const isSelected = !state.monitoring;
    dispatch({ type: 'set-monitoring', monitoring: isSelected });
    kvSet('Monitoring', isSelected);
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCard
        title={t('feedback.github')}
        onClick={async () => open(GitHubIssuesUrl)}
        leadingComponent={
          <GitHubIcon className="w-6 h-7 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('feedback.send')}
        leadingIcon="send"
        trailingIcon="arrow_right"
        onClick={async () => open(EmailSupportUrl)}
      />
      <SettingsMenuCard
        title={t('feedback.matrix')}
        onClick={async () => open(MatrixRoomUrl)}
        leadingComponent={
          <ElementIcon className="w-6 h-6 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('feedback.discord')}
        onClick={async () => open(DiscordInviteUrl)}
        leadingComponent={
          <DiscordIcon className="w-6 h-6 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('feedback.error-monitoring.title')}
        desc={t('feedback.error-monitoring.desc')}
        leadingIcon="bug_report"
        onClick={handleMonitoringChanged}
        trailingComponent={
          <Switch checked={monitoring} onChange={handleMonitoringChanged} />
        }
      />
    </PageAnim>
  );
}

export default Feedback;
