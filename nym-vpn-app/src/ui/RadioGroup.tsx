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
  descWrap?: boolean;
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
  const [selected, setSelected] = useState<K | undefined>(
    defaultValue || options[0]?.key,
  );
  const testId = rest['data-testid'] || 'radio-group';

  const handleChange = (value: K) => {
    setSelected(value);
    onChange(value);
  };

  const checkedIcon = (checked: boolean) => {
    if (checked) {
      return (
        <span
          className="font-icon text-brand-primary text-2xl"
          data-testid={`${testId}-checked-icon`}
        >
          radio_button_checked
        </span>
      );
    }
    return (
      <span
        className="font-icon text-text-secondary text-2xl"
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
      data-test-disabled={disabled ? 'true' : 'false'}
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
            className="text-text-primary mb-6 cursor-default text-base font-medium"
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
                    'bg-surface-elev relative flex rounded-2xl px-5 py-2 focus:outline-hidden',
                    checked &&
                      'border-brand-primary hover:border-brand-primary border',
                    checked &&
                      'dark:border-brand-primary dark:hover:border-brand-primary',
                    !checked && 'border-surface-elev border',
                    !option.disabled &&
                      !checked &&
                      'hover:border-surface-elev/85',
                    !option.disabled && 'hover:bg-surface-elev/85',
                    'transition-noborder cursor-default',
                    option.tooltip && 'attach-tooltip',
                    disabled && 'hover:bg-surface-elev! opacity-50',
                  ])
                }
                disabled={option.disabled}
                data-testid={optionTestId}
                data-key={String(option.key)}
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
                          'flex flex-1 items-center justify-start gap-5 overflow-hidden',
                          option.className && option.className,
                        ])}
                        data-testid={`${optionTestId}-content`}
                      >
                        {radioIcons && checkedIcon(checked)}
                        {option.icon && (
                          <div
                            className="flex w-7 items-center justify-center"
                            data-testid={`${optionTestId}-icon-container`}
                          >
                            {typeof option.icon === 'function'
                              ? option.icon(checked)
                              : option.icon}
                          </div>
                        )}
                        <div
                          className="flex min-w-0 flex-col justify-center"
                          data-testid={`${optionTestId}-text-container`}
                        >
                          <Label
                            as="p"
                            className={clsx([
                              'text-text-primary truncate text-base',
                            ])}
                            data-testid={`${optionTestId}-label`}
                          >
                            {option.label}
                          </Label>
                          {option.desc && (
                            <Description
                              as="span"
                              className={clsx(
                                'text-text-secondary text-xs',
                                !option.descWrap && 'truncate',
                              )}
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
