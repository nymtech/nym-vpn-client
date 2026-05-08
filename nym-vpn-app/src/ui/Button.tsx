import { ReactNode } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import Spinner from './Spinner';

export type ButtonProps = {
  children: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  color?: 'malachite' | 'cornflower' | 'gray' | 'red';
  textSize?: 'base' | 'lg';
  outline?: boolean;
  className?: string;
  spinner?: boolean;
  'data-testid'?: string;
};

function Button({
  onClick,
  children,
  disabled,
  color = 'malachite',
  textSize = 'lg',
  outline,
  className,
  spinner,
  ...rest
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
          'bg-iron/70 data-hover:bg-iron/90',
          'dark:bg-bombay dark:data-hover:bg-bombay/80',
        ];
      case 'cornflower':
        return [
          'bg-cornflower data-hover:bg-cornflower/85',
          'dark:data-hover:bg-cornflower/80',
        ];
      case 'red':
        return [
          'bg-aphrodisiac data-hover:bg-aphrodisiac/85',
          'dark:data-hover:bg-aphrodisiac/80',
        ];
    }
  };

  const getOutlineColorStyle = () => {
    switch (color) {
      case 'malachite':
        return 'border border-malachite outline-malachite data-hover:ring-1 data-hover:ring-malachite';
      case 'red':
        return [
          'bg-aphrodisiac/10 data-hover:bg-aphrodisiac/20',
          'border border-aphrodisiac outline-aphrodisiac',
        ];
      case 'gray':
        return 'data-hover:border-iron dark:data-hover:text-bombay data-hover:ring-1 data-hover:ring-malachite';
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
        return 'text-text-secondary';
      case 'cornflower':
        return 'text-cornflower';
      case 'red':
        return 'text-aphrodisiac dark:text-white';
    }
  };

  const getTextSizeStyle = () => {
    switch (textSize) {
      case 'base':
        return 'text-base';
      case 'lg':
        return 'text-lg';
    }
  };
  const colorStyle = outline ? getOutlineColorStyle() : getColorStyle();
  const testId = rest['data-testid'] || 'button';

  return (
    <HuButton
      className={clsx([
        'flex w-full items-center justify-center',
        'rounded-lg px-6 py-3 font-medium',
        getTextSizeStyle(),
        outline ? getOutlineTextColor() : 'text-baltic-sea',
        'focus:outline-hidden',
        'transition data-active:ring-0 data-disabled:opacity-60',
        'cursor-default tracking-normal',
        colorStyle,
        className && className,
      ])}
      onClick={onClick}
      disabled={disabled}
      data-testid={testId}
      data-test-color={color}
      data-test-outline={outline ? 'true' : 'false'}
      data-test-disabled={disabled ? 'true' : 'false'}
    >
      {spinner ? (
        <Spinner />
      ) : (
        <div className="truncate" data-testid={`${testId}-text`}>
          {children}
        </div>
      )}
    </HuButton>
  );
}

export default Button;
