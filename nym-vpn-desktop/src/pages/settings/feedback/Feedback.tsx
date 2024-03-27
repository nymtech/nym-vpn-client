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
import { useMainDispatch, useMainState } from '../../../contexts';
import { kvSet } from '../../../kvStore';
import { StateDispatch } from '../../../types';
import { PageAnim, SettingsMenuCard, Snackbar, Switch } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets';

const ThrottleDelay = 10000; // ms

function Feedback() {
  const [snackIsOpen, setSnackIsOpen] = useState(false);

  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [monitoring, setMonitoring] = useState(state.monitoring);

  const { t } = useTranslation('settings');

  useEffect(() => {
    setMonitoring(state.monitoring);
  }, [state]);

  // notify the user at most once per every 10s when he toggles monitoring
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const showSnackbar = useCallback(
    _.throttle(() => setSnackIsOpen(true), ThrottleDelay, {
      leading: true,
      trailing: false,
    }),
    [],
  );

  const handleMonitoringChanged = async () => {
    showSnackbar();
    const isSelected = !state.monitoring;
    dispatch({ type: 'set-monitoring', monitoring: isSelected });
    kvSet('Monitoring', isSelected);
  };

  return (
    <>
      <Snackbar
        open={snackIsOpen}
        onClose={() => setSnackIsOpen(false)}
        text={t('feedback.monitoring-alert')}
        position="bottom"
        closeIcon
      />
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
    </>
  );
}

export default Feedback;
