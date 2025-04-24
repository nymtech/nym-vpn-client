import clsx from 'clsx';
import { Field, Label, Textarea } from '@headlessui/react';
import { inputStates } from './common-styles';

export type TextAreaProps = {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  placeholder?: string;
  // The number of visible text lines
  rows?: number;
  resize?: 'none' | 'vertical' | 'horizontal' | 'both';
  spellCheck?: boolean;
  // Additional css style for textarea element
  className?: string;
  'data-testid'?: string;
};

function TextArea({
  value,
  onChange,
  rows = 2,
  spellCheck,
  resize,
  label,
  placeholder,
  className,
  ...rest
}: TextAreaProps) {
  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    onChange(e.target.value);
  };

  const getResizeClass = (resize: string) => {
    switch (resize) {
      case 'none':
        return 'resize-none';
      case 'vertical':
        return 'resize-y';
      case 'horizontal':
        return 'resize-x';
      case 'both':
        return 'resize';
      default:
        return 'resize';
    }
  };

  const testId = rest['data-testid'] || 'text-area';

  return (
    <Field
      className={clsx([
        'w-full flex flex-row items-center mb-2',
        label && 'relative',
      ])}
      data-testid={`${testId}-field`}
    >
      <Textarea
        id="passphrase"
        name="passphrase"
        value={value}
        aria-multiline={true}
        className={clsx([
          'text-base bg-faded-lavender dark:bg-ash transition',
          'w-full flex flex-row justify-between items-center py-4 px-4',
          'text-baltic-sea dark:text-white',
          'placeholder:text-iron dark:placeholder:text-bombay',
          ...inputStates,
          resize && getResizeClass(resize),
          label && 'relative',
          className,
        ])}
        placeholder={placeholder}
        onChange={handleChange}
        rows={rows}
        spellCheck={spellCheck}
        data-testid={testId}
        data-resize={resize}
        data-rows={rows}
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
    </Field>
  );
}

export default TextArea;
