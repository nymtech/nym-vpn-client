import clsx from 'clsx';
import { Switch as HuSwitch } from '@headlessui/react';

export type SwitchProps = {
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
  className?: string;
  'data-testid'?: string;
};

function Switch({
  checked,
  onChange,
  disabled,
  className,
  ...rest
}: SwitchProps) {
  const testId = rest['data-testid'] || 'switch';

  return (
    <HuSwitch
      checked={checked}
      onChange={onChange}
      className={clsx([
        checked ? 'bg-primary' : 'bg-bombay dark:bg-iron',
        'relative inline-flex h-7 w-11 min-w-11 cursor-default items-center rounded-full',
        className,
      ])}
      disabled={disabled}
      data-testid={testId}
      data-test-checked={checked ? 'true' : 'false'}
      data-test-disabled={disabled ? 'true' : 'false'}
    >
      <span
        className={clsx([
          checked
            ? 'translate-x-5 rtl:-translate-x-5'
            : 'translate-x-1 rtl:-translate-x-1',
          'h-5 w-5',
          'bg-gray dark:bg-background',
          'inline-block transform rounded-full transition',
        ])}
        data-testid={`${testId}-thumb`}
      />
    </HuSwitch>
  );
}

export default Switch;
