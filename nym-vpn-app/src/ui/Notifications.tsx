import { Viewport } from '@radix-ui/react-toast';
import clsx from 'clsx';
import { useInAppNotify } from '../contexts';
import { Toast } from './index';

function Notifications() {
  const { current, onClose } = useInAppNotify();

  const handleOpenChange = (open: boolean) => {
    if (open) {
      return;
    }
    onClose();
    if (current?.onClose) {
      current.onClose();
    }
  };

  return (
    <>
      <Viewport
        className={clsx(
          'fixed top-20 right-0 z-99 m-0 flex w-full',
          'list-none flex-col gap-2.5 outline-none',
          'cursor-default select-none',
        )}
      />
      <Toast
        open={!!current}
        message={current?.message || ''}
        onOpenChange={handleOpenChange}
        close={current?.close}
        duration={current?.duration}
        type={current?.type}
        clickAway={current?.clickAway}
      />
    </>
  );
}

export default Notifications;
