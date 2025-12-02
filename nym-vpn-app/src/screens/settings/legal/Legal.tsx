import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { PrivacyPolicyUrl, ToSUrl } from '../../../constants';
import { routes } from '../../../router';
import { PageAnim } from '../../../ui';
import SettingsGroup from '../SettingsGroup';
import { useMainState } from '../../../contexts';

function Legal() {
  const { t } = useTranslation('settings');
  const { codeDepsJs, codeDepsRust } = useMainState();
  const navigate = useNavigate();
  const licensesAvailable = codeDepsJs.length > 0 || codeDepsRust.length > 0;

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('legal.tos'),
            onClick: () => {
              openUrl(ToSUrl);
            },
            trailingIcon: 'open_in_new',
          },
          {
            title: t('legal.policy'),
            onClick: () => {
              openUrl(PrivacyPolicyUrl);
            },
            trailingIcon: 'open_in_new',
          },
        ]}
      />
      {licensesAvailable && (
        <SettingsGroup
          settings={[
            codeDepsRust.length > 0 && {
              title: t('legal.licenses-rust'),
              onClick: () => {
                navigate(routes.licensesRust);
              },
              trailingIcon: 'arrow_right',
            },
            codeDepsJs.length > 0 && {
              title: t('legal.licenses-js'),
              onClick: () => {
                navigate(routes.licensesJs);
              },
              trailingIcon: 'arrow_right',
            },
          ]}
        />
      )}
    </PageAnim>
  );
}

export default Legal;
