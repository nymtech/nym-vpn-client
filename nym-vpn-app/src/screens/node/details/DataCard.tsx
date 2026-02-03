import React from 'react';
import clsx from 'clsx';

export type DataCardProps = {
  rows: (
    | { row: React.ReactNode; key: string }
    | undefined
    | false
    | null
    | ''
  )[];
  footer?: React.ReactNode;
};

function DataCard({ rows, footer }: DataCardProps) {
  const filtered = rows.filter(
    (row) => typeof row === 'object' && row !== null,
  );

  return (
    <div>
      <ul
        className={clsx([
          'flex flex-col justify-center items-center gap-0',
          'bg-white dark:bg-charcoal rounded-lg p-4',
          'cursor-default',
        ])}
      >
        {filtered.map(({ row, key }) => (
          <li
            key={key}
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
              'self-start mt-3 text-sm text-iron dark:text-bombay select-none',
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
