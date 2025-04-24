import clsx from 'clsx';
import { Switch as HuSwitch } from '@headlessui/react';

export type SwitchProps = {
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
  'data-testid'?: string;
};

function Switch({ checked, onChange, disabled, ...rest }: SwitchProps) {
  const testId = rest['data-testid'] || 'switch';

  return (
    <HuSwitch
      checked={checked}
      onChange={onChange}
      className={clsx([
        checked ? 'bg-malachite' : 'bg-bombay/60 dark:bg-iron',
        'relative inline-flex h-7 w-11 min-w-11 items-center rounded-full cursor-default',
      ])}
      disabled={disabled}
      data-testid={testId}
      data-checked={checked ? 'true' : 'false'}
      data-disabled={disabled ? 'true' : 'false'}
    >
      <span
        className={clsx([
          checked ? 'translate-x-5' : 'translate-x-1',
          checked
            ? 'bg-faded-lavender dark:bg-ash h-5 w-5'
            : 'bg-iron dark:bg-bombay h-4 w-4',
          'inline-block transform rounded-full transition',
        ])}
        data-testid={`${testId}-thumb`}
      />
    </HuSwitch>
  );
}

export default Switch;
