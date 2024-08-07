import clsx from 'clsx';
import { Button } from '@headlessui/react';
import { open } from '@tauri-apps/api/shell';
import MsIcon from './MsIcon';

type LinkProps = {
  text: string;
  url: string;
  icon?: boolean | string;
  className?: string;
  textClassName?: string;
};

function Link({ text, url, icon, className, textClassName }: LinkProps) {
  return (
    <Button
      as="a"
      className={clsx([
        'focus:outline-none select-none cursor-default',
        'flex flex-row items-center gap-1 text-melon',
        className && className,
      ])}
      onClick={() => open(url)}
    >
      {({ hover }) => (
        <>
          <span
            className={clsx([
              hover ? 'underline' : '',
              'decoration-2 underline-offset-4',
              textClassName && textClassName,
            ])}
          >
            {text}
          </span>
          {icon && (
            <MsIcon
              className="!no-underline hover:!no-underline"
              icon={typeof icon === 'string' ? icon : 'open_in_new'}
            />
          )}
        </>
      )}
    </Button>
  );
}

export default Link;
