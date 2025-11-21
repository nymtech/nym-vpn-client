import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';
import Switch from './Switch';

export type CardSwitchProps = {
  header: string | ReactNode;
  subheader?: string;
  subheaderColor?: 'default' | 'king-nacho';
  checked: boolean;
  onClick: () => void;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
  noHoverEffect?: boolean;
};

function CardSwitch({
  header,
  subheader,
  subheaderColor = 'default',
  checked,
  onClick,
  className,
  style,
  disabled,
  noHoverEffect,
}: CardSwitchProps) {
  const Header = () => (
    <div className="min-w-0 flex flex-col justify-center gap-1">
      <p className="truncate text-base text-baltic-sea dark:text-white select-none">
        {header}
      </p>
      {subheader && (
        <p
          className={clsx(
            'text-sm select-none',
            subheaderColor === 'default'
              ? 'text-iron dark:text-bombay'
              : 'text-cheddar dark:text-king-nacho',
          )}
        >
          {subheader}
        </p>
      )}
    </div>
  );

  return (
    <div
      className={clsx(
        'w-full flex flex-row justify-between items-center gap-4 select-none',
        'bg-white dark:bg-charcoal px-5 min-h-16 rounded-t-lg py-4',
        !noHoverEffect && 'hover:bg-iron/5 dark:hover:bg-black/10',
        'cursor-default',
        disabled && 'pointer-events-none',
        'overflow-hidden',
        className,
      )}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter') onClick?.();
      }}
      role="button"
      tabIndex={disabled ? -1 : 0}
      style={style}
    >
      {typeof header === 'string' ? <Header /> : header}
      <Switch
        checked={checked}
        onChange={onClick}
        disabled={disabled}
        className={clsx(
          'self-start',
          subheader && 'mt-2',
          disabled && 'opacity-50',
        )}
      />
    </div>
  );
}

export default CardSwitch;
