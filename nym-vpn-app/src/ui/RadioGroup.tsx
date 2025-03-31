import React, { Key, useState } from 'react';
import {
  Description,
  RadioGroup as HuRadioGroup,
  Label,
  Radio,
} from '@headlessui/react';
import clsx from 'clsx';

export type RadioGroupOptionCursor = 'default' | 'pointer' | 'not-allowed';

export type RadioGroupOption<K extends Key> = {
  key: K;
  label: string;
  desc?: string;
  disabled?: boolean;
  icon?: React.ReactNode | ((checked: boolean) => React.ReactNode);
  cursor?: RadioGroupOptionCursor;
  // custom style applied to the container of the option
  className?: string;
  tooltip?: string;
};

export type RadioGroupProps<K extends Key> = {
  options: RadioGroupOption<K>[];
  defaultValue?: K;
  onChange: (value: K) => void;
  rootLabel?: string;
  // either or not to show checked/unchecked circular icons
  radioIcons?: boolean;
  disabled?: boolean;
};

function RadioGroup<K extends Key>({
  options,
  defaultValue,
  onChange,
  rootLabel,
  radioIcons = true,
  disabled = false,
}: RadioGroupProps<K>) {
  const [selected, setSelected] = useState(defaultValue || options[0]);

  const handleChange = (value: K) => {
    setSelected(value);
    onChange(value);
  };

  const checkedIcon = (checked: boolean) => {
    if (checked) {
      return (
        <span className="font-icon text-2xl text-malachite-moss dark:text-malachite">
          radio_button_checked
        </span>
      );
    }
    return (
      <span className="font-icon text-2xl text-iron dark:text-bombay">
        radio_button_unchecked
      </span>
    );
  };

  return (
    <div className="select-none">
      <HuRadioGroup
        value={selected}
        onChange={handleChange}
        disabled={disabled}
      >
        {rootLabel && (
          <Label
            as="div"
            className="font-medium text-base text-baltic-sea dark:text-white mb-6 cursor-default"
          >
            {rootLabel}
          </Label>
        )}
        <div className="space-y-4">
          {options.map((option) => (
            <Radio
              key={option.key}
              value={option.key}
              className={({ checked }) =>
                clsx([
                  'bg-white dark:bg-charcoal relative flex rounded-lg px-5 py-2 focus:outline-hidden',
                  checked &&
                    'border border-malachite-moss hover:border-malachite-moss',
                  checked &&
                    'dark:border-malachite dark:hover:border-malachite',
                  !checked && 'border border-white dark:border-charcoal',
                  !option.disabled &&
                    !checked &&
                    'hover:border-transparent dark:hover:border-charcoal/85',
                  !option.disabled &&
                    'hover:bg-white/60 dark:hover:bg-charcoal/85',
                  'transition-noborder cursor-default',
                  option.tooltip && 'attach-tooltip',
                  disabled &&
                    'opacity-50 dark:opacity-60 hover hover:bg-white! dark:hover:bg-charcoal!',
                ])
              }
              disabled={option.disabled}
            >
              {({ checked }) => {
                return (
                  <>
                    {option.tooltip && (
                      <div className="tooltip -mt-8 -ml-2 max-w-[90%]">
                        {option.tooltip}
                      </div>
                    )}
                    <div
                      className={clsx([
                        'overflow-hidden flex flex-1 items-center justify-start gap-5',
                        option.className && option.className,
                      ])}
                    >
                      {radioIcons && checkedIcon(checked)}
                      {option.icon && (
                        <div className="w-7 flex justify-center items-center">
                          {typeof option.icon === 'function'
                            ? option.icon(checked)
                            : option.icon}
                        </div>
                      )}
                      <div className="min-w-0 flex flex-col justify-center">
                        <Label
                          as="p"
                          className={clsx([
                            'truncate text-base text-baltic-sea dark:text-white',
                          ])}
                        >
                          {option.label}
                        </Label>
                        {option.desc && (
                          <Description
                            as="span"
                            className="truncate text-sm text-iron dark:text-bombay"
                          >
                            <span>{option.desc}</span>
                          </Description>
                        )}
                      </div>
                    </div>
                  </>
                );
              }}
            </Radio>
          ))}
        </div>
      </HuRadioGroup>
    </div>
  );
}

export default RadioGroup;
