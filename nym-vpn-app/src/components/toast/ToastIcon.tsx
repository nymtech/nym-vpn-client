import { MsIcon } from '../../ui';
import { ToastAddData } from '../../hooks';

export function ToastIcon({ type }: { type?: ToastAddData['type'] }) {
  switch (type) {
    case 'error':
      return (
        <MsIcon icon="error" className="h-4 w-4 leading-none text-white" />
      );
    case 'warn':
      return (
        <MsIcon
          icon="fmd_bad"
          className="text-cheddar dark:text-king-nacho h-4 w-4 leading-none"
        />
      );
    case 'info':
      return (
        <MsIcon icon="info" className="text-baltic-sea h-4 w-4 leading-none" />
      );
    case 'success':
      return (
        <MsIcon
          icon="check_circle"
          className="text-primary h-4 w-4 leading-none"
        />
      );
    default:
      return (
        <MsIcon icon="info" className="text-primary h-4 w-4 leading-none" />
      );
  }
}
