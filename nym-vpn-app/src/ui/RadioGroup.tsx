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
  icon?: React.ReactNode;
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
        <span className="font-icon text-2xl text-melon">
          radio_button_checked
        </span>
      );
    }
    return (
      <span className="font-icon text-2xl text-cement-feet dark:laughing-jack">
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
            className="font-semibold text-base text-baltic-sea dark:text-white mb-6 cursor-default"
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
                  'bg-white dark:bg-baltic-sea-jaguar relative flex rounded-lg px-5 py-2 focus:outline-none',
                  checked && 'border border-melon hover:border-melon',
                  !checked &&
                    'border border-white dark:border-baltic-sea-jaguar',
                  !option.disabled &&
                    !checked &&
                    'hover:border-platinum dark:hover:border-baltic-sea-jaguar',
                  !option.disabled && 'hover:bg-platinum dark:hover:bg-onyx',
                  'transition-noborder cursor-default',
                  option.tooltip && 'attach-tooltip',
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
                          {option.icon}
                        </div>
                      )}
                      <div className="min-w-0 flex flex-col justify-center">
                        <Label
                          as="p"
                          className={clsx([
                            'truncate text-base text-baltic-sea dark:text-mercury-pinkish',
                          ])}
                        >
                          {option.label}
                        </Label>
                        {option.desc && (
                          <Description
                            as="span"
                            className="truncate text-sm text-cement-feet dark:text-mercury-mist"
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
