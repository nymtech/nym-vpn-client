import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { Root } from '@radix-ui/react-toast';
import { useClickAway } from '../hooks';
import MsIcon from './MsIcon';

export type ToastProps = {
  // Whether the toast is open when it is initially rendered
  defaultOpen?: boolean;
  // The controlled state of the toast
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  // The time in ms that should elapse before automatically closing the toast
  duration?: number;
  title?: string;
  message?: string;
  // Show a button to dismiss the toast before its duration has elapsed
  close?: boolean;
  action?: React.ReactNode;
  type?: 'error' | 'warn' | 'info' | 'ghost';
  clickAway?: boolean;
  'data-testid'?: string;
  content?: React.ReactNode;
};

function Toast({
  defaultOpen = true,
  open: openCtrl,
  onOpenChange,
  duration = 2000,
  title,
  message,
  close,
  type = 'info',
  clickAway = false,
  content,
  ...rest
}: ToastProps) {
  const [open, setOpen] = useState(() => {
    if (openCtrl !== undefined) {
      return openCtrl;
    }
    return defaultOpen;
  });

  const ref = useClickAway<HTMLDivElement>({
    on: () => {
      handleOpenChange(false);
    },
    disabled: !clickAway,
  });

  useEffect(() => {
    if (openCtrl !== undefined) {
      setOpen(openCtrl);
    }
  }, [openCtrl]);

  const handleOpenChange = (open: boolean) => {
    setOpen(open);
    if (onOpenChange) {
      onOpenChange(open);
    }
  };

  const testId = rest['data-testid'] || 'toast';

  const CloseButton = () => (
    <motion.button
      key="snackbar-close-button"
      initial={{ opacity: 0.7 }}
      whileHover={{ opacity: 1 }}
      whileTap={{ opacity: 1 }}
      transition={{ duration: 0.15 }}
      className="ml-4 w-6 cursor-default text-white focus:outline-hidden dark:text-black"
      onClick={() => handleOpenChange(false)}
      data-testid={`${testId}-close-button`}
    >
      <MsIcon
        icon="close"
        className="text-3xl"
        data-testid={`${testId}-close-icon`}
      />
    </motion.button>
  );

  return (
    <AnimatePresence>
      {open && (
        <div ref={ref} data-testid={`${testId}-container`}>
          <Root
            onOpenChange={handleOpenChange}
            duration={duration}
            asChild
            forceMount
          >
            <motion.ul
              className={clsx(
                'mx-auto w-full max-w-lg px-5 py-4',
                'flex cursor-default items-center justify-between rounded-lg select-none',
                'dark:text-baltic-sea bg-charcoal text-white dark:bg-white',
                type === 'error' &&
                  'text-aphrodisiac! dark:text-aphrodisiac! border-aphrodisiac border-2',
                type === 'warn' && 'border-king-nacho border-2',
                type === 'info' && 'dark:border-iron border-bombay border-2',
                type === 'ghost' && 'border-2 border-transparent',
              )}
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.1, ease: 'easeOut' }}
              layout
              data-testid={testId}
              data-test-type={type}
              data-test-duration={duration}
              data-test-open={open ? 'true' : 'false'}
            >
              {content ? (
                content
              ) : (
                <>
                  {title && <div>{title}</div>}
                  {message && <div>{message}</div>}
                </>
              )}
              {close && <CloseButton />}
            </motion.ul>
          </Root>
        </div>
      )}
    </AnimatePresence>
  );
}

export default Toast;
