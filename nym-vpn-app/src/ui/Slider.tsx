import clsx from 'clsx';
import * as RxSlider from '@radix-ui/react-slider';

export type SliderProps = {
  value: number;
  step?: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
  onCommit: (value: number) => void;
  disabled?: boolean;
  className?: string;
};

function Slider({
  value,
  step,
  min,
  max,
  onChange,
  onCommit,
  disabled,
  className,
}: SliderProps) {
  return (
    <RxSlider.Root
      step={step}
      min={min}
      max={max}
      value={[value]}
      onValueChange={(numbers) => onChange(numbers[0])}
      onValueCommit={(numbers) => onCommit(numbers[0])}
      className={clsx(
        'relative flex h-6 w-full max-w-80 touch-none select-none items-center',
        className,
      )}
      disabled={disabled}
    >
      <RxSlider.Track className="relative h-1.5 grow rounded-full bg-bombay/60 dark:bg-iron">
        <RxSlider.Range className="absolute h-full rounded-full bg-malachite-moss/50 dark:bg-malachite-moss/60" />
      </RxSlider.Track>
      <RxSlider.Thumb
        className={clsx(
          'block size-4 rounded-full bg-malachite transition hover:scale-110 duration-150',
          'focus:outline-hidden focus:ring-4 focus:ring-malachite/35 dark:focus:ring-malachite/15',
        )}
      />
    </RxSlider.Root>
  );
}

export default Slider;
