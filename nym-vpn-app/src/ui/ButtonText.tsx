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
  colors?: 'rain';
  'data-testid'?: string;
};

function ButtonText({
  onClick,
  onDoubleClick,
  children,
  disabled,
  className,
  truncate,
  colors = 'rain',
  ...rest
}: ButtonTextProps) {
  const getColors = () => {
    switch (colors) {
      case 'rain':
        return [
          'bg-faded-lavender dark:bg-ash',
          'data-hover:text-baltic-sea data-hover:bg-iron/20',
          'dark:data-hover:text-bombay dark:data-hover:bg-charcoal/65',
        ];
    }
  };

  const testId = rest['data-testid'] || 'button-text';

  return (
    <HuButton
      className={clsx([
        'rounded-lg px-2',
        'focus:outline-hidden data-focus:ring-0',
        'transition data-disabled:opacity-60 data-active:ring-0',
        'tracking-normal cursor-default',
        truncate && 'overflow-hidden',
        className && className,
        ...getColors(),
      ])}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      disabled={disabled}
      data-testid={testId}
      data-disabled={disabled ? 'true' : 'false'}
      data-truncate={truncate ? 'true' : 'false'}
    >
      <div
        className={clsx(truncate && 'text-nowrap truncate')}
        data-testid={`${testId}-content`}
      >
        {children}
      </div>
    </HuButton>
  );
}

export default ButtonText;
