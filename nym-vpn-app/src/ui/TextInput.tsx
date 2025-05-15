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
  'data-testid'?: string;
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
  ...rest
}: TextInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  const testId = rest['data-testid'] || 'text-input';

  return (
    <Field
      className={clsx([
        'w-full flex flex-row items-center',
        label && 'relative',
      ])}
      data-testid={`${testId}-field`}
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
          'text-baltic-sea dark:text-white',
          'placeholder:text-iron dark:placeholder:text-bombay',
          ...inputStates,
          className,
          label && 'relative',
          leftIcon && 'pl-11',
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        spellCheck={spellCheck}
        autoFocus={autoFocus}
        data-testid={testId}
        data-has-left-icon={leftIcon ? 'true' : 'false'}
      />
      {label && (
        <Label
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-white',
            'bg-faded-lavender dark:bg-ash text-xs',
          ])}
          data-testid={`${testId}-label`}
        >
          {label}
        </Label>
      )}
      {leftIcon && (
        <MsIcon
          icon={leftIcon}
          className="absolute left-3 text-baltic-sea dark:text-bombay hover:cursor-text"
          data-testid={`${testId}-left-icon`}
        />
      )}
    </Field>
  );
}

export default TextInput;
