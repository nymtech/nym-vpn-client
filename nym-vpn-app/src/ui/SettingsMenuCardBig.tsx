import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';

export type SettingsMenuCardBigProps = {
  header: ReactNode;
  children: ReactNode;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
};

function SettingsMenuCardBig({
  header,
  children,
  disabled,
  className,
  style,
}: SettingsMenuCardBigProps) {
  return (
    <div
      className={clsx([
        'flex flex-col items-center justify-center gap-4 select-none',
        'dark:bg-charcoal min-h-16 rounded-lg bg-white',
        'cursor-default transition',
        disabled && 'pointer-events-none opacity-50',
        className,
      ])}
      style={style}
    >
      {header}
      <div className="w-full px-5 pb-4">{children}</div>
    </div>
  );
}

export default SettingsMenuCardBig;
