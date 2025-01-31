// TODO _WIP_

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
        'text-white dark:text-mercury-mist bg-aluminium dark:bg-ash',
        'data-[hover]:dark:text-white data-[hover]:dark:bg-baltic-sea-jaguar/80',
        'focus:outline-none data-[focus]:ring-2 data-[focus]:ring-black data-[focus]:dark:ring-white',
        'transition data-[disabled]:opacity-60 data-[active]:ring-0',
        'shadow tracking-normal cursor-default',
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
