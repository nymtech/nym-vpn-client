import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { routes } from '../../../router';
import { PageAnim } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Appearance() {
  const navigate = useNavigate();
  const { t } = useTranslation('settings');

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('language', { ns: 'common' }),
            leadingIcon: 'translate',
            onClick: () => navigate(routes.lang),
            trailingIcon: 'arrow_right',
          },
          {
            title: t('display-theme'),
            leadingIcon: 'contrast',
            onClick: () => navigate(routes.display),
            trailingIcon: 'arrow_right',
          },
        ]}
      />
    </PageAnim>
  );
}

export default Appearance;
