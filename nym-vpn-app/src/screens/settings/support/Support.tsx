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
      className="h-full flex flex-col mt-2 gap-6"
      data-testid="support-page"
    >
      <div>
        <p className="truncate text-base select-none text-baltic-sea dark:text-white">
          {t('support.intro.title')}
        </p>
        <p className="text-sm whitespace-pre-line mt-4 text-iron dark:text-bombay">
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
            leadingComponent: <GitHubIcon className="w-6 h-6 fill-bombay" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.matrix'),
            onClick: () => {
              openUrl(MatrixRoomUrl);
            },
            leadingComponent: <ElementIcon className="w-6 h-6 fill-bombay" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.discord'),
            onClick: () => {
              openUrl(DiscordInviteUrl);
            },
            leadingComponent: <DiscordIcon className="w-6 h-6 fill-bombay" />,
            trailingIcon: 'open_in_new',
          },
          {
            title: t('support.telegram'),
            onClick: () => {
              openUrl(TelegramUrl);
            },
            leadingComponent: <TelegramIcon className="w-6 h-6 fill-bombay" />,
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
