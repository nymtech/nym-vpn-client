import { useNotifications } from '../contexts';
import Snackbar from './Snackbar';

function Notifications() {
  const { current, next } = useNotifications();

  const onClose = () => {
    next();
    if (current?.onClose) {
      current?.onClose();
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
    />
  );
}

export default Notifications;
