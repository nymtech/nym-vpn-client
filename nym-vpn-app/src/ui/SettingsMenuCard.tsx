import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';
import MsIcon from './MsIcon';

export type SettingsMenuCardProps = {
  title: string;
  leadingIcon?: string;
  leadingComponent?: ReactNode;
  description?: string;
  descriptionColor?: 'normal' | 'yellow' | 'red';
  onClick?: () => void;
  trailingIcon?: string;
  trailingComponent?: ReactNode;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
  noHoverEffect?: boolean;
  color?: 'normal' | 'red' | 'gray';
};

function SettingsMenuCard({
  title,
  leadingIcon,
  leadingComponent,
  description,
  descriptionColor = 'normal',
  onClick,
  trailingIcon,
  trailingComponent,
  disabled,
  className,
  style,
  noHoverEffect,
  color = 'normal',
}: SettingsMenuCardProps) {
  return (
    <div
      className={clsx([
        // normal color
        color === 'normal' && 'dark:bg-charcoal bg-white',
        color === 'normal' &&
          !noHoverEffect &&
          'dark:hover:bg-charcoal/85 hover:bg-white/60',
        // red color
        color === 'red' &&
          'border-aphrodisiac bg-aphrodisiac/10 dark:bg-aphrodisiac/10 border',
        color === 'red' &&
          !noHoverEffect &&
          'hover:bg-aphrodisiac/20 dark:hover:bg-aphrodisiac/20',
        // gray color
        color === 'gray' && 'dark:bg-mine-shaft bg-white',
        color === 'gray' &&
          !noHoverEffect &&
          'dark:hover:bg-mine-shaft/85 hover:bg-white/60',
        'flex flex-row items-center justify-between gap-4 select-none',
        'min-h-16 rounded-lg px-5',
        description ? 'py-2' : 'py-4',
        'cursor-default transition',
        disabled && 'pointer-events-none opacity-50',
        className,
      ])}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter') onClick?.();
      }}
      role="button"
      tabIndex={disabled ? -1 : 0}
      style={style}
    >
      <div
        className={clsx(
          'flex flex-row items-center justify-between gap-4 overflow-hidden',
        )}
      >
        {leadingIcon && (
          <MsIcon icon={leadingIcon} className="text-text-secondary" />
        )}
        {leadingComponent && <div>{leadingComponent}</div>}
        <div className="flex min-w-0 flex-col justify-center">
          <p className="text-text-primary truncate text-base select-none">
            {title}
          </p>
          {description && (
            <p
              className={clsx(
                'truncate text-sm select-none',
                descriptionColor === 'normal' && 'text-text-secondary',
                descriptionColor === 'red' && 'text-aphrodisiac',
                descriptionColor === 'yellow' && 'text-king-nacho',
              )}
            >
              {description}
            </p>
          )}
        </div>
      </div>
      {trailingIcon && (
        <MsIcon icon={trailingIcon} className="text-bombay text-xl" />
      )}
      {trailingComponent && <div>{trailingComponent}</div>}
    </div>
  );
}

export default SettingsMenuCard;
