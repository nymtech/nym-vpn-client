import clsx from 'clsx';
import { Progress as BuiProgress } from '@base-ui-components/react/progress';

export type ProgressProps = {
  value?: number | null;
  label?: string;
  className?: string;
};

export default function Progress({
  value = null,
  label,
  className,
}: ProgressProps) {
  return (
    <BuiProgress.Root
      className={clsx(
        'flex flex-col gap-2',
        'text-iron dark:text-bombay',
        className,
      )}
      value={value}
    >
      <div className="flex justify-between items-center">
        <BuiProgress.Label className="text-sm">
          {label || 'Progress'}
        </BuiProgress.Label>
        <BuiProgress.Value className="text-sm" />
      </div>
      <BuiProgress.Track className="h-1.5 rounded-full bg-faded-lavender dark:bg-ash border-none">
        <BuiProgress.Indicator className="rounded-full transition-all duration-200 bg-malachite border-none" />
      </BuiProgress.Track>
    </BuiProgress.Root>
  );
}
