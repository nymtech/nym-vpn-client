import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
  FaqUrl,
  GitHubIssuesUrl,
  MatrixRoomUrl,
  TelegramUrl,
  TranslationHelpUrl,
} from '../../../constants';
import { PageAnim } from '../../../ui';
import {
  DiscordIcon,
  ElementIcon,
  GitHubIcon,
  TelegramIcon,
} from '../../../assets';
import SettingsGroup from '../SettingsGroup';

function Support() {
  const { t } = useTranslation('settings');

  return (
    <PageAnim
      className="mt-2 flex h-full flex-col gap-6"
      data-testid="support-page"
    >
      <div>
        <p className="text-text-primary truncate text-base select-none">
          {t('support.intro.title')}
        </p>
        <p className="text-text-secondary mt-4 text-sm whitespace-pre-line">
          {t('support.intro.description')}
        </p>
      </div>
      <SettingsGroup
        settings={[
          {
            title: t('support.faq'),
            onClick: () => {
              openUrl(FaqUrl);
            },
            leadingIcon: 'help',
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.contact'),
            onClick: () => {
              openUrl(ContactSupportUrl);
            },
            leadingIcon: 'mail_outline',
            trailingIcon: 'open_in_new',
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('support.github'),
            onClick: () => {
              openUrl(GitHubIssuesUrl);
            },
            leadingComponent: <GitHubIcon className="fill-bombay h-6 w-6" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.matrix'),
            onClick: () => {
              openUrl(MatrixRoomUrl);
            },
            leadingComponent: <ElementIcon className="fill-bombay h-6 w-6" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.discord'),
            onClick: () => {
              openUrl(DiscordInviteUrl);
            },
            leadingComponent: <DiscordIcon className="fill-bombay h-6 w-6" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.telegram'),
            onClick: () => {
              openUrl(TelegramUrl);
            },
            leadingComponent: <TelegramIcon className="fill-bombay h-6 w-6" />,
            trailingIcon: 'open_in_new',
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('support.help.title'),
            desc: t('support.help.description'),
            leadingIcon: 'language',
            trailingIcon: 'open_in_new',
            onClick: () => {
              openUrl(TranslationHelpUrl);
            },
          },
        ]}
      />
    </PageAnim>
  );
}

export default Support;
