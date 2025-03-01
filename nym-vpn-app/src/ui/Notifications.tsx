import { useInAppNotify } from '../contexts';
import Snackbar from './Snackbar';

function Notifications() {
  const { current, next } = useInAppNotify();

  const onClose = () => {
    next();
    if (current?.onClose) {
      current.onClose();
    }
  };

  return (
    <Snackbar
      open={current !== null}
      onClose={onClose}
      text={current?.text || ''}
      position={current?.position}
      closeIcon={current?.closeIcon}
      autoHideDuration={current?.autoHideDuration}
      clickAway={current?.clickAway}
      type={current?.type}
    />
  );
}

export default Notifications;
