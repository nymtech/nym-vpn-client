import clsx from 'clsx';

export type MsIconProps = {
  // icon name
  icon: string;
  filled?: boolean;
  className?: string;
  'data-testid'?: string;
};

// Component for rendering Google Material Symbols icons
//  https://fonts.google.com/icons
function MsIcon({ icon, filled = false, className, ...rest }: MsIconProps) {
  const testId = rest['data-testid'] || `icon-${icon}`;

  return (
    <span
      className={clsx([
        'font-icon inline-block text-2xl select-none rtl:-scale-x-100',
        className && className,
      ])}
      style={filled ? { fontVariationSettings: "'FILL' 1" } : undefined}
      data-testid={testId}
      data-test-icon={icon}
    >
      {icon}
    </span>
  );
}

export default MsIcon;
