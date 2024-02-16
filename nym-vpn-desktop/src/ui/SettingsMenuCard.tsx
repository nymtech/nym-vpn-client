import { ReactNode } from 'react';
import clsx from 'clsx';
import MsIcon from './MsIcon';

export type SettingMenuCardCursor = 'default' | 'pointer' | 'not-allowed';

type SettingMenuCardProps = {
  title: string;
  leadingIcon?: string;
  leadingComponent?: ReactNode;
  desc?: string;
  onClick?: () => Promise<void>;
  trailingIcon?: string;
  trailingComponent?: ReactNode;
  disabled?: boolean;
  cursor?: SettingMenuCardCursor;
};

function SettingMenuCard({
  title,
  leadingIcon,
  leadingComponent,
  desc,
  onClick,
  trailingIcon,
  trailingComponent,
  disabled,
  cursor,
}: SettingMenuCardProps) {
  return (
    <div
      className={clsx([
        'flex flex-row justify-between items-center gap-4 select-none',
        'bg-white dark:bg-baltic-sea-jaguar px-5 py-4 rounded-lg min-h-16',
        desc ? 'py-2' : 'py-4',
        disabled && 'opacity-50 pointer-events-none',
        cursor === 'default' && 'cursor-default',
        cursor === 'pointer' && 'cursor-pointer',
        cursor === 'not-allowed' && 'cursor-not-allowed',
      ])}
      onClick={onClick}
      onKeyDown={(e) => {
        if (e.key === 'Enter') onClick?.();
      }}
      role="button"
      tabIndex={disabled ? -1 : 0}
    >
      <div className={clsx('flex flex-row items-center justify-between gap-4')}>
        {leadingIcon && (
          <MsIcon icon={leadingIcon} style="dark:text-mercury-pinkish" />
        )}
        {leadingComponent && !leadingIcon && leadingComponent}
        <div className="flex flex-1 items-center">
          <div className="text-sm">
            <div className="text-base text-baltic-sea dark:text-mercury-pinkish select-none">
              {title}
            </div>
            <div className="text-sm text-cement-feet dark:text-mercury-mist select-none">
              {desc}
            </div>
          </div>
        </div>
      </div>
      {trailingIcon && <MsIcon icon={trailingIcon} />}
      {trailingComponent && !trailingIcon && trailingComponent}
    </div>
  );
}

export default SettingMenuCard;
