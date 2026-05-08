// This is attempt to upgrade Card components currently used in the app
// So in the end there will be only one Card component

import clsx from 'clsx';
import { CSSProperties, ReactNode } from 'react';
import { useClipboard } from '../hooks';
import Switch from './Switch';
import ButtonIcon from './ButtonIcon';
import Skeleton from './Skeleton';

export function CardNewHeader({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={clsx(
        'flex w-full flex-row items-start justify-start p-4',
        className,
      )}
    >
      {children}
    </div>
  );
}

export function CardNewFooter({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={clsx('w-full px-5 pt-3 pb-5', className)}>{children}</div>
  );
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
        'flex w-full flex-col items-start justify-center px-4',
        className,
      )}
    >
      {children}
    </div>
  );
}

export function CardDivider({ className }: { className?: string }) {
  return (
    <div
      className={clsx(
        'h-px w-full shrink-0 bg-black/8 dark:bg-white/10',
        className,
      )}
    />
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
    <div className="flex w-full items-center justify-between py-[7px]">
      <p className="text-text-secondary truncate select-none">{label}</p>
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
    <div className="flex w-full items-center justify-between gap-2 py-[7px]">
      {loading ? (
        <Skeleton className="h-4 w-full" />
      ) : (
        <>
          <p className="text-text-secondary truncate text-wrap wrap-break-word">
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
        'flex w-full flex-row items-center justify-between gap-4 select-none',
        'dark:bg-charcoal min-h-16 rounded-t-lg bg-white px-4 py-4',
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
      <div className="flex min-w-0 flex-col justify-center gap-1">
        <p className="text-text-primary truncate text-base select-none">
          {header}
        </p>
        {subheader && (
          <p
            className={clsx(
              'text-sm select-none',
              subheaderColor === 'default'
                ? 'text-text-secondary'
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
        '',
        'flex flex-col items-center justify-center overflow-hidden select-none',
        'dark:bg-aph-light min-h-16 rounded-2xl bg-white',
        'cursor-default transition',
        disabled && 'pointer-events-none opacity-50',
        className,
      ])}
    >
      {children}
    </div>
  );
}
