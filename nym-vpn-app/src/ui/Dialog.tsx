import { ReactNode } from 'react';
import clsx from 'clsx';
import {
  DialogBackdrop,
  DialogPanel,
  Dialog as HuDialog,
} from '@headlessui/react';
import { useMainState } from '../contexts';

export type DialogProps = {
  open: boolean;
  onClose: () => void;
  children?: ReactNode;
  className?: string;
  'data-testid'?: string;
};

function Dialog({ open, onClose, children, className, ...rest }: DialogProps) {
  // manually injecting the theme is required as dialogs are rendered
  // outside the main app container (using a portal)
  const { uiTheme } = useMainState();
  const testId = rest['data-testid'] || 'dialog';

  return (
    <HuDialog
      as="div"
      className={clsx([
        uiTheme === 'dark' && 'dark',
        'relative z-50 focus:outline-hidden select-none cursor-default',
      ])}
      open={open}
      onClose={onClose}
      data-testid={testId}
      data-open={open ? 'true' : 'false'}
      data-theme={uiTheme}
    >
      <DialogBackdrop
        transition
        className={clsx([
          'fixed inset-0 bg-black/30 duration-200 ease-out data-closed:opacity-0',
        ])}
        data-testid={`${testId}-backdrop`}
      />
      <div
        className="fixed inset-0 z-50 w-screen overflow-y-auto"
        data-testid={`${testId}-container`}
      >
        <div
          className="flex min-h-full items-center justify-center p-4 mx-4"
          data-testid={`${testId}-wrapper`}
        >
          <DialogPanel
            transition
            className={clsx(
              [
                'text-base min-w-80 overflow-x-hidden',
                'max-w-md rounded-xl bg-white dark:bg-charcoal p-6',
                'duration-200 ease-out data-closed:opacity-0',
              ],
              className,
            )}
            data-testid={`${testId}-panel`}
          >
            {children}
          </DialogPanel>
        </div>
      </div>
    </HuDialog>
  );
}

export default Dialog;
