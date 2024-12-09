import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';

type ButtonTextProps = {
  children: ReactNode;
  onClick?: () => void;
  onDoubleClick?: () => void;
  disabled?: boolean;
  className?: string;
  textClassName?: string;
  truncate?: boolean;
  colors?: 'rain';
};

function ButtonText({
  onClick,
  onDoubleClick,
  children,
  disabled,
  className,
  truncate,
  colors = 'rain',
}: ButtonTextProps) {
  const getColors = () => {
    switch (colors) {
      case 'rain':
        return [
          'bg-blanc-nacre dark:bg-baltic-sea',
          'data-[hover]:text-oil data-[hover]:bg-cement-feet/30',
          'data-[hover]:dark:text-laughing-jack data-[hover]:dark:bg-baltic-sea-jaguar/80',
        ];
    }
  };

  return (
    <HuButton
      className={clsx([
        'rounded-lg px-2',
        'focus:outline-none data-[focus]:ring-0',
        'transition data-[disabled]:opacity-60 data-[active]:ring-0',
        'tracking-normal cursor-default',
        truncate && 'overflow-hidden',
        className && className,
        ...getColors(),
      ])}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      disabled={disabled}
    >
      <div className={clsx(truncate && 'text-nowrap truncate')}>{children}</div>
    </HuButton>
  );
}

export default ButtonText;
