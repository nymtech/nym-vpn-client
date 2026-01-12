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
        checked ? 'bg-malachite' : 'bg-bombay dark:bg-iron',
        'relative inline-flex h-7 w-11 min-w-11 items-center rounded-full cursor-default',
        className,
      ])}
      disabled={disabled}
      data-testid={testId}
      data-test-checked={checked ? 'true' : 'false'}
      data-test-disabled={disabled ? 'true' : 'false'}
    >
      <span
        className={clsx([
          checked ? 'translate-x-5' : 'translate-x-1',
          'bg-white h-5 w-5',
          'inline-block transform rounded-full transition',
        ])}
        data-testid={`${testId}-thumb`}
      />
    </HuSwitch>
  );
}

export default Switch;
