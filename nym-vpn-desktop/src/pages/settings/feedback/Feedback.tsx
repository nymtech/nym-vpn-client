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
import { SettingsMenuCard, Switch } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets/icons';

function Feedback() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [monitoring, setMonitoring] = useState(state.monitoring);

  const { t } = useTranslation('settings');

  useEffect(() => {
    setMonitoring(state.monitoring);
  }, [state]);

  const handleMonitoringChanged = async () => {
    const isSelected = !state.monitoring;
    dispatch({ type: 'set-monitoring', monitoring: isSelected });
    kvSet('Monitoring', isSelected).catch((e) => {
      console.warn(e);
    });
  };

  return (
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
        title={t('feedback.error-report.title')}
        desc={t('feedback.error-report.desc')}
        leadingIcon="error"
        trailingComponent={
          <Switch
            checked={monitoring}
            onChange={handleMonitoringChanged}
            disabled
          />
        }
        disabled
      />
    </div>
  );
}

export default Feedback;
