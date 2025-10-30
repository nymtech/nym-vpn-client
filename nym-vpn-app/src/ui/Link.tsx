import React from 'react';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { Button } from '@headlessui/react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Routes } from '../types';
import MsIcon from './MsIcon';

type LinkProps = {
  text?: string;
  children?: React.ReactNode;
  url?: string;
  to?: Routes;
  icon?: boolean | string;
  color?: 'primary' | 'malachite' | 'iron';
  className?: string;
  textClassName?: string;
  iconClassName?: string;
  selectable?: boolean;
  'data-testid'?: string;
};

function Link({
  text,
  children,
  url,
  to,
  icon,
  color = 'malachite',
  className,
  textClassName,
  iconClassName,
  selectable,
  ...rest
}: LinkProps) {
  const testId =
    rest['data-testid'] ||
    `link-${(text || 'unknown').replace(/\s+/g, '-').toLowerCase()}`;
  const navigate = useNavigate();

  const handleClick = () => {
    if (url) {
      openUrl(url);
    } else if (to) {
      navigate(to);
    }
  };

  return (
    <Button
      as="a"
      className={clsx([
        'focus:outline-hidden select-none cursor-default',
        'inline-flex flex-row items-center gap-1',
        color === 'malachite' && 'text-malachite-moss dark:text-malachite',
        color === 'iron' && 'text-iron dark:text-bombay',
        color === 'primary' && 'text-baltic-sea dark:text-white',
        className && className,
        selectable && '!select-text',
      ])}
      onClick={handleClick}
      data-testid={testId}
      data-test-url={url}
    >
      {({ hover }) => (
        <>
          <span
            className={clsx([
              hover ? 'underline' : '',
              'decoration-2 underline-offset-4',
              textClassName && textClassName,
            ])}
            data-testid={`${testId}-text`}
          >
            {children ? children : text}
          </span>
          {icon && (
            <MsIcon
              className={clsx(
                'no-underline! hover:no-underline!',
                iconClassName,
              )}
              icon={typeof icon === 'string' ? icon : 'open_in_new'}
              data-testid={`${testId}-icon`}
            />
          )}
        </>
      )}
    </Button>
  );
}

export default Link;
