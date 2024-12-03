import { open } from '@tauri-apps/plugin-shell';
import { useTranslation } from 'react-i18next';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
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
        onClick={() => {
          open(GitHubIssuesUrl);
        }}
        leadingComponent={
          <GitHubIcon className="w-6 h-7 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('feedback.get-in-touch')}
        leadingIcon="send"
        trailingIcon="arrow_right"
        onClick={() => {
          open(ContactSupportUrl);
        }}
      />
      <SettingsMenuCard
        title={t('feedback.matrix')}
        onClick={() => {
          open(MatrixRoomUrl);
        }}
        leadingComponent={
          <ElementIcon className="w-6 h-6 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('feedback.discord')}
        onClick={() => {
          open(DiscordInviteUrl);
        }}
        leadingComponent={
          <DiscordIcon className="w-6 h-6 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
    </PageAnim>
  );
}

export default Feedback;
