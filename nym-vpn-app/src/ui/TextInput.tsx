import React from 'react';
import clsx from 'clsx';
import { Field, Input, Label } from '@headlessui/react';
import { inputStates } from './common-styles';
import MsIcon from './MsIcon';

export type TextInputProps = {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  spellCheck?: boolean;
  autoFocus?: boolean;
  // custom input style
  className?: string;
  leftIcon?: string;
  readonly?: boolean;
};

function TextInput({
  value,
  onChange,
  spellCheck,
  label,
  placeholder,
  leftIcon,
  autoFocus,
  className,
}: TextInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
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
        value={value}
        aria-multiline={true}
        className={clsx([
          'text-base bg-faded-lavender dark:bg-ash transition',
          'w-full flex flex-row justify-between items-center py-3 px-4',
          'text-baltic-sea dark:text-mercury-pinkish',
          'placeholder:text-cement-feet placeholder:dark:text-mercury-mist',
          ...inputStates,
          className,
          label && 'relative',
          leftIcon && 'pl-11',
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        spellCheck={spellCheck}
        autoFocus={autoFocus}
      />
      {label && (
        <Label
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-mercury-pinkish',
            'bg-faded-lavender dark:bg-ash text-xs',
          ])}
        >
          {label}
        </Label>
      )}
      {leftIcon && (
        <MsIcon
          icon={leftIcon}
          className="absolute left-3 text-baltic-sea dark:text-laughing-jack hover:cursor-text"
        />
      )}
    </Field>
  );
}

export default TextInput;
