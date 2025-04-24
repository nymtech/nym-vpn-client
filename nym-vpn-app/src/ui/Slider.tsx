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
  'data-testid'?: string;
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
  ...rest
}: SliderProps) {
  const testId = rest['data-testid'] || 'slider';

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
      data-testid={testId}
      data-value={value}
      data-min={min}
      data-max={max}
      data-disabled={disabled ? 'true' : 'false'}
    >
      <RxSlider.Track
        className="relative h-1.5 grow rounded-full bg-bombay/60 dark:bg-iron"
        data-testid={`${testId}-track`}
      >
        <RxSlider.Range
          className="absolute h-full rounded-full bg-malachite-moss/50 dark:bg-malachite-moss/60"
          data-testid={`${testId}-range`}
        />
      </RxSlider.Track>
      <RxSlider.Thumb
        className={clsx(
          'block size-4 rounded-full bg-malachite transition hover:scale-110 duration-150',
          'focus:outline-hidden focus:ring-4 focus:ring-malachite/35 dark:focus:ring-malachite/15',
        )}
        data-testid={`${testId}-thumb`}
      />
    </RxSlider.Root>
  );
}

export default Slider;
