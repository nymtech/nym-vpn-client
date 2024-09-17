import { ReactNode } from 'react';
import clsx from 'clsx';
import {
  DialogBackdrop,
  DialogPanel,
  Dialog as HuDialog,
} from '@headlessui/react';
import { useMainState } from '../contexts';

type DialogProps = {
  open: boolean;
  onClose: () => void;
  children?: ReactNode;
};

function Dialog({ open, onClose, children }: DialogProps) {
  // manually injecting the theme is required as dialogs are rendered
  // outside the main app container (using a portal)
  const { uiTheme } = useMainState();

  return (
    <HuDialog
      as="div"
      className={clsx([
        uiTheme === 'Dark' && 'dark',
        'relative z-50 focus:outline-none select-none cursor-default',
      ])}
      open={open}
      onClose={onClose}
    >
      <DialogBackdrop
        className={clsx([
          'fixed inset-0 bg-black/30',
          'duration-150 ease-in-out data-[closed]:opacity-0',
        ])}
      />
      <div className="fixed inset-0 z-50 w-screen overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4 mx-4">
          <DialogPanel
            transition
            className={clsx([
              'text-base min-w-80',
              'w-full max-w-md rounded-xl bg-white dark:bg-oil p-6',
              'flex flex-col items-center gap-6',
              'duration-150 ease-in-out data-[closed]:ease-out data-[closed]:scale-95 data-[closed]:opacity-0',
            ])}
          >
            {children}
          </DialogPanel>
        </div>
      </div>
    </HuDialog>
  );
}

export default Dialog;
