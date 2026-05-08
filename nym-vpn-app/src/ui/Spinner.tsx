import { type } from '@tauri-apps/plugin-os';
import clsx from 'clsx';

function Spinner({ className }: { className?: string }) {
  const os = type();

  return (
    <span
      className={clsx([
        'loader h-[22px] w-[22px]',
        os !== 'linux' && 'border-4',
        'border-baltic-sea border-b-transparent dark:border-white dark:border-b-transparent',
        className,
      ])}
      data-testid="button-spinner"
    ></span>
  );
}

export default Spinner;
