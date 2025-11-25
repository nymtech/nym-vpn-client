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
  variant?: 'normal' | 'danger';
};

const getVariantStyles = (
  variant: 'normal' | 'danger',
  noHoverEffect?: boolean,
) => {
  const variantStyles = {
    normal: {
      base: 'bg-white dark:bg-charcoal',
      hover: 'hover:bg-white/60 dark:hover:bg-charcoal/85',
    },
    danger: {
      base: 'border-aphrodisiac border bg-aphrodisiac/10 dark:bg-aphrodisiac/10',
      hover: 'hover:bg-aphrodisiac/10 dark:hover:bg-aphrodisiac/20',
    },
  };

  return (
    variantStyles[variant].base +
    (noHoverEffect ? '' : ` ${variantStyles[variant].hover}`)
  );
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
  variant = 'normal',
}: SettingsMenuCardProps) {
  const variantStyles = getVariantStyles(variant, noHoverEffect);

  return (
    <div
      className={clsx([
        variantStyles,
        'flex flex-row justify-between items-center gap-4 select-none',
        'px-5 rounded-lg min-h-16',
        description ? 'py-2' : 'py-4',
        'transition cursor-default',
        disabled && 'opacity-50 pointer-events-none',
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
          'overflow-hidden flex flex-row items-center justify-between gap-4',
        )}
      >
        {leadingIcon && <MsIcon icon={leadingIcon} className="text-bombay" />}
        {leadingComponent && !leadingIcon && <div>{leadingComponent}</div>}
        <div className="min-w-0 flex flex-col justify-center">
          <p className="truncate text-base text-baltic-sea dark:text-white select-none">
            {title}
          </p>
          {description && (
            <p
              className={clsx(
                'truncate text-sm select-none',
                descriptionColor === 'normal' && 'text-iron dark:text-bombay',
                descriptionColor === 'red' && 'text-aphrodisiac',
                descriptionColor === 'yellow' && 'text-king-nacho',
              )}
            >
              {description}
            </p>
          )}
        </div>
      </div>
      {trailingIcon && <MsIcon icon={trailingIcon} className="text-bombay" />}
      {trailingComponent && !trailingIcon && <div>{trailingComponent}</div>}
    </div>
  );
}

export default SettingsMenuCard;
