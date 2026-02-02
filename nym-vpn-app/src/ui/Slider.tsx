import { Slider as HuSlider } from '@base-ui-components/react';
import clsx from 'clsx';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

export type SliderProps = {
  defaultValue?: number;
  value: number;
  onChange?: (value: number) => void;
  onValueCommitted: (value: number) => void;
  min: number;
  max: number;
  step: number;
  labels?: React.ReactNode[];
  className?: string;
  valueIndicator?: boolean;
};

function Slider({
  defaultValue,
  value,
  onChange,
  onValueCommitted,
  min,
  max,
  step,
  labels,
  valueIndicator,
  className,
}: SliderProps) {
  const { t } = useTranslation('common');

  const [internalValue, setInternalValue] = useState(value);

  useEffect(() => {
    setInternalValue(value);
  }, [value]);

  // Calculate position percentage for each label
  const getLabelPosition = (index: number): number => {
    if (!labels || labels.length === 0) return 0;

    const range = max - min;
    const useEvenDistribution = range > 5;

    // First label at leftmost (0%)
    if (index === 0) return 0;

    // Last label at rightmost (100%)
    if (index === labels.length - 1) return 100;

    // For ranges > 5: distribute labels evenly
    if (useEvenDistribution) {
      // Evenly distribute: position = (index / (totalLabels - 1)) * 100
      return (index / (labels.length - 1)) * 100;
    }

    // For ranges <= 5: calculate position based on step value
    // Label at index i corresponds to value: min + i * step
    const labelValue = min + index * step;
    const position = ((labelValue - min) / (max - min)) * 100;

    return position;
  };

  return (
    <div className={clsx('w-full', className)}>
      <HuSlider.Root
        min={min}
        max={max}
        step={step}
        defaultValue={defaultValue}
        value={internalValue}
        onValueCommitted={onValueCommitted}
        onValueChange={(val) => {
          setInternalValue(val);
          onChange?.(val);
        }}
        className="relative flex w-full touch-none select-none items-center"
      >
        <HuSlider.Control className="w-full">
          <HuSlider.Track className="relative h-2 w-full rounded-full bg-bombay dark:bg-iron">
            <HuSlider.Indicator
              className={clsx([
                'absolute h-full rounded-full bg-malachite-moss dark:bg-malachite',
                'transition-[width] duration-300 ease-out',
              ])}
            />
            <HuSlider.Thumb
              className={clsx([
                'group block h-6 w-6 rounded-full border active:bg-faded-lavender bg-white hover:bg-faded-lavender shadow-md focus:outline-none focus:ring-2 focus:ring-malachite',
                'transition-[inset] duration-300 ease-out ',
              ])}
            >
              {/*   */}
            </HuSlider.Thumb>
          </HuSlider.Track>
        </HuSlider.Control>
      </HuSlider.Root>

      <div className="relative">
        {valueIndicator && (
          <div className="absolute left-1/2 -translate-x-1/2 flex flex-col justify-between items-center text-sm text-cornflower">
            <span className="whitespace-nowrap">
              {internalValue === defaultValue ? t('default') : t('current')}
            </span>
            <span className="whitespace-nowrap">{internalValue} ms</span>
          </div>
        )}
        {labels && (
          <div className="mt-5 w-full h-10">
            {labels.map((label, idx) => {
              const position = getLabelPosition(idx);
              return (
                <div
                  key={idx}
                  className="absolute flex items-center justify-center"
                  style={{
                    left: `${position}%`,
                    transform:
                      idx === 0
                        ? 'translateX(0)'
                        : idx === labels.length - 1
                          ? 'translateX(-100%)'
                          : 'translateX(-50%)',
                  }}
                >
                  {label}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

export default Slider;
