import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { PrivacyPolicyUrl, ToSUrl } from '../../../constants';
import { routes } from '../../../router';
import { MsIcon, PageAnim } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Legal() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('legal.tos'),
            onClick: () => {
              openUrl(ToSUrl);
            },
            trailing: <MsIcon icon="open_in_new" />,
          },
          {
            title: t('legal.policy'),
            onClick: () => {
              openUrl(PrivacyPolicyUrl);
            },
            trailing: <MsIcon icon="open_in_new" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('legal.licenses-rust'),
            onClick: () => {
              navigate(routes.licensesRust);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
          {
            title: t('legal.licenses-js'),
            onClick: () => {
              navigate(routes.licensesJs);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
        ]}
      />
    </PageAnim>
  );
}

export default Legal;
