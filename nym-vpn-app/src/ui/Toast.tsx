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
  message: string;
  // Show a button to dismiss the toast before its duration has elapsed
  close?: boolean;
  action?: React.ReactNode;
  type?: 'error' | 'warn' | 'info';
  clickAway?: boolean;
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

  const CloseButton = () => (
    <motion.button
      key="snackbar-close-button"
      initial={{ opacity: 0.7 }}
      whileHover={{ opacity: 1 }}
      whileTap={{ opacity: 1 }}
      transition={{ duration: 0.15 }}
      className="w-6 ml-4 focus:outline-hidden text-black dark:text-white cursor-default"
      onClick={() => handleOpenChange(false)}
    >
      <MsIcon icon="close" className="text-3xl" />
    </motion.button>
  );

  return (
    <AnimatePresence>
      {open && (
        <div ref={ref}>
          <Root
            onOpenChange={handleOpenChange}
            duration={duration}
            asChild
            forceMount
          >
            <motion.ul
              className={clsx(
                'mx-6 px-5 py-4 min-w-54 max-w-lg',
                'flex justify-between items-center rounded-lg select-none cursor-default',
                'text-baltic-sea dark:text-white bg-white dark:bg-charcoal',
                type === 'error' &&
                  'border-2 text-aphrodisiac! dark:text-aphrodisiac! border-aphrodisiac',
                type === 'warn' && 'border-2 border-king-nacho',
                type === 'info' && 'border-2 border-iron dark:border-bombay',
              )}
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.1, ease: 'easeOut' }}
              layout
            >
              {title && <div>{title}</div>}
              <div>{message}</div>
              {close && <CloseButton />}
            </motion.ul>
          </Root>
        </div>
      )}
    </AnimatePresence>
  );
}

export default Toast;
