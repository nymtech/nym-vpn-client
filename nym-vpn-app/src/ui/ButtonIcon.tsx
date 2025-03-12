import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import { MsIcon } from './index';

export type ButtonIconProps = {
  icon: string;
  onClick: () => void;
  disabled?: boolean;
  className?: string;
  iconClassName?: string;
};

function ButtonIcon({
  onClick,
  icon,
  disabled,
  className,
  iconClassName,
}: ButtonIconProps) {
  return (
    <HuButton
      className={clsx([
        'rounded-full w-10 h-10 min-w-10 min-h-10',
        'text-malachite-moss/80 data-hover:text-malachite-moss',
        'dark:text-malachite/80 data-hover:dark:text-malachite',
        'focus:outline-hidden',
        'transition data-disabled:opacity-60 data-active:ring-0',
        'cursor-default select-none',
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {
        <MsIcon
          icon={icon}
          className={clsx([
            'text-2xl w-10 h-10 min-w-10 min-h-10',
            iconClassName,
          ])}
        />
      }
    </HuButton>
  );
}

export default ButtonIcon;
