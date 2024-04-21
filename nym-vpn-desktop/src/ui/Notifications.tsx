import { useNotifications } from '../contexts';
import Snackbar from './Snackbar';

function Notifications() {
  const { current, next } = useNotifications();

  return (
    <Snackbar
      open={current !== null}
      onClose={() => next()}
      text={current?.text || ''}
      position={current?.position}
      closeIcon={current?.closeIcon}
    />
  );
}

export default Notifications;
