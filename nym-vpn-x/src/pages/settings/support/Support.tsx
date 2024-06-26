import { open } from '@tauri-apps/api/shell';
import { useTranslation } from 'react-i18next';
import {
  ContactSupportUrl,
  DiscordInviteUrl,
  FaqUrl,
  MatrixRoomUrl,
} from '../../../constants';
import { PageAnim, SettingsMenuCard } from '../../../ui';
import { DiscordIcon, ElementIcon } from '../../../assets';

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
        title={t('support.contact')}
        onClick={() => {
          open(ContactSupportUrl);
        }}
        leadingIcon="email"
        trailingIcon="arrow_right"
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

export default Support;
