import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';
import MsIcon from './MsIcon';

export type SettingsMenuCardProps = {
  title: string;
  leadingIcon?: string;
  leadingComponent?: ReactNode;
  desc?: string;
  onClick?: () => void;
  trailingIcon?: string;
  trailingComponent?: ReactNode;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
  noHoverEffect?: boolean;
  'data-testid'?: string;
};

function SettingsMenuCard({
  title,
  leadingIcon,
  leadingComponent,
  desc,
  onClick,
  trailingIcon,
  trailingComponent,
  disabled,
  className,
  style,
  noHoverEffect,
  ...rest
}: SettingsMenuCardProps) {
  const testId =
    rest['data-testid'] ||
    `settings-card-${title.replace(/\s+/g, '-').toLowerCase()}`;

  return (
    <div
      className={clsx([
        'flex flex-row justify-between items-center gap-4 select-none',
        'bg-white dark:bg-charcoal px-5 rounded-lg min-h-16',
        desc ? 'py-2' : 'py-4',
        !noHoverEffect && 'hover:bg-white/60 dark:hover:bg-charcoal/85',
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
      data-testid={testId}
      data-disabled={disabled ? 'true' : 'false'}
    >
      <div
        className={clsx(
          'overflow-hidden flex flex-row items-center justify-between gap-4',
        )}
        data-testid={`${testId}-content`}
      >
        {leadingIcon && (
          <MsIcon
            icon={leadingIcon}
            className="dark:text-white"
            data-testid={`${testId}-leading-icon`}
          />
        )}
        {leadingComponent && !leadingIcon && (
          <div data-testid={`${testId}-leading-component`}>
            {leadingComponent}
          </div>
        )}
        <div
          className="min-w-0 flex flex-col justify-center"
          data-testid={`${testId}-text-container`}
        >
          <p
            className="truncate text-base text-baltic-sea dark:text-white select-none"
            data-testid={`${testId}-title`}
          >
            {title}
          </p>
          {desc && (
            <p
              className="truncate text-sm text-iron dark:text-bombay select-none"
              data-testid={`${testId}-description`}
            >
              {desc}
            </p>
          )}
        </div>
      </div>
      {trailingIcon && (
        <MsIcon icon={trailingIcon} data-testid={`${testId}-trailing-icon`} />
      )}
      {trailingComponent && !trailingIcon && (
        <div data-testid={`${testId}-trailing-component`}>
          {trailingComponent}
        </div>
      )}
    </div>
  );
}

export default SettingsMenuCard;
