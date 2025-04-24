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
  'data-testid'?: string;
};

export type RadioGroupProps<K extends Key> = {
  options: RadioGroupOption<K>[];
  defaultValue?: K;
  onChange: (value: K) => void;
  rootLabel?: string;
  // either or not to show checked/unchecked circular icons
  radioIcons?: boolean;
  disabled?: boolean;
  'data-testid'?: string;
};

function RadioGroup<K extends Key>({
  options,
  defaultValue,
  onChange,
  rootLabel,
  radioIcons = true,
  disabled = false,
  ...rest
}: RadioGroupProps<K>) {
  const [selected, setSelected] = useState(defaultValue || options[0]);
  const testId = rest['data-testid'] || 'radio-group';

  const handleChange = (value: K) => {
    setSelected(value);
    onChange(value);
  };

  const checkedIcon = (checked: boolean) => {
    if (checked) {
      return (
        <span
          className="font-icon text-2xl text-malachite-moss dark:text-malachite"
          data-testid={`${testId}-checked-icon`}
        >
          radio_button_checked
        </span>
      );
    }
    return (
      <span
        className="font-icon text-2xl text-iron dark:text-bombay"
        data-testid={`${testId}-unchecked-icon`}
      >
        radio_button_unchecked
      </span>
    );
  };

  return (
    <div
      className="select-none"
      data-testid={testId}
      data-disabled={disabled ? 'true' : 'false'}
    >
      <HuRadioGroup
        value={selected}
        onChange={handleChange}
        disabled={disabled}
        data-testid={`${testId}-container`}
      >
        {rootLabel && (
          <Label
            as="div"
            className="font-medium text-base text-baltic-sea dark:text-white mb-6 cursor-default"
            data-testid={`${testId}-label`}
          >
            {rootLabel}
          </Label>
        )}
        <div className="space-y-4" data-testid={`${testId}-options-container`}>
          {options.map((option) => {
            const optionTestId =
              option['data-testid'] || `${testId}-option-${String(option.key)}`;

            return (
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
                data-testid={optionTestId}
                data-key={String(option.key)}
                data-disabled={option.disabled ? 'true' : 'false'}
              >
                {({ checked }) => {
                  return (
                    <>
                      {option.tooltip && (
                        <div
                          className="tooltip -mt-8 -ml-2 max-w-[90%]"
                          data-testid={`${optionTestId}-tooltip`}
                        >
                          {option.tooltip}
                        </div>
                      )}
                      <div
                        className={clsx([
                          'overflow-hidden flex flex-1 items-center justify-start gap-5',
                          option.className && option.className,
                        ])}
                        data-testid={`${optionTestId}-content`}
                        data-checked={checked ? 'true' : 'false'}
                      >
                        {radioIcons && checkedIcon(checked)}
                        {option.icon && (
                          <div
                            className="w-7 flex justify-center items-center"
                            data-testid={`${optionTestId}-icon-container`}
                          >
                            {typeof option.icon === 'function'
                              ? option.icon(checked)
                              : option.icon}
                          </div>
                        )}
                        <div
                          className="min-w-0 flex flex-col justify-center"
                          data-testid={`${optionTestId}-text-container`}
                        >
                          <Label
                            as="p"
                            className={clsx([
                              'truncate text-base text-baltic-sea dark:text-white',
                            ])}
                            data-testid={`${optionTestId}-label`}
                          >
                            {option.label}
                          </Label>
                          {option.desc && (
                            <Description
                              as="span"
                              className="truncate text-sm text-iron dark:text-bombay"
                              data-testid={`${optionTestId}-description`}
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
            );
          })}
        </div>
      </HuRadioGroup>
    </div>
  );
}

export default RadioGroup;
