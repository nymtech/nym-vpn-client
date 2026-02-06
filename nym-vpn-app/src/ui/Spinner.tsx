import { type } from '@tauri-apps/plugin-os';
import clsx from 'clsx';

function Spinner() {
  const os = type();

  return (
    <span
      className={clsx([
        'loader h-[22px] w-[22px]',
        os !== 'linux' && 'border-4',
        'border-baltic-sea dark:border-white border-b-transparent dark:border-b-transparent',
      ])}
      data-testid="button-spinner"
    ></span>
  );
}

export default Spinner;
