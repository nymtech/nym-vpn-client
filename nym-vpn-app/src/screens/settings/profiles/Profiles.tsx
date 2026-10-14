import { useRef } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { PROFILES, ProfilesLearnMoreUrl } from '../../../constants';
import { useSetProfile } from '../../../hooks';
import { Link, PageAnim, ProfileIcon } from '../../../ui';
import SettingsGroup from '../SettingsGroup';

function Profiles() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const setProfile = useSetProfile();
  // guards against a second click firing another `set_profile` + `navigate(-1)`
  // while the first selection is still in flight
  const isSelectingRef = useRef(false);

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <p className="text-text-secondary">{t('profiles.intro')}</p>
      <SettingsGroup
        settings={PROFILES.map((id) => ({
          title: t(`profiles.${id}.title`),
          desc: t(`profiles.${id}.desc`),
          leadingComponent: (
            <ProfileIcon
              profile={id}
              className="text-text-tertiary group-hover:animate-nod group-hover:text-text-primary group-active:animate-nod-deep group-active:text-brand-primary transition-colors"
            />
          ),
          onClick: async () => {
            if (isSelectingRef.current) {
              return;
            }
            isSelectingRef.current = true;
            const success = await setProfile(id);
            if (success) {
              navigate(-1);
            } else {
              isSelectingRef.current = false;
            }
          },
        }))}
      />
      <Link
        text={t('profiles.learnMore')}
        url={ProfilesLearnMoreUrl}
        color="primary"
        icon
      />
    </PageAnim>
  );
}

export default Profiles;
