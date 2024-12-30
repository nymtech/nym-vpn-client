import { open } from '@tauri-apps/plugin-shell';
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
              open(ToSUrl);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
          {
            title: t('legal.policy'),
            onClick: () => {
              open(PrivacyPolicyUrl);
            },
            trailing: <MsIcon icon="arrow_right" />,
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
