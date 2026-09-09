import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { Menu } from '@base-ui-components/react';
import { PROFILES } from '../../constants';
import { Profile } from '../../types';
import { MsIcon, ProfileIcon } from '../../ui';
import { useAppStore } from '../../store';

type ProfileItemProps = {
  profile: Profile;
  onSelect?: (profile: Profile) => void;
};

const rampClasses =
  'text-text-tertiary group-hover:text-text-primary group-active:text-brand-primary transition-colors';

function ProfileItem({ profile, onSelect }: ProfileItemProps) {
  const { t } = useTranslation();

  return (
    <Menu.Item
      onClick={() => onSelect?.(profile)}
      className={clsx(
        'group flex cursor-default items-center gap-3 rounded-lg px-4 py-3 select-none first:rounded-t-2xl last:rounded-b-2xl',
        'hover:bg-black/5 dark:hover:bg-white/5',
      )}
    >
      <ProfileIcon
        profile={profile}
        className={clsx(
          rampClasses,
          'group-hover:animate-nod group-active:animate-nod-deep',
        )}
      />
      <div className="flex flex-col">
        <span className={clsx(rampClasses, 'text-base')}>
          {t(`profiles.${profile}.title`)}
        </span>
        <span className="text-text-tertiary text-sm">
          {t(`profiles.${profile}.desc`)}
        </span>
      </div>
    </Menu.Item>
  );
}

type ProfilesMenuProps = {
  onSelect?: (profile: Profile) => void;
};

function ProfilesMenu({ onSelect }: ProfilesMenuProps) {
  const { t } = useTranslation();
  const uiTheme = useAppStore((s) => s.uiTheme);

  return (
    <Menu.Root>
      <Menu.Trigger
        aria-label={t('profiles.title')}
        className={clsx(
          'flex h-10 w-10 items-center justify-center rounded-full transition-colors',
          'text-text-secondary hover:bg-surface-sunken',
          'focus-visible:outline focus-visible:-outline-offset-1',
        )}
      >
        <MsIcon icon="local_fire_department" className="text-3xl" />
      </Menu.Trigger>
      <Menu.Portal>
        {/* manually re-adding the theme class is required as the menu is
            rendered outside the main app container (using a portal) */}
        <Menu.Positioner
          className={clsx('z-50 outline-none', uiTheme === 'dark' && 'dark')}
          sideOffset={8}
          align="start"
        >
          <Menu.Popup
            className={clsx(
              'origin-(--transform-origin) rounded-2xl p-0 shadow-lg transition-[transform,scale,opacity] data-ending-style:scale-90 data-ending-style:opacity-0 data-starting-style:scale-90 data-starting-style:opacity-0',
              'outline-surface-bg bg-white shadow-gray-200',
              'dark:outline-text-secondary dark:bg-surface-elev dark:shadow-none dark:-outline-offset-1',
            )}
          >
            {PROFILES.map((profile) => (
              <ProfileItem
                key={profile}
                profile={profile}
                onSelect={onSelect}
              />
            ))}
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.Root>
  );
}

export default ProfilesMenu;
