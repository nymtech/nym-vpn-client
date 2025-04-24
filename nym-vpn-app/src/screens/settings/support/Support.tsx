import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
  FaqUrl,
  GitHubIssuesUrl,
  MatrixRoomUrl,
  TelegramUrl,
} from '../../../constants';
import { PageAnim, SettingsMenuCard } from '../../../ui';
import {
  DiscordIcon,
  ElementIcon,
  GitHubIcon,
  TelegramIcon,
} from '../../../assets';

function Support() {
  const { t } = useTranslation('settings');

  return (
    <PageAnim
      className="h-full flex flex-col mt-2 gap-6"
      data-testid="support-page"
    >
      <SettingsMenuCard
        title={t('support.faq')}
        onClick={() => {
          openUrl(FaqUrl);
        }}
        leadingIcon="help"
        trailingIcon="open_in_new"
        data-testid="support-faq-button"
      />
      <SettingsMenuCard
        title={t('support.get-in-touch')}
        onClick={() => {
          openUrl(ContactSupportUrl);
        }}
        leadingIcon="send"
        trailingIcon="open_in_new"
        data-testid="support-contact-button"
      />
      <SettingsMenuCard
        title={t('support.telegram')}
        onClick={() => {
          openUrl(TelegramUrl);
        }}
        leadingComponent={
          <TelegramIcon className="w-6 h-6 fill-baltic-sea dark:fill-white" />
        }
        trailingIcon="open_in_new"
        data-testid="support-telegram-button"
      />
      <SettingsMenuCard
        title={t('support.matrix')}
        onClick={() => {
          openUrl(MatrixRoomUrl);
        }}
        leadingComponent={
          <ElementIcon className="w-6 h-6 fill-baltic-sea dark:fill-white" />
        }
        trailingIcon="open_in_new"
        data-testid="support-matrix-button"
      />
      <SettingsMenuCard
        title={t('support.discord')}
        onClick={() => {
          openUrl(DiscordInviteUrl);
        }}
        leadingComponent={
          <DiscordIcon className="w-6 h-6 fill-baltic-sea dark:fill-white" />
        }
        trailingIcon="open_in_new"
        data-testid="support-discord-button"
      />
      <SettingsMenuCard
        title={t('support.github')}
        onClick={() => {
          openUrl(GitHubIssuesUrl);
        }}
        leadingComponent={
          <GitHubIcon className="w-6 h-7 fill-baltic-sea dark:fill-white" />
        }
        trailingIcon="open_in_new"
        data-testid="support-github-button"
      />
    </PageAnim>
  );
}

export default Support;
