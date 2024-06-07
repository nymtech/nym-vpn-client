import { ReactNode } from 'react';
import clsx from 'clsx';

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
    <button
      className={clsx([
        'flex justify-center items-center w-full',
        'rounded-lg text-lg font-bold py-3 px-6',
        'text-white dark:text-baltic-sea',
        'focus:outline-none focus:ring-4 focus:ring-black focus:dark:ring-white',
        'transition hover:opacity-80 disabled:opacity-50',
        'shadow tracking-normal cursor-default',
        getColorStyle(),
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}

export default Button;
