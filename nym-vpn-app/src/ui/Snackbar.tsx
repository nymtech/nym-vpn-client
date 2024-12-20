import { ClickAwayListener, useSnackbar } from '@mui/base';
import { AnimatePresence, motion } from 'framer-motion';
import clsx from 'clsx';
import { useMainState } from '../contexts';
import MsIcon from './MsIcon';

export type SnackbarProps = {
  open: boolean;
  onClose: () => void;
  text: string;
  // Number of ms to wait before automatically close the snackbar
  autoHideDuration?: number;
  closeIcon?: boolean;
  clickAway?: boolean;
  position?: 'top' | 'bottom';
};

function Snackbar({
  open,
  onClose,
  text,
  autoHideDuration = 2000,
  closeIcon,
  clickAway,
  position = 'top',
}: SnackbarProps) {
  const { uiTheme } = useMainState();

  const { getRootProps, onClickAway } = useSnackbar({
    onClose,
    open,
    autoHideDuration,
  });

  const snackbar = (
    <motion.div
      initial={{ opacity: 0, y: position === 'top' ? -10 : 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.1 }}
      exit={{ opacity: 0 }}
      className={clsx([
        'fixed z-30 inset-x-0 mx-6 px-5 py-4 min-w-56',
        position === 'top' ? 'top-20' : 'bottom-6',
        'flex justify-between items-center rounded-lg select-none cursor-default',
        'text-baltic-sea dark:text-mercury-pinkish bg-seashell dark:bg-poivre-noir',
      ])}
      {...getRootProps()}
    >
      <p>{text}</p>
      {closeIcon && (
        <motion.button
          key="snackbar-close-button"
          initial={{ opacity: 0.7 }}
          whileHover={{ opacity: 1 }}
          whileTap={{ opacity: 1 }}
          transition={{ duration: 0.15 }}
          className="w-6 ml-4 focus:outline-none text-black dark:text-white cursor-default"
          onClick={() => onClose()}
        >
          <MsIcon icon="close" className="text-3xl" />
        </motion.button>
      )}
    </motion.div>
  );

  return (
    <AnimatePresence>
      {open && (
        <div className={clsx([uiTheme === 'Dark' && 'dark'])}>
          {clickAway ? (
            <ClickAwayListener onClickAway={onClickAway}>
              {snackbar}
            </ClickAwayListener>
          ) : (
            snackbar
          )}
        </div>
      )}
    </AnimatePresence>
  );
}

export default Snackbar;
