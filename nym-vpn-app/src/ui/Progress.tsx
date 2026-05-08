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
      className={clsx('flex flex-col gap-2', 'text-text-secondary', className)}
      value={value}
    >
      <div className="flex items-center justify-between">
        <BuiProgress.Label className="text-sm">
          {label || 'Progress'}
        </BuiProgress.Label>
        <BuiProgress.Value className="text-sm" />
      </div>
      <BuiProgress.Track className="bg-faded-lavender dark:bg-ash h-1.5 rounded-full border-none">
        <BuiProgress.Indicator className="bg-malachite rounded-full border-none transition-all duration-150" />
      </BuiProgress.Track>
    </BuiProgress.Root>
  );
}
