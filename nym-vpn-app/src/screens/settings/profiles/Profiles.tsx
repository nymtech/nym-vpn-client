import { useRef } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { PROFILES } from '../../../constants';
import { useSetProfile } from '../../../hooks';
import { MsIcon, PageAnim } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Profiles() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const setProfile = useSetProfile();
  // guards against a second click firing another `set_profile` + `navigate(-1)`
  // while the first selection is still in flight
  const isSelecting = useRef(false);

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <p className="text-text-secondary">{t('profiles.intro')}</p>
      <SettingsGroup
        settings={PROFILES.map(({ id, icon }) => ({
          title: t(`profiles.${id}.title`),
          desc: t(`profiles.${id}.desc`),
          leadingComponent: (
            <span className="font-icon text-text-secondary group-hover:animate-nod text-2xl select-none">
              {icon}
            </span>
          ),
          onClick: async () => {
            if (isSelecting.current) {
              return;
            }
            isSelecting.current = true;
            const success = await setProfile(id);
            if (success) {
              navigate(-1);
            } else {
              isSelecting.current = false;
            }
          },
          trailing: (
            <MsIcon icon="chevron_right" className="text-text-primary" />
          ),
        }))}
      />
    </PageAnim>
  );
}

export default Profiles;
