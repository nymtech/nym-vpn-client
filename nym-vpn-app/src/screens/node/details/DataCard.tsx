import React from 'react';
import clsx from 'clsx';

export type DataCardProps = {
  children: React.ReactNode[];
  footer?: React.ReactNode;
};

function DataCard({ children, footer }: DataCardProps) {
  const items = children.filter((row) => row !== undefined && row !== null);

  return (
    <div>
      <ul
        className={clsx([
          'flex flex-col justify-center items-center gap-0 select-none',
          'bg-white dark:bg-charcoal rounded-lg p-4',
          'cursor-default',
        ])}
      >
        {items.map((row, i) => (
          <li
            key={i}
            className={clsx(
              'w-full flex border-b last:border-b-0',
              'py-2 last:pb-0 first:pt-0 border-bombay dark:border-iron',
              footer && '[&:nth-last-child(-n+2)]:border-b-0',
            )}
          >
            {row}
          </li>
        ))}
        {footer && (
          <div
            className={clsx(
              'self-start mt-3 text-sm text-iron dark:text-bombay',
            )}
          >
            {footer}
          </div>
        )}
      </ul>
    </div>
  );
}

export default DataCard;
