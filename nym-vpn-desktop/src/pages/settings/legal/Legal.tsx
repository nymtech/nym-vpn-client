import { open } from '@tauri-apps/api/shell';
import { useTranslation } from 'react-i18next';
import { PrivacyPolicyUrl, ToSUrl } from '../../../constants';
import { useExit } from '../../../state';
import { MsIcon, SettingsMenuCard } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Legal() {
  const { t } = useTranslation('settings');
  const { exit } = useExit();

  return (
    <div className="h-full flex flex-col mt-2 gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('legal.tos'),
            onClick: async () => open(ToSUrl),
            trailing: <MsIcon icon="arrow_right" />,
          },
          {
            title: t('legal.policy'),
            onClick: async () => open(PrivacyPolicyUrl),
            trailing: <MsIcon icon="arrow_right" />,
          },
        ]}
      />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
    </div>
  );
}

export default Legal;
