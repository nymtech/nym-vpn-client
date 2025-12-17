import React from 'react';
import clsx from 'clsx';
import { Field, Input, Label } from '@headlessui/react';
import { inputStates } from './common-styles';
import MsIcon from './MsIcon';
import ButtonIcon from './ButtonIcon';

export type TextInputProps = {
  ref?: React.RefObject<HTMLInputElement | null>;
  // default value for uncontrolled input
  defaultValue?: string;
  // value for controlled input
  value?: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  spellCheck?: boolean;
  autoFocus?: boolean;
  // custom input style
  className?: string;
  leftIcon?: string;
  readonly?: boolean;
  clearable?: boolean;
  color?: 'default' | 'gray';
  disabled?: boolean;
};

function TextInput({
  ref,
  defaultValue,
  value,
  onChange,
  spellCheck,
  label,
  placeholder,
  leftIcon,
  autoFocus,
  className,
  clearable = false,
  color = 'default',
  disabled = false,
}: TextInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  const handleClear = () => {
    onChange('');
  };

  const getColorClass = () => {
    switch (color) {
      case 'default':
        return 'bg-faded-lavender dark:bg-ash';
      case 'gray':
        return 'bg-white dark:bg-charcoal';
    }
  };

  return (
    <Field
      className={clsx([
        'w-full flex flex-row items-center',
        label && 'relative',
      ])}
    >
      <Input
        id="passphrase"
        name="passphrase"
        type="text"
        ref={ref}
        defaultValue={defaultValue}
        value={value}
        aria-multiline={true}
        className={clsx([
          'text-base transition',
          'w-full flex flex-row justify-between items-center py-3 px-4',
          !disabled && 'text-baltic-sea dark:text-white',
          disabled && 'text-iron dark:text-bombay',
          'placeholder:text-iron dark:placeholder:text-bombay',
          ...inputStates,
          getColorClass(),
          className,
          label && 'relative',
          leftIcon && 'pl-11',
          clearable && 'pr-11',
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        spellCheck={spellCheck}
        autoFocus={autoFocus}
        data-test-has-left-icon={leftIcon ? 'true' : 'false'}
        disabled={disabled}
      />
      {label && (
        <Label
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-white',
            getColorClass(),
            'text-xs',
          ])}
        >
          {label}
        </Label>
      )}
      {leftIcon && (
        <MsIcon
          icon={leftIcon}
          className="absolute left-3 text-baltic-sea dark:text-bombay hover:cursor-text"
        />
      )}
      {clearable && !!value && (
        <ButtonIcon
          color="chalk"
          icon="cancel"
          className="absolute top-2.5 right-1 text-baltic-sea dark:text-bombay hover:cursor-pointer"
          onClick={handleClear}
        />
      )}
    </Field>
  );
}

export default TextInput;
