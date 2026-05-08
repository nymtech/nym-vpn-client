import { Button } from '@base-ui/react';
import clsx from 'clsx';
import { ReactNode } from 'react';
import Spinner from './Spinner';

export type ButtonVariant =
  | 'primary'
  | 'outlined'
  | 'destructive'
  | 'destructive-outlined';

const variantStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'bg-primary text-baltic-sea',
    'hover:bg-primary-hover',
    'data-active:bg-primary-active',
    'data-disabled:bg-secondary',
  ],
  outlined: [
    'border-1 border-black dark:border-white',
    'text-text-primary',
    'hover:bg-baltic-sea/10 dark:hover:bg-white/10',
    'data-active:bg-transparent',
    'data-disabled:border-black/50 data-disabled:text-black/50 data-disabled:cursor-not-allowed',
    'dark:data-disabled:border-white/50 dark:data-disabled:text-white/50',
  ],
  destructive: [
    'bg-aphrodisiac text-white',
    'hover:bg-aphrodisiac/80',
    'data-active:bg-aphrodisiac/90',
    'data-disabled:bg-secondary',
  ],
  'destructive-outlined': [
    'border-1 border-aphrodisiac text-aphrodisiac',
    'hover:bg-aphrodisiac/10',
    'data-active:bg-aphrodisiac/20',
    'data-disabled:border-aphrodisiac/50 data-disabled:text-aphrodisiac/50',
  ],
};

const loadingStyles: Record<ButtonVariant, string[]> = {
  primary: [
    'group-data-disabled:border-baltic-sea dark:group-data-disabled:border-baltic-sea group-data-disabled:border-b-transparent dark:group-data-disabled:border-b-transparent',
  ],
  outlined: [
    'group-data-disabled:border-black/50 group-data-disabled:border-b-transparent',
  ],
  destructive: [
    'group-data-disabled:border-aphrodisiac/50 group-data-disabled:border-b-transparent',
  ],
  'destructive-outlined': [
    'group-data-disabled:border-aphrodisiac/50 group-data-disabled:border-b-transparent',
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
        'px-4 py-6 text-base',
        'h-12 w-full rounded-3xl',
        'text-base leading-6 font-medium tracking-[0.01em] whitespace-nowrap',
        'cursor-default transition-colors select-none',
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
