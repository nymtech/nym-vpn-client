import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';

export type ButtonTextProps = {
  children: ReactNode;
  onClick?: () => void;
  onDoubleClick?: () => void;
  disabled?: boolean;
  className?: string;
  textClassName?: string;
  truncate?: boolean;
  color?: 'rain' | 'transparent' | 'malachite';
  'data-testid'?: string;
};

function ButtonText({
  onClick,
  onDoubleClick,
  children,
  disabled,
  className,
  truncate,
  color = 'rain',
  ...rest
}: ButtonTextProps) {
  const colors = {
    rain: [
      'bg-faded-lavender dark:bg-ash',
      'data-hover:text-baltic-sea data-hover:bg-iron/20',
      'dark:data-hover:text-bombay dark:data-hover:bg-charcoal/65',
    ],
    transparent: [
      'text-black dark:text-white',
      'data-hover:underline decoration-2 text-lg font-medium',
    ],
    malachite: [
      'text-primary',
      'data-hover:text-malachite-moss/80 dark:data-hover:text-malachite/80',
    ],
  };

  const testId = rest['data-testid'] || 'button-text';

  return (
    <HuButton
      className={clsx([
        'rounded-lg px-2',
        'focus:outline-hidden data-focus:ring-0',
        'transition data-active:ring-0 data-disabled:opacity-60',
        'cursor-default tracking-normal',
        truncate && 'overflow-hidden',
        ...colors[color],
        className && className,
      ])}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      disabled={disabled}
      data-testid={testId}
      data-test-disabled={disabled ? 'true' : 'false'}
      data-test-truncate={truncate ? 'true' : 'false'}
    >
      <div
        className={clsx(truncate && 'truncate text-nowrap')}
        data-testid={`${testId}-content`}
      >
        {children}
      </div>
    </HuButton>
  );
}

export default ButtonText;
