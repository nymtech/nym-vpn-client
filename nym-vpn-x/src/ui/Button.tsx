import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';

type ButtonProps = {
  children: ReactNode;
  onClick: () => Promise<void>;
  disabled?: boolean;
  color?: 'melon' | 'cornflower' | 'grey';
  className?: string;
};

function Button({
  onClick,
  children,
  disabled,
  color = 'melon',
  className,
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
        'focus:outline-none focus:ring-2 focus:ring-black focus:dark:ring-white',
        'active:ring-0',
        'transition hover:opacity-80 disabled:opacity-50',
        'shadow tracking-normal cursor-default',
        getColorStyle(),
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </HuButton>
  );
}

export default Button;
