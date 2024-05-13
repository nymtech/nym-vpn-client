import { open } from '@tauri-apps/api/shell';
import { useTranslation } from 'react-i18next';
import {
  DiscordInviteUrl,
  EmailSupportUrl,
  GitHubIssuesUrl,
  MatrixRoomUrl,
} from '../../../constants';
import { PageAnim, SettingsMenuCard } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets';

function Feedback() {
  const { t } = useTranslation('settings');

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
    </PageAnim>
  );
}

export default Feedback;
