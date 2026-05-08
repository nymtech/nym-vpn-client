import React from 'react';
import clsx from 'clsx';
import { Field, Input } from '@headlessui/react';
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
    <Field className={clsx(['relative flex w-full flex-row items-center'])}>
      <Input
        id="passphrase"
        name="passphrase"
        type="text"
        ref={ref}
        defaultValue={defaultValue}
        value={value}
        aria-multiline={true}
        className={clsx([
          'relative text-base transition',
          'flex w-full flex-row items-center justify-between px-4 py-3',
          !disabled && 'text-text-primary',
          disabled && 'text-text-secondary',
          'placeholder:text-iron dark:placeholder:text-bombay',
          ...inputStates,
          getColorClass(),
          className,
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
      {leftIcon && (
        <MsIcon
          icon={leftIcon}
          className="text-baltic-sea dark:text-bombay absolute left-3 hover:cursor-text"
        />
      )}
      {clearable && !!value && (
        <ButtonIcon
          color="chalk"
          icon="cancel"
          className="text-baltic-sea dark:text-bombay absolute top-2.5 right-1 hover:cursor-pointer"
          onClick={handleClear}
        />
      )}
    </Field>
  );
}

export default TextInput;
