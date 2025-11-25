import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { routes } from '../../../router';
import { PageAnim, SettingsMenuCard } from '../../../ui';

function Appearance() {
  const navigate = useNavigate();
  const { t } = useTranslation('settings');

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <SettingsMenuCard
        title={t('language', { ns: 'common' })}
        onClick={() => navigate(routes.lang)}
        leadingIcon="translate"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('display-theme')}
        onClick={() => navigate(routes.display)}
        leadingIcon="contrast"
        trailingIcon="arrow_right"
      />
    </PageAnim>
  );
}

export default Appearance;
