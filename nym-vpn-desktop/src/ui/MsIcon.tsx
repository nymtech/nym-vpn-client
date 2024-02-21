import clsx from 'clsx';

type MsIconProps = {
  // icon name
  icon: string;
  style?: string;
};

// Component for rendering Google Material Symbols icons
//  https://fonts.google.com/icons
function MsIcon({ icon, style }: MsIconProps) {
  return (
    <span className={clsx(['font-icon text-2xl select-none', style && style])}>
      {icon}
    </span>
  );
}

export default MsIcon;
