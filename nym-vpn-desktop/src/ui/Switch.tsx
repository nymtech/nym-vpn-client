import clsx from 'clsx';
import { Switch as HeadlessUiSwitch } from '@headlessui/react';

type Props = {
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
};

function Switch({ checked, onChange, disabled }: Props) {
  return (
    <HeadlessUiSwitch
      checked={checked}
      onChange={onChange}
      className={clsx([
        checked ? 'bg-melon' : 'bg-mercury-pinkish dark:bg-gun-powder',
        'relative inline-flex h-6 w-11 items-center rounded-full',
      ])}
      disabled={disabled}
    >
      <span
        className={clsx([
          checked ? 'translate-x-6' : 'translate-x-1',
          checked
            ? 'bg-white dark:bg-baltic-sea'
            : 'bg-cement-feet dark:bg-mercury-mist',
          'inline-block h-4 w-4 transform rounded-full transition',
        ])}
      />
    </HeadlessUiSwitch>
  );
}

export default Switch;
