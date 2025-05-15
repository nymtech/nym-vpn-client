import clsx from 'clsx';

export type MsIconProps = {
  // icon name
  icon: string;
  className?: string;
  'data-testid'?: string;
};

// Component for rendering Google Material Symbols icons
//  https://fonts.google.com/icons
function MsIcon({ icon, className, ...rest }: MsIconProps) {
  const testId = rest['data-testid'] || `icon-${icon}`;

  return (
    <span
      className={clsx([
        'font-icon text-2xl select-none',
        className && className,
      ])}
      data-testid={testId}
      data-icon={icon}
    >
      {icon}
    </span>
  );
}

export default MsIcon;
