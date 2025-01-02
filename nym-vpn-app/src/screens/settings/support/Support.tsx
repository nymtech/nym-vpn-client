import { open } from '@tauri-apps/plugin-shell';
import { useTranslation } from 'react-i18next';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
  FaqUrl,
  GitHubIssuesUrl,
  MatrixRoomUrl,
} from '../../../constants';
import { PageAnim, SettingsMenuCard } from '../../../ui';
import { DiscordIcon, ElementIcon, GitHubIcon } from '../../../assets';

function Support() {
  const { t } = useTranslation('settings');

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCard
        title={t('support.faq')}
        onClick={() => {
          open(FaqUrl);
        }}
        leadingIcon="help"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('support.get-in-touch')}
        onClick={() => {
          open(ContactSupportUrl);
        }}
        leadingIcon="send"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('support.github')}
        onClick={() => {
          open(GitHubIssuesUrl);
        }}
        leadingComponent={
          <GitHubIcon className="w-6 h-7 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('support.matrix')}
        onClick={() => {
          open(MatrixRoomUrl);
        }}
        leadingComponent={
          <ElementIcon className="w-6 h-6 fill-baltic-sea dark:fill-mercury-pinkish" />
        }
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('support.discord')}
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

export default Support;
