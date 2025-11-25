import React, { useState } from 'react';
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
};

function TextInput({
  ref,
  defaultValue,
  onChange,
  spellCheck,
  label,
  placeholder,
  leftIcon,
  autoFocus,
  className,
  clearable = false,
}: TextInputProps) {
  const [value, setValue] = useState(defaultValue || '');

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setValue(e.target.value);
    onChange(e.target.value);
  };

  const handleClear = () => {
    setValue('');
    onChange('');
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
          'text-base bg-faded-lavender dark:bg-ash transition',
          'w-full flex flex-row justify-between items-center py-3 px-4',
          'text-baltic-sea dark:text-white',
          'placeholder:text-iron dark:placeholder:text-bombay',
          ...inputStates,
          className,
          label && 'relative',
          leftIcon && 'pl-11',
          'pr-11',
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        spellCheck={spellCheck}
        autoFocus={autoFocus}
        data-test-has-left-icon={leftIcon ? 'true' : 'false'}
      />
      {label && (
        <Label
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-white',
            'bg-faded-lavender dark:bg-ash text-xs',
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
      {clearable && value.length > 0 && (
        <ButtonIcon
          color="chalk"
          icon="cancel"
          className="absolute right-1 text-baltic-sea dark:text-bombay hover:cursor-pointer"
          onClick={handleClear}
        />
      )}
    </Field>
  );
}

export default TextInput;
