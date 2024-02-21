import { open } from '@tauri-apps/api/shell';
import { useTranslation } from 'react-i18next';
import {
  DiscordInviteUrl,
  EmailSupportUrl,
  FaqUrl,
  MatrixRoomUrl,
} from '../../../constants';
import { SettingsMenuCard } from '../../../ui';
import { DiscordIcon, ElementIcon } from '../../../assets/icons';

function Support() {
  const { t } = useTranslation('settings');

  return (
    <div className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCard
        title={t('support.faq')}
        onClick={async () => open(FaqUrl)}
        leadingIcon="help"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('support.email')}
        onClick={async () => open(EmailSupportUrl)}
        leadingIcon="email"
        trailingIcon="arrow_right"
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
    </div>
  );
}

export default Support;
