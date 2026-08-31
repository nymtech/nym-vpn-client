import { Button as BuButton } from '@base-ui/react';
import clsx from 'clsx';
import { ReactNode } from 'react';
import Spinner from './Spinner';

export type ButtonVariant =
  'primary' | 'outlined' | 'destructive' | 'destructive-outlined';

const variantStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'bg-brand-primary text-brand-on-primary',
    'hover:bg-brand-primary-hover',
    'data-active:bg-brand-primary-active',
    'data-disabled:bg-secondary',
  ],
  outlined: [
    'border-1 border-black dark:border-white',
    'text-text-primary',
    'hover:bg-text-primary/10 dark:hover:bg-white/10',
    'data-active:bg-transparent',
    'data-disabled:border-black/50 data-disabled:text-black/50 data-disabled:cursor-not-allowed',
    'dark:data-disabled:border-white/50 dark:data-disabled:text-white/50',
  ],
  destructive: [
    'bg-status-error text-white',
    'hover:bg-status-error/80',
    'data-active:bg-status-error/90',
    'data-disabled:bg-secondary',
  ],
  'destructive-outlined': [
    'border-1 border-status-error text-status-error',
    'hover:bg-status-error/10',
    'data-active:bg-status-error/20',
    'data-disabled:border-status-error/50 data-disabled:text-status-error/50',
  ],
};

const loadingStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'group-data-disabled:border-text-primary dark:group-data-disabled:border-text-primary group-data-disabled:border-b-transparent dark:group-data-disabled:border-b-transparent',
  ],
  outlined: [
    'group-data-disabled:border-black/50 group-data-disabled:border-b-transparent',
  ],
  destructive: [
    'group-data-disabled:border-status-error/50 group-data-disabled:border-b-transparent',
  ],
  'destructive-outlined': [
    'group-data-disabled:border-status-error/50 group-data-disabled:border-b-transparent',
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

function Button({
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
    <BuButton
      className={clsx([
        'group',
        'flex items-center justify-center',
        'px-4 py-6 text-base',
        'h-12 w-full rounded-3xl',
        'text-base leading-6 font-medium tracking-[0.01em] whitespace-nowrap',
        'cursor-default transition-colors select-none',
        'focus:outline-none',
        'data-disabled:pointer-events-none',
        'focus-visible:outline-brand-primary focus-visible:outline-2 focus-visible:outline-offset-2',
        disabled
          ? 'text-text-secondary bg-surface-hair'
          : [...variantStyles[variant]],
        className,
      ])}
      onClick={onClick}
      disabled={disabled || loading}
    >
      {content}
    </BuButton>
  );
}

export default Button;
