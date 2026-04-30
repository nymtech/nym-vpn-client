import { Button } from '@base-ui/react';
import clsx from 'clsx';
import { ReactNode } from 'react';
import Spinner from './Spinner';

export type ButtonVariant = 'primary' | 'outlined';

const variantStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'bg-malachite-200 text-baltic-sea',
    'hover:bg-malachite-300',
    'data-active:bg-malachite-400',
    'data-disabled:bg-secondary',
  ],
  outlined: [
    'border-1 border-black dark:border-white',
    'text-baltic-sea dark:text-white',
    'hover:bg-baltic-sea/10 dark:hover:bg-white/10',
    'data-active:bg-transparent',
    'data-disabled:border-black/50 data-disabled:text-black/50 data-disabled:cursor-not-allowed',
    'dark:data-disabled:border-white/50 dark:data-disabled:text-white/50',
  ],
};

const loadingStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'group-data-disabled:border-baltic-sea dark:group-data-disabled:border-baltic-sea group-data-disabled:border-b-transparent dark:group-data-disabled:border-b-transparent',
  ],
  outlined: [
    'group-data-disabled:border-black/50 group-data-disabled:border-b-transparent',
  ],
};
export type ButtonNewProps = {
  children: ReactNode;
  variant?: ButtonVariant;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  loading?: boolean;
};

function ButtonNew({
  children,
  variant = 'primary',
  onClick,
  disabled,
  className,
  loading,
}: ButtonNewProps) {
  const content = loading ? (
    <Spinner className={clsx(loadingStyles[variant])} />
  ) : (
    children
  );
  return (
    <Button
      className={clsx([
        'group',
        'flex items-center justify-center',
        'py-7 text-base',
        'h-12 w-full rounded-3xl',
        'font-medium text-base tracking-[0.01em] leading-6 whitespace-nowrap',
        'transition-colors cursor-default select-none',
        'focus:outline-none',
        'data-disabled:pointer-events-none',
        ...variantStyles[variant],
        className,
      ])}
      onClick={onClick}
      disabled={disabled || loading}
    >
      {content}
    </Button>
  );
}

export default ButtonNew;
