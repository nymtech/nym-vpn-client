import React, { Key, useEffect, useState } from 'react';
import { RadioGroup as HuRadioGroup } from '@headlessui/react';
import clsx from 'clsx';

export type RadioGroupOptionCursor = 'default' | 'pointer' | 'not-allowed';

export type RadioGroupOption<K extends Key> = {
  key: K;
  label: string;
  desc?: string;
  disabled?: boolean;
  icon?: React.ReactNode;
  cursor?: RadioGroupOptionCursor;
  // custom style applied to the container of the option
  className?: string;
};

export type RadioGroupProps<K extends Key> = {
  options: RadioGroupOption<K>[];
  defaultValue?: K;
  onChange: (value: K) => void;
  rootLabel?: string;
};

function RadioGroup<K extends Key>({
  options,
  defaultValue,
  onChange,
  rootLabel,
}: RadioGroupProps<K>) {
  const [selected, setSelected] = useState(defaultValue || options[0]);

  useEffect(() => {
    setSelected(defaultValue || options[0]);
  }, [defaultValue, options]);

  const handleChange = (value: K) => {
    setSelected(value);
    onChange(value);
  };

  return (
    <div className="select-none">
      <HuRadioGroup value={selected} onChange={handleChange}>
        {rootLabel && (
          <HuRadioGroup.Label
            as="div"
            className="font-semibold text-base text-baltic-sea dark:text-white mb-6 cursor-default"
          >
            {rootLabel}
          </HuRadioGroup.Label>
        )}
        <div className="space-y-4">
          {options.map((option) => (
            <HuRadioGroup.Option
              key={option.key}
              value={option.key}
              className={({ checked }) =>
                clsx([
                  'bg-white dark:bg-baltic-sea-jaguar relative flex rounded-lg px-5 py-2 focus:outline-none',
                  checked && 'border border-melon hover:border-melon',
                  !checked &&
                    'border border-white dark:border-baltic-sea-jaguar hover:border-platinum dark:hover:border-baltic-sea-jaguar',
                  'hover:bg-platinum dark:hover:bg-onyx',
                  'transition',
                  option.cursor === 'default' && 'cursor-default',
                  option.cursor === 'pointer' && 'cursor-pointer',
                  option.cursor === 'not-allowed' && 'cursor-not-allowed',
                ])
              }
              disabled={option.disabled}
            >
              {({ checked }) => {
                return (
                  <div
                    className={clsx([
                      'flex flex-1 items-center justify-between gap-4',
                      option.className && option.className,
                    ])}
                  >
                    {checked ? (
                      <span className="font-icon text-2xl text-melon">
                        radio_button_checked
                      </span>
                    ) : (
                      <span className="font-icon text-2xl text-cement-feet dark:laughing-jack">
                        radio_button_unchecked
                      </span>
                    )}
                    {option.icon && (
                      <div className="w-7 flex justify-center items-center">
                        {option.icon}
                      </div>
                    )}
                    <div className="flex flex-1 items-center">
                      <div className="text-sm">
                        <HuRadioGroup.Label
                          as="p"
                          className={clsx([
                            'text-base text-baltic-sea dark:text-mercury-pinkish',
                            checked && 'font-semibold',
                          ])}
                        >
                          {option.label}
                        </HuRadioGroup.Label>
                        {option.desc && (
                          <HuRadioGroup.Description
                            as="span"
                            className="text-sm text-cement-feet dark:text-mercury-mist"
                          >
                            <span>{option.desc}</span>
                          </HuRadioGroup.Description>
                        )}
                      </div>
                    </div>
                  </div>
                );
              }}
            </HuRadioGroup.Option>
          ))}
        </div>
      </HuRadioGroup>
    </div>
  );
}

export default RadioGroup;
