import { ClickAwayListener, useSnackbar } from '@mui/base';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { useMainState } from '../contexts';
import MsIcon from './MsIcon';

export type SnackbarProps = {
  open: boolean;
  onClose: () => void;
  text: string;
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
        'fixed z-50 inset-x-0 mx-5 px-5 py-4',
        position === 'top' ? 'top-6' : 'bottom-6',
        'flex justify-between items-center rounded-lg',
        'text-baltic-sea dark:text-mercury-pinkish bg-seashell dark:bg-poivreNoir',
      ])}
      {...getRootProps()}
    >
      <p>{text}</p>
      {closeIcon && (
        <motion.button
          initial={{ opacity: 0.7 }}
          whileHover={{ opacity: 1 }}
          transition={{ duration: 0.05 }}
          className="w-6 ml-4 focus:outline-none text-black dark:text-white"
          onClick={() => onClose()}
        >
          <MsIcon icon="close" style="text-3xl" />
        </motion.button>
      )}
    </motion.div>
  );

  if (!open) {
    return null;
  }

  return (
    <div className={clsx([uiTheme === 'Dark' && 'dark'])}>
      {clickAway ? (
        <ClickAwayListener onClickAway={onClickAway}>
          {snackbar}
        </ClickAwayListener>
      ) : (
        <>{snackbar}</>
      )}
    </div>
  );
}

export default Snackbar;
