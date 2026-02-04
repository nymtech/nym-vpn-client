// This is attempt to upgrade Card components currently used in the app
// So in the end there will be only one Card component

import clsx from 'clsx';
import { CSSProperties, ReactNode } from 'react';
import { useClipboard } from '../hooks';
import Switch from './Switch';
import ButtonIcon from './ButtonIcon';
import Skeleton from './Skeleton';

export function CardNewHeader({ children }: { children: ReactNode }) {
  return (
    <div className="w-full flex flex-row items-start justify-start px-5 pt-5">
      {children}
    </div>
  );
}

export function CardNewFooter({ children }: { children: ReactNode }) {
  return <div className="w-full px-5 pb-5">{children}</div>;
}

export function CardNewBody({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={clsx(
        'w-full flex flex-col justify-center items-start gap-3 px-5',
        className,
      )}
    >
      {children}
    </div>
  );
}

export function CardDataRow({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <div className="w-full flex justify-between items-center">
      <p className="text-iron dark:text-bombay truncate select-none">{label}</p>
      <div className="flex flex-nowrap items-center gap-2 overflow-hidden">
        {children}
      </div>
    </div>
  );
}

export function CardNewCopyableRow({
  value,
  label,
  loading = false,
}: {
  value: string;
  label: string;
  loading?: boolean;
}) {
  const { copy } = useClipboard();

  return (
    <div className="w-full flex justify-between items-center gap-2">
      {loading ? (
        <Skeleton className="w-full h-4" />
      ) : (
        <>
          <p className="text-iron dark:text-bombay truncate text-wrap wrap-break-word">
            {label}
          </p>
          <ButtonIcon
            className="self-start"
            iconClassName="!text-xl"
            clickedIconClassName="!text-xl"
            icon="content_copy"
            color="chalk"
            onClick={() => copy(value, false)}
            clickFeedback
            noDefaultSize
          />
        </>
      )}
    </div>
  );
}

export type CardHeaderSwitchProps = {
  header: string;
  subheader?: string;
  subheaderColor?: 'default' | 'king-nacho';
  checked: boolean;
  onClick: () => void;
  disabled?: boolean;
  className?: string;
  style?: CSSProperties;
  noHoverEffect?: boolean;
};

export function CardHeaderSwitch({
  header,
  subheader,
  subheaderColor = 'default',
  checked,
  onClick,
  className,
  style,
  disabled,
  noHoverEffect,
}: CardHeaderSwitchProps) {
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

export type CardNewProps = {
  children: ReactNode;
  disabled?: boolean;
  className?: string;
};

export function CardNew({ children, disabled, className }: CardNewProps) {
  return (
    <div
      className={clsx([
        'flex flex-col justify-center items-center gap-4 select-none',
        'bg-white dark:bg-charcoal rounded-lg min-h-16',
        'transition cursor-default',
        disabled && 'opacity-50 pointer-events-none',
        className,
      ])}
    >
      {children}
    </div>
  );
}
