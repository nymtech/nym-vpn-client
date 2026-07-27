import React from 'react';
import clsx from 'clsx';

export type DataCardProps = {
  rows: (
    { row: React.ReactNode; key: string } | undefined | false | null | ''
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
          'flex flex-col items-center justify-center gap-0',
          'dark:bg-surface-elev rounded-lg bg-white p-4',
          'cursor-default',
        ])}
      >
        {filtered.map(({ row, key }) => (
          <li
            key={key}
            className={clsx(
              'flex w-full border-b last:border-b-0',
              'border-text-tertiary dark:border-text-secondary py-2 first:pt-0 last:pb-0',
              footer && '[&:nth-last-child(-n+2)]:border-b-0',
            )}
          >
            {row}
          </li>
        ))}
        {footer && (
          <div
            className={clsx(
              'text-text-secondary mt-3 self-start text-sm select-none',
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
