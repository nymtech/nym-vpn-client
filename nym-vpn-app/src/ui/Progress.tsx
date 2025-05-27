// eslint-disable-next-line import/no-unresolved
import { Progress as BuiProgress } from '@base-ui-components/react/progress';

export type ProgressProps = {
  value?: number | null;
  label?: string;
  className?: string;
};

export default function Progress({ value = null, label }: ProgressProps) {
  return (
    <BuiProgress.Root className="grid w-48 grid-cols-2 gap-y-2" value={value}>
      <BuiProgress.Label className="text-sm font-medium text-gray-900">
        {label || 'Progress'}
      </BuiProgress.Label>
      <BuiProgress.Value className="col-start-2 text-right text-sm text-gray-900" />
      <BuiProgress.Track className="col-span-full h-1 overflow-hidden rounded bg-gray-200 shadow-[inset_0_0_0_1px] shadow-gray-200">
        <BuiProgress.Indicator className="block bg-gray-500 transition-all duration-500" />
      </BuiProgress.Track>
    </BuiProgress.Root>
  );
}
