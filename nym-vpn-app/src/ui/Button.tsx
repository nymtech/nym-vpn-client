import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import { type } from '@tauri-apps/plugin-os';

export type ButtonProps = {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  color?: 'malachite' | 'cornflower' | 'gray';
  outline?: boolean;
  className?: string;
  spinner?: boolean;
};

function Spinner() {
  const os = type();

  return (
    <span
      className={clsx([
        'loader',
        os === 'linux' ? 'h-[28px] w-[28px]' : 'h-[22px] w-[22px] border-4',
        'border:white dark:border-[#252426] border-b-transparent dark:border-b-transparent',
      ])}
    ></span>
  );
}

function Button({
  onClick,
  children,
  disabled,
  color = 'malachite',
  outline,
  className,
  spinner,
}: ButtonProps) {
  const getColorStyle = () => {
    switch (color) {
      case 'malachite':
        if (outline) {
          return 'border border-malachite outline-malachite';
        } else {
          return 'bg-malachite';
        }
      case 'gray':
        return 'bg-dim-gray bg-opacity-70 dark:bg-dusty-grey dark:bg-opacity-100';
      case 'cornflower':
        return 'bg-cornflower';
    }
  };

  const getOutilineTextColor = () => {
    switch (color) {
      case 'malachite':
        return 'text-malachite';
      case 'gray':
        return 'text-dim-gray dark:text-dusty-grey';
      case 'cornflower':
        return 'text-cornflower';
    }
  };

  return (
    <HuButton
      className={clsx([
        'flex justify-center items-center w-full',
        'rounded-lg text-lg font-bold py-3 px-6',
        outline ? getOutilineTextColor() : 'text-black dark:text-baltic-sea',
        'focus:outline-none data-[focus]:ring-2 data-[focus]:ring-black data-[focus]:dark:ring-white',
        'transition data-[disabled]:opacity-60 data-[active]:ring-0',
        outline
          ? 'data-[hover]:ring-1 data-[hover]:ring-malachite'
          : 'data-[hover]:bg-opacity-80 data-[hover]:dark:bg-opacity-85',
        'shadow tracking-normal cursor-default',
        getColorStyle(),
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {spinner ? Spinner() : <div className="truncate">{children}</div>}
    </HuButton>
  );
}

export default Button;
