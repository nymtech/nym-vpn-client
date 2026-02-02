import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';

export type SettingsMenuCardBigProps = {
  header: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
};

function SettingsMenuCardBig({
  header,
  footer,
  children,
  disabled,
  className,
  style,
}: SettingsMenuCardBigProps) {
  return (
    <div
      className={clsx([
        'flex flex-col justify-center items-center gap-4 select-none',
        'bg-white dark:bg-charcoal rounded-lg min-h-16',
        'transition cursor-default',
        disabled && 'opacity-50 pointer-events-none',
        className,
      ])}
      style={style}
    >
      {header}
      <div className={clsx('px-5 w-full', !footer && 'pb-4')}>{children}</div>
      {footer && footer}
    </div>
  );
}

export default SettingsMenuCardBig;
