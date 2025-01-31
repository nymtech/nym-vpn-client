import clsx from 'clsx';

export type MsIconProps = {
  // icon name
  icon: string;
  className?: string;
};

// Component for rendering Google Material Symbols icons
//  https://fonts.google.com/icons
function MsIcon({ icon, className }: MsIconProps) {
  return (
    <span
      className={clsx([
        'font-icon text-2xl select-none',
        className && className,
      ])}
    >
      {icon}
    </span>
  );
}

export default MsIcon;
