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
          className="h-4 w-4 leading-none text-cheddar dark:text-king-nacho"
        />
      );
    case 'info':
      return (
        <MsIcon icon="info" className="h-4 w-4 leading-none text-baltic-sea" />
      );
    case 'success':
      return (
        <MsIcon
          icon="check_circle"
          className="h-4 w-4 leading-none text-malachite-moss dark:text-malachite"
        />
      );
    default:
      return (
        <MsIcon
          icon="info"
          className="h-4 w-4 leading-none text-malachite-moss dark:text-malachite"
        />
      );
  }
}
