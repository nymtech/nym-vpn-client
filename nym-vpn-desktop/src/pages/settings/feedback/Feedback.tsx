import { open } from '@tauri-apps/api/shell';
import { useEffect, useState } from 'react';
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
import { SettingsMenuCard, Snackbar, Switch } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets/icons';

const MaxNotifiedCount = 2;

function Feedback() {
  const [dialogIsOpen, setDialogIsOpen] = useState(false);

  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [monitoring, setMonitoring] = useState(state.monitoring);

  // the count of times the user has been notified by
  // "restart the application for the change to take effect"
  // whenever he's interacting with error monitoring switch
  const [notifiedCount, setNotifiedCount] = useState(0);

  const { t } = useTranslation('settings');

  useEffect(() => {
    setMonitoring(state.monitoring);
  }, [state]);

  const handleMonitoringChanged = async () => {
    if (notifiedCount < MaxNotifiedCount) {
      setDialogIsOpen(true);
      setNotifiedCount(notifiedCount + 1);
    }
    const isSelected = !state.monitoring;
    dispatch({ type: 'set-monitoring', monitoring: isSelected });
    kvSet('Monitoring', isSelected);
  };

  return (
    <>
      <Snackbar
        open={dialogIsOpen}
        onClose={() => setDialogIsOpen(false)}
        text={t('feedback.monitoring-alert')}
        position="bottom"
        closeIcon
      />
      <div className="h-full flex flex-col mt-2 gap-6">
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
          leadingIcon="error"
          onClick={handleMonitoringChanged}
          trailingComponent={
            <Switch checked={monitoring} onChange={handleMonitoringChanged} />
          }
        />
      </div>
    </>
  );
}

export default Feedback;
