import React from 'react';
import clsx from 'clsx';
import MsIcon from './MsIcon';

type TextInputProps = {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  spellCheck?: boolean;
  autoFocus?: boolean;
  // custom input style
  style?: string;
  leftIcon?: string;
};

/* eslint-disable jsx-a11y/no-autofocus */
function TextInput({
  value,
  onChange,
  spellCheck,
  label,
  placeholder,
  leftIcon,
  autoFocus,
  style,
}: TextInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  return (
    <div
      className={clsx([
        'w-full flex flex-row items-center',
        label && 'relative',
      ])}
    >
      <input
        id="passphrase"
        name="passphrase"
        value={value}
        aria-multiline={true}
        className={clsx([
          'text-base bg-blanc-nacre dark:bg-baltic-sea',
          'border-cement-feet dark:border-gun-powder border rounded-lg',
          'hover:ring-1 hover:border-black hover:dark:border-white',
          'focus:border-black focus:dark:border-white',
          'focus:outline-none focus:ring-2 ring-black dark:ring-white',
          'w-full flex flex-row justify-between items-center py-3 px-4',
          'text-baltic-sea dark:text-mercury-pinkish',
          'placeholder:text-cement-feet placeholder:dark:text-mercury-mist',
          style,
          label && 'relative',
          leftIcon && 'pl-11',
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        spellCheck={spellCheck}
        autoFocus={autoFocus}
      />
      {label && (
        <div
          className={clsx([
            'select-none absolute left-3 -top-2 px-1',
            'dark:text-mercury-pinkish',
            'bg-blanc-nacre dark:bg-baltic-sea text-xs',
          ])}
        >
          {label}
        </div>
      )}
      {leftIcon && (
        <MsIcon
          icon={leftIcon}
          style="absolute left-3 text-baltic-sea dark:text-laughing-jack"
        />
      )}
    </div>
  );
}

export default TextInput;
