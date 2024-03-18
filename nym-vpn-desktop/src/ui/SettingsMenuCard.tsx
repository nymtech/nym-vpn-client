import { CSSProperties, ReactNode } from 'react';
import clsx from 'clsx';
import MsIcon from './MsIcon';

export type SettingMenuCardCursor = 'default' | 'pointer' | 'not-allowed';

export type SettingsMenuCardProps = {
  title: string;
  leadingIcon?: string;
  leadingComponent?: ReactNode;
  desc?: string;
  onClick?: () => Promise<void>;
  trailingIcon?: string;
  trailingComponent?: ReactNode;
  disabled?: boolean;
  cursor?: SettingMenuCardCursor;
  className?: string;
  style?: CSSProperties;
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
  cursor,
  className,
  style,
}: SettingsMenuCardProps) {
  return (
    <div
      className={clsx([
        'flex flex-row justify-between items-center gap-4 select-none',
        'bg-white dark:bg-baltic-sea-jaguar px-5 py-4 rounded-lg min-h-16',
        desc ? 'py-2' : 'py-4',
        'hover:bg-platinum dark:hover:bg-onyx',
        'transition',
        disabled && 'opacity-50 pointer-events-none',
        cursor === 'default' && 'cursor-default',
        cursor === 'pointer' && 'cursor-pointer',
        cursor === 'not-allowed' && 'cursor-not-allowed',
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
        {leadingIcon && (
          <MsIcon icon={leadingIcon} className="dark:text-mercury-pinkish" />
        )}
        {leadingComponent && !leadingIcon && leadingComponent}
        <div className="min-w-0 flex flex-col justify-center">
          <p className="truncate text-base text-baltic-sea dark:text-mercury-pinkish select-none">
            {title}
          </p>
          <p className="truncate text-sm text-cement-feet dark:text-mercury-mist select-none">
            {desc}
          </p>
        </div>
      </div>
      {trailingIcon && <MsIcon icon={trailingIcon} />}
      {trailingComponent && !trailingIcon && trailingComponent}
    </div>
  );
}

export default SettingsMenuCard;
