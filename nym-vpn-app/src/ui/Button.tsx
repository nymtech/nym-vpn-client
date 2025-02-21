import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import { type } from '@tauri-apps/plugin-os';

export type ButtonProps = {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  color?: 'malachite' | 'cornflower' | 'gray' | 'red';
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
        return [
          'bg-malachite data-hover:bg-malachite/75',
          'dark:data-hover:bg-malachite/80',
        ];
      case 'gray':
        return [
          'bg-dim-gray/70 data-hover:bg-dim-gray/85',
          'dark:bg-dusty-grey dark:data-hover:bg-dusty-grey/80',
        ];
      case 'cornflower':
        return [
          'bg-cornflower data-hover:bg-cornflower/85',
          'dark:data-hover:bg-cornflower/80',
        ];
      case 'red':
        return [
          'bg-rouge-ecarlate data-hover:bg-rouge-ecarlate/85',
          'dark:data-hover:bg-rouge-ecarlate/80',
        ];
    }
  };

  const getOutlineColorStyle = () => {
    switch (color) {
      case 'malachite':
        return 'border border-malachite outline-malachite';
      default:
        // TODO add style for other colors
        return null;
    }
  };

  const getOutlineTextColor = () => {
    switch (color) {
      case 'malachite':
        return 'text-malachite';
      case 'gray':
        return 'text-dim-gray dark:text-dusty-grey';
      case 'cornflower':
        return 'text-cornflower';
      case 'red':
        return 'text-rouge-ecarlate';
    }
  };

  const colorStyle = outline ? getOutlineColorStyle() : getColorStyle();

  return (
    <HuButton
      className={clsx([
        'flex justify-center items-center w-full',
        'rounded-lg text-lg font-bold py-3 px-6',
        outline ? getOutlineTextColor() : 'text-black dark:text-baltic-sea',
        'focus:outline-hidden data-focus:ring-2 data-focus:ring-black dark:data-focus:ring-white',
        'transition data-disabled:opacity-60 data-active:ring-0',
        outline && 'data-hover:ring-1 data-hover:ring-malachite',
        'shadow-sm tracking-normal cursor-default',
        colorStyle,
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
