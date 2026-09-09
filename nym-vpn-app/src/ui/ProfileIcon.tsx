import { FunctionComponent, SVGProps } from 'react';
import clsx from 'clsx';
import {
  ProfileCompassIcon,
  ProfileDiceIcon,
  ProfileRabbitIcon,
  ProfileTurtleIcon,
} from '../assets/icons';
import { Profile } from '../types';

const icons: Record<Profile, FunctionComponent<SVGProps<SVGSVGElement>>> = {
  safest: ProfileCompassIcon,
  random: ProfileDiceIcon,
  mostPrivate: ProfileTurtleIcon,
  fastest: ProfileRabbitIcon,
};

// each glyph sits at its native dimensions inside a 28x28 viewBox, so a 28px
// box renders it at the size it has in Figma; 24px matches a `text-2xl` MsIcon
const sizes = {
  md: 'size-7',
  sm: 'size-6',
} as const;

export type ProfileIconProps = {
  profile: Profile;
  size?: keyof typeof sizes;
  className?: string;
  'data-testid'?: string;
};

// Component for rendering the icon of a connection profile
function ProfileIcon({
  profile,
  size = 'md',
  className,
  ...rest
}: ProfileIconProps) {
  const Icon = icons[profile];

  return (
    <Icon
      className={clsx([
        'inline-block shrink-0 select-none rtl:-scale-x-100',
        sizes[size],
        className && className,
      ])}
      data-testid={rest['data-testid'] || `icon-profile-${profile}`}
      data-test-icon={profile}
    />
  );
}

export default ProfileIcon;
