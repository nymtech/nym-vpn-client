import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import { useMainState } from '../contexts';

type ButtonProps = {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  color?: 'melon' | 'cornflower' | 'grey';
  className?: string;
  loading?: boolean;
};

function Spinner() {
  const { os } = useMainState();
  return (
    <span
      className={clsx([
        'loader',
        os === 'linux' ? 'h-[32px] w-[32px]' : 'h-[22px] w-[22px] border-4',
        'border:white dark:border-[#2c2b2e] border-b-transparent dark:border-b-transparent',
      ])}
    ></span>
  );
}

function Button({
  onClick,
  children,
  disabled,
  color = 'melon',
  className,
  loading,
}: ButtonProps) {
  const getColorStyle = () => {
    switch (color) {
      case 'melon':
        return 'bg-melon';
      case 'grey':
        return 'bg-dim-gray dark:bg-dusty-grey';
      case 'cornflower':
        return 'bg-cornflower';
    }
  };

  return (
    <HuButton
      className={clsx([
        'flex justify-center items-center w-full',
        'rounded-lg text-lg font-bold py-3 px-6',
        'text-white dark:text-baltic-sea',
        'focus:outline-none data-[focus]:ring-2 data-[focus]:ring-black data-[focus]:dark:ring-white',
        'transition data-[hover]:opacity-80 data-[disabled]:opacity-60 data-[active]:ring-0',
        'shadow tracking-normal cursor-default',
        getColorStyle(),
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {loading ? Spinner() : children}
    </HuButton>
  );
}

export default Button;
