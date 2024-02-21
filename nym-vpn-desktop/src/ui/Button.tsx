import { ReactNode } from 'react';
import clsx from 'clsx';

type ButtonProps = {
  children: ReactNode;
  onClick: () => Promise<void>;
  disabled?: boolean;
};

function Button({ onClick, children, disabled }: ButtonProps) {
  return (
    <button
      className={clsx([
        'flex justify-center items-center w-full',
        'rounded-lg text-lg font-bold py-3 px-6',
        'focus:outline-none focus:ring-4 focus:ring-black focus:dark:ring-white',
        'bg-melon text-white dark:text-baltic-sea shadow tracking-normal',
      ])}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}

export default Button;
